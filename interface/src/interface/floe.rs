use serde_json;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{ComponentUpdate, Entities, FloeCommand, IglooCommand, InitPayload};

#[derive(Error, Debug)]
pub enum IglooInterfaceError {
    #[error("IO error")]
    Io(#[from] tokio::io::Error),
    #[error("JSON serialization error")]
    Json(#[from] serde_json::Error),
    #[error("Channel send error")]
    ChannelSend(String),
    #[error("Channel receive error")]
    ChannelReceive(String),
    #[error("Init not received")]
    InitNotReceived,
}

#[trait_variant::make(FloeHandler: Send)]
pub trait LocalFloeHandler {
    /// Called once at boot
    async fn init(&mut self, init: InitPayload, manager: &IglooInterface);

    async fn updates_requested(&mut self, update: Vec<ComponentUpdate>, manager: &IglooInterface);

    async fn custom(&mut self, name: String, data: String, manager: &IglooInterface);
}

#[derive(Clone)]
pub struct IglooInterface {
    tx: mpsc::Sender<FloeCommand>,
}

impl IglooInterface {
    pub async fn run<H: FloeHandler>(mut handler: H) -> Result<(), IglooInterfaceError> {
        let (tx, mut rx) = mpsc::channel::<FloeCommand>(100);
        let manager = IglooInterface { tx };

        let writer_handle = tokio::spawn(async move {
            let mut stdout = tokio::io::stdout();
            while let Some(cmd) = rx.recv().await {
                let json = serde_json::to_string(&cmd)?;
                stdout.write_all(json.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
            Ok::<(), IglooInterfaceError>(())
        });

        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        // wait for init
        let init_payload = loop {
            line.clear();
            let bytes_read = reader
                .read_line(&mut line)
                .await
                .map_err(IglooInterfaceError::Io)?;

            if bytes_read == 0 {
                // EOF
                return Err(IglooInterfaceError::InitNotReceived);
            }

            let cmd: IglooCommand = serde_json::from_str(&line)?;
            if let IglooCommand::Init(init) = cmd {
                break init;
            }
        };

        handler.init(init_payload, &manager).await;

        loop {
            line.clear();
            let bytes_read = reader
                .read_line(&mut line)
                .await
                .map_err(IglooInterfaceError::Io)?;

            if bytes_read == 0 {
                // EOF
                break;
            }

            let cmd: IglooCommand = match serde_json::from_str(&line) {
                Ok(cmd) => cmd,
                Err(e) => {
                    let _ = manager.log(format!("Failed to parse command: {}", e)).await;
                    continue;
                }
            };

            match cmd {
                IglooCommand::Init(_) => {
                    // unexpected init
                    manager
                        .log("Received unexpected Init command".to_string())
                        .await?;
                }
                IglooCommand::ReqComponentUpdates(update) => {
                    handler.updates_requested(update, &manager).await
                }
                IglooCommand::Custom(name, data) => handler.custom(name, data, &manager).await,
            }
        }

        drop(manager);
        let _ = writer_handle.await;
        Ok(())
    }

    // send component updates under devices you registered
    pub async fn send_updates(
        &self,
        update: Vec<ComponentUpdate>,
    ) -> Result<(), IglooInterfaceError> {
        self.send_command(FloeCommand::ComponentUpdates(update))
            .await
    }

    /// registers a new device with Igloo
    pub async fn add_device(
        &self,
        id: Uuid,
        device_name: String,
        entities: Entities,
    ) -> Result<(), IglooInterfaceError> {
        self.send_command(FloeCommand::AddDevice(id, device_name, entities))
            .await
    }

    /// logs a message (use this over println!() which causes parsing errors)
    pub async fn log(&self, message: String) -> Result<(), IglooInterfaceError> {
        self.send_command(FloeCommand::Log(message)).await
    }

    /// save config to Igloo (you will then get back on init())
    /// realistically you would use serde_json to generate the string
    pub async fn save_config(&self, config: String) -> Result<(), IglooInterfaceError> {
        self.send_command(FloeCommand::SaveConfig(config)).await
    }

    /// send if there was an error with your custom command
    pub async fn send_custom_error(&self, error: String) -> Result<(), IglooInterfaceError> {
        self.send_command(FloeCommand::CustomError(error)).await
    }

    async fn send_command(&self, cmd: FloeCommand) -> Result<(), IglooInterfaceError> {
        self.tx
            .send(cmd)
            .await
            .map_err(|_| IglooInterfaceError::ChannelSend("Failed to send command".to_string()))
    }
}
