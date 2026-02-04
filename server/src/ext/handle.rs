use super::EXTS_DIR;
use crate::{
    DATA_DIR, PACKAGES_DIR,
    core::{IglooError, IglooRequest},
};
use futures_util::StreamExt;
use igloo_interface::{
    MSIC,
    id::{ExtensionID, ExtensionIndex},
    ipc::{DATA_PATH_ENV_VAR, IReader, IWriter, IglooMessage, MSIM},
};
use std::{
    path::{self, Path},
    process::Stdio,
};
use tokio::{
    fs,
    net::UnixListener,
    process::{self, Child, Command},
    task::JoinHandle,
};

pub struct ExtensionHandle {
    pub id: ExtensionID,
    pub index: ExtensionIndex,
    pub tx: kanal::AsyncSender<IglooRequest>,
    pub reader: IReader,
    pub msic: u16,
    pub msim: u8,
}

pub const SOCKET: &str = "igloo.sock";
pub const EXECUTABLE: &str = "./ext";

impl ExtensionHandle {
    // TODO remove unwraps and panics
    pub async fn new(
        id: ExtensionID,
        tx: &kanal::Sender<IglooRequest>,
    ) -> Result<(Self, IWriter), IglooError> {
        println!("Initializing Extension {id}");

        let mut cwd = PACKAGES_DIR.get().unwrap().clone();
        cwd.push(EXTS_DIR);
        cwd.push(id.0.to_string());

        let mut data_path = DATA_DIR.get().unwrap().clone();
        data_path.push(EXTS_DIR);
        data_path.push(id.0.to_string());
        data_path = path::absolute(&data_path)?;

        let mut socket_path = PACKAGES_DIR.get().unwrap().clone();
        socket_path.push(EXTS_DIR);
        socket_path.push(id.0.to_string());
        socket_path.push(SOCKET);

        fs::create_dir_all(&data_path).await?;
        let _ = fs::remove_file(&socket_path).await;

        let listener = UnixListener::bind(&socket_path)?;

        // TODO need to properly keep track of this for shutdown
        let mut process = Command::new(EXECUTABLE)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env(DATA_PATH_ENV_VAR, data_path)
            .spawn()?;

        proxy_stdio(&mut process, &id);

        let (stream, _addr) = listener.accept().await?;
        let (reader, writer) = stream.into_split();
        let writer = IWriter::new(writer);
        let mut reader = IReader::new(reader);

        let (msic, msim) = match reader.next().await {
            Some(Ok(IglooMessage::WhatsUpIgloo { msic, msim })) => {
                if msic > MSIC || msim > MSIM {
                    panic!(
                        "{id} has a newer protocol than Igloo. Please upgrade Igloo. Igloo has MSIC={MSIC}, MSIM={MSIM} and {id} has msic={msic}, msim={msim}"
                    );
                }

                println!("{id} initialized!");
                (msic, msim)
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

        Ok((
            ExtensionHandle {
                id,
                index: ExtensionIndex(usize::MAX),
                tx: tx.clone_async(),
                reader,
                msic,
                msim,
            },
            writer,
        ))
    }

    pub fn spawn(self) -> JoinHandle<()> {
        tokio::spawn(async move { self.run().await })
    }

    /// just forward transactions up to to Glacier
    pub async fn run(mut self) {
        println!("{} running as {}", self.id, self.index);

        while let Some(msg) = self.reader.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!(
                        "Error reading msg from {}/{}: {e}. Skipping..",
                        self.id, self.index
                    );
                    continue;
                }
            };

            let req = IglooRequest::Ext {
                sender: self.index,
                content: msg,
            };

            if let Err(e) = self.tx.send(req).await {
                eprintln!("{}/{} failed to message to core: {e}", self.id, self.index);
            }
        }

        println!("{}/{} shutdown", self.id, self.index);
    }
}

/// Proxies stdout and stderr to this process prefixed with Extension's name
fn proxy_stdio(child: &mut Child, eid: &ExtensionID) {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    if let Some(stdout) = stdout {
        let eid_1 = eid.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                println!("[{eid_1}] {line}");
            }
        });
    }

    if let Some(stderr) = stderr {
        let eid_1 = eid.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("[{eid_1}] {line}");
            }
        });
    }
}
