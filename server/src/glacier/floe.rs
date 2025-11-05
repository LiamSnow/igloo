use futures_util::StreamExt;
use igloo_interface::{
    CREATE_DEVICE, DESELECT_ENTITY, END_TRANSACTION, MAX_SUPPORTED_COMPONENT, START_TRANSACTION,
    WHATS_UP_IGLOO, WhatsUpIgloo,
    floe::{FloeCodec, FloeReaderDefault, FloeWriter, FloeWriterDefault},
    id::{FloeID, FloeRef},
};
use smallvec::smallvec;
use std::{error::Error, mem, path::Path, process::Stdio};
use tokio::{
    fs,
    io::BufWriter,
    net::UnixListener,
    process::{self, Child},
    sync::mpsc,
};
use tokio_util::codec::FramedRead;

use crate::glacier::{Command, Commands};

pub struct FloeManager {
    pub fid: FloeID,
    pub fref: FloeRef,
    pub cmds_tx: mpsc::Sender<(FloeRef, Commands)>,
    pub reader: FloeReaderDefault,
}

// TODO remove unwraps and panics
pub async fn init(
    name: String,
) -> Result<(FloeReaderDefault, FloeWriterDefault, u16), Box<dyn Error>> {
    println!("Initializing Floe '{name}'");

    let cwd = format!("./floes/{name}");

    let data_path = format!("{cwd}/data");
    fs::create_dir_all(&data_path).await?;

    let socket_path = format!("{cwd}/floe.sock");
    let _ = fs::remove_file(&socket_path).await;
    let listener = UnixListener::bind(&socket_path)?;

    let mut process = process::Command::new(Path::new("./floe"))
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    proxy_stdio(&mut process, name.to_string());

    let (stream, _) = listener.accept().await?;
    let (reader, writer) = stream.into_split();
    let writer = FloeWriter(BufWriter::new(writer));
    let mut reader = FramedRead::new(reader, FloeCodec::new());

    let max_supported_component = match reader.next().await {
        Some(Ok((WHATS_UP_IGLOO, payload))) => {
            let res: WhatsUpIgloo = borsh::from_slice(&payload).unwrap();

            if res.max_supported_component > MAX_SUPPORTED_COMPONENT {
                panic!("Floe '{name}' has a newer protocol than Igloo. Please upgrade Igloo",)
            }

            println!("Floe '{name}' initialized!!!");
            res.max_supported_component
        }
        Some(Ok((cmd_id, _))) => {
            panic!("Floe '{name}' didn't init. Sent {cmd_id} instead.")
        }
        Some(Err(e)) => {
            panic!("Failed to read Floe '{name}'s init message: {e}")
        }
        None => {
            panic!("Floe '{name}' immediately closed the socket!")
        }
    };

    Ok((reader, writer, max_supported_component))
}

impl FloeManager {
    /// just forward transactions up to to Glacier
    pub async fn run(mut self) {
        println!("Floe '{:?}' running as #{:?}", self.fid, self.fref);

        let mut cur_trans = Commands::new();

        while let Some(res) = self.reader.next().await {
            let (cmd_id, payload) = match res {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error reading frame from Floe '{:?}': {e}", self.fid);
                    continue;
                }
            };

            if cmd_id == CREATE_DEVICE {
                let res = self
                    .cmds_tx
                    .try_send((self.fref, smallvec![Command { cmd_id, payload }]));
                if let Err(e) = res {
                    eprintln!("Failed to send transaction to Glacier: {e}");
                }
            } else if (START_TRANSACTION..=DESELECT_ENTITY).contains(&cmd_id) || cmd_id >= 64 {
                cur_trans.push(Command { cmd_id, payload });

                if cmd_id == END_TRANSACTION {
                    let res = self
                        .cmds_tx
                        .try_send((self.fref, mem::take(&mut cur_trans)));
                    if let Err(e) = res {
                        eprintln!("Failed to send transaction to Glacier: {e}");
                    }
                }
            } else {
                eprintln!("Floe '{:?}' send unexpected command {cmd_id}", self.fid);
            }
        }
    }
}

/// Proxies stdout and stderr to this process prefixed with Floe's name
fn proxy_stdio(child: &mut Child, name: String) {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    if let Some(stdout) = stdout {
        let name_stdout = name.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                println!("[{name_stdout}] {line}");
            }
        });
    }

    if let Some(stderr) = stderr {
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("[{name}] {line}");
            }
        });
    }
}
