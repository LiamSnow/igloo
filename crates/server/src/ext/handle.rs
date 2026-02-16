use super::EXTS_DIR;
use crate::core::{IglooError, IglooRequest};
use crate::{DATA_DIR, PACKAGES_DIR};
use futures_util::{SinkExt, StreamExt};
use igloo_interface::id::{ExtensionID, ExtensionIndex};
use igloo_interface::ipc::codec::LengthDelimitedJSONCodec;
use igloo_interface::ipc::{
    DATA_PATH_ENV_VAR, ExtensionToIgloo, IReader, IWriter, IglooToExtension,
};
use std::path::{self, PathBuf};
use std::sync::Arc;
use std::{io, process::Stdio};
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::{fs, net::UnixListener};
use tokio_util::codec::{FramedRead, FramedWrite};

pub const SOCKET: &str = "igloo.sock";
pub const EXECUTABLE: &str = "./ext";

#[derive(Debug)]
pub struct ExtensionHandle {
    pub id: ExtensionID,
    pub index: ExtensionIndex,
    pub core_tx: kanal::AsyncSender<IglooRequest>,
    pub ext_rx: kanal::AsyncReceiver<ExtensionRequest>,
    pub writer: IWriter,
    pub reader: IReader,
    pub process: Child,
}

#[derive(Debug)]
pub struct ExtensionProcess {
    pub id: ExtensionID,
    pub index: ExtensionIndex,
    pub process: RwLock<Child>,
}

#[derive(Clone)]
pub enum ExtensionRequest {
    Msg(IglooToExtension),
    Flush,
}

fn cwd(id: &ExtensionID) -> PathBuf {
    let mut path = PACKAGES_DIR.get().unwrap().clone();
    path.push(EXTS_DIR);
    path.push(&id.0);
    path
}

fn data_path(id: &ExtensionID) -> io::Result<PathBuf> {
    let mut path = DATA_DIR.get().unwrap().clone();
    path.push(EXTS_DIR);
    path.push(&id.0);
    path::absolute(&path)
}

fn socket_path(id: &ExtensionID) -> PathBuf {
    let mut path = PACKAGES_DIR.get().unwrap().clone();
    path.push(EXTS_DIR);
    path.push(&id.0);
    path.push(SOCKET);
    path
}

impl ExtensionHandle {
    pub async fn new(
        id: ExtensionID,
        to_core_tx: kanal::Sender<IglooRequest>,
    ) -> Result<(Self, kanal::Sender<ExtensionRequest>), IglooError> {
        println!("Initializing Extension {id}");

        let cwd = cwd(&id);
        let data_path = data_path(&id)?;
        let socket_path = socket_path(&id);

        fs::create_dir_all(&data_path).await?;
        _ = fs::remove_file(&socket_path).await;

        let listener = UnixListener::bind(&socket_path)?;

        // TODO need to properly keep track of this for shutdown
        let mut process = Command::new(EXECUTABLE)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env(DATA_PATH_ENV_VAR, data_path)
            .spawn()?;

        proxy_logs(&mut process, &id);

        let (stream, _addr) = listener.accept().await?;

        let (reader, writer) = stream.into_split();
        let writer = FramedWrite::new(writer, LengthDelimitedJSONCodec::new());
        let mut reader = FramedRead::new(reader, LengthDelimitedJSONCodec::new());

        match reader.next().await {
            Some(Ok(ExtensionToIgloo::WhatsUpIgloo)) => {
                println!("{id} initialized!");
            }
            Some(Ok(msg)) => {
                panic!("{id} didn't init. Sent '{msg:?}' instead.")
            }
            Some(Err(e)) => {
                panic!("Failed to read {id}s init message: {e}")
            }
            None => {
                // FIXME return error
                panic!("{id} immediately closed the socket!")
            }
        };

        let (ext_tx, ext_rx) = kanal::bounded(20);

        Ok((
            ExtensionHandle {
                id,
                index: ExtensionIndex(usize::MAX),
                core_tx: to_core_tx.to_async(),
                ext_rx: ext_rx.to_async(),
                writer,
                reader,
                process,
            },
            ext_tx,
        ))
    }

    pub fn kill(mut self) -> io::Result<()> {
        self.process.start_kill()
        // TODO are log_tasks auto killed/ended?
    }

    pub fn spawn(self) -> Arc<ExtensionProcess> {
        let process = Arc::new(ExtensionProcess {
            id: self.id.clone(),
            index: self.index,
            process: RwLock::new(self.process),
        });

        tokio::spawn(read_task(self.id, self.index, self.reader, self.core_tx));

        tokio::spawn(write_task(self.writer, self.ext_rx, process.clone()));

        process
    }
}

/// Proxies requests to Extension
async fn write_task(
    mut writer: IWriter,
    ext_rx: kanal::AsyncReceiver<ExtensionRequest>,
    process: Arc<ExtensionProcess>,
) {
    while let Ok(msg) = ext_rx.recv().await {
        use ExtensionRequest::*;
        // TODO auto restarting when program ends
        // TODO force kill (no restart) when backlogged
        let res = match msg {
            Msg(msg) => writer.send(msg).await,
            Flush => writer.flush().await,
        };

        if let Err(e) = res {
            // TODO FIXME
            eprintln!("Error from ext: {e}");
        }
    }

    _ = process.kill().await;
}

impl ExtensionProcess {
    pub async fn kill(&self) -> io::Result<()> {
        let mut proc = self.process.write().await;
        proc.kill().await?;
        _ = fs::remove_file(&socket_path(&self.id)).await;
        println!("{}/{} shutdown gracefully", self.id, self.index);
        Ok(())
        // TODO are other tasks shutdown (read, log proxy)?
    }

    pub fn start_kill(&self) -> io::Result<()> {
        let mut proc = self.process.blocking_write();
        proc.start_kill()?;
        println!("{}/{} shutdown", self.id, self.index);
        Ok(())
        // TODO are other tasks shutdown (read, log proxy)?
    }
}

/// Proxies requests to IglooCore
async fn read_task(
    id: ExtensionID,
    index: ExtensionIndex,
    mut reader: IReader,
    core_tx: kanal::AsyncSender<IglooRequest>,
) {
    println!("{} running as {}", id, index);

    while let Some(msg) = reader.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Error reading msg from {id}/{index}: {e}. Skipping..",);
                continue;
            }
        };

        let req = IglooRequest::Ext {
            sender: index,
            content: msg,
        };

        if let Err(e) = core_tx.send(req).await {
            eprintln!("{id}/{index} failed to message to core: {e}");
        }
    }
}

/// Proxies stdout and stderr to this process prefixed with Extension's name
fn proxy_logs(child: &mut Child, eid: &ExtensionID) -> Vec<JoinHandle<()>> {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let mut tasks = Vec::with_capacity(4);

    if let Some(stdout) = stdout {
        let eid_1 = eid.clone();
        tasks.push(tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                println!("[{eid_1}] {line}");
            }
        }));
    }

    if let Some(stderr) = stderr {
        let eid_1 = eid.clone();
        tasks.push(tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("[{eid_1}] {line}");
            }
        }));
    }

    tasks
}
