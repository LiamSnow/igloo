use crate::core::{IglooError, IglooRequest};
use futures_util::StreamExt;
use igloo_interface::{
    MSIC,
    id::{FloeID, FloeRef},
    ipc::{IReader, IWriter, IglooMessage, MSIM},
};
use std::{path::Path, process::Stdio};
use tokio::{
    fs,
    net::UnixListener,
    process::{self, Child},
    task::JoinHandle,
};

pub struct FloeHandle {
    pub id: FloeID,
    pub fref: FloeRef,
    pub tx: kanal::AsyncSender<IglooRequest>,
    pub reader: IReader,
    pub msic: u16,
    pub msim: u8,
}

impl FloeHandle {
    // TODO remove unwraps and panics
    pub async fn new(
        id: FloeID,
        tx: &kanal::Sender<IglooRequest>,
    ) -> Result<(Self, IWriter), IglooError> {
        println!("Initializing Floe {id}");

        let cwd = format!("./floes/{}", id.0);

        let data_path = format!("{cwd}/data");
        fs::create_dir_all(&data_path).await?;

        let socket_path = format!("{cwd}/igloo.sock");
        let _ = fs::remove_file(&socket_path).await;
        let listener = UnixListener::bind(&socket_path)?;

        // TODO need to properly keep track of this for shutdown
        let mut process = process::Command::new(Path::new("./floe"))
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
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
            FloeHandle {
                id,
                fref: FloeRef(usize::MAX),
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
        println!("{} running as {}", self.id, self.fref);

        while let Some(msg) = self.reader.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!(
                        "Error reading msg from {}/{}: {e}. Skipping..",
                        self.id, self.fref
                    );
                    continue;
                }
            };

            let req = IglooRequest::HandleMessage {
                sender: self.fref,
                content: msg,
            };

            if let Err(e) = self.tx.send(req).await {
                eprintln!("{}/{} failed to message to core: {e}", self.id, self.fref);
            }
        }

        println!("{}/{} shutdown", self.id, self.fref);
    }
}

/// Proxies stdout and stderr to this process prefixed with Floe's name
fn proxy_stdio(child: &mut Child, fid: &FloeID) {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    if let Some(stdout) = stdout {
        let fid_1 = fid.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                println!("[{fid_1}] {line}");
            }
        });
    }

    if let Some(stderr) = stderr {
        let fid_1 = fid.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("[{fid_1}] {line}");
            }
        });
    }
}
