use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::{FloeCommand, FloeResponse, IglooCommand, IglooResponse};

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("IO error")]
    Io(#[from] tokio::io::Error),
    #[error("JSON serialization error")]
    Json(#[from] serde_json::Error),
    #[error("Request timeout")]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error("Request was canceled")]
    RequestCanceled,
    #[error("Channel send failed")]
    ChannelSend,
}

pub type ProtocolResult<T> = Result<T, ProtocolError>;

#[derive(Serialize, Deserialize)]
struct Message {
    id: Option<Uuid>,
    command: Option<FloeCommand>,
    igloo_command: Option<IglooCommand>,
    response: Option<IglooResponse>,
    floe_response: Option<FloeResponse>,
}

struct OutgoingCommand {
    command: FloeCommand,
    response_sender: Option<oneshot::Sender<IglooResponse>>,
}

pub struct FloeInterfaceManager {
    command_sender: mpsc::UnboundedSender<OutgoingCommand>,
    _task_handle: JoinHandle<()>,
}

impl FloeInterfaceManager {
    pub fn new<F, Fut>(handler: F) -> Self
    where
        F: Fn(IglooCommand) -> Fut + Send + 'static,
        Fut: Future<Output = FloeResponse> + Send + 'static,
    {
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<OutgoingCommand>();
        let pending: Arc<Mutex<HashMap<Uuid, oneshot::Sender<IglooResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let pending_clone = pending.clone();

        let task_handle = tokio::spawn(async move {
            let mut stdin = BufReader::new(tokio::io::stdin());
            let mut stdout = tokio::io::stdout();

            loop {
                tokio::select! {
                    // outgoing commands
                    Some(outgoing) = cmd_rx.recv() => {
                        let id = if outgoing.response_sender.is_some() {
                            let id = Uuid::now_v7();
                            pending_clone.lock().await.insert(id, outgoing.response_sender.unwrap());
                            Some(id)
                        } else {
                            None
                        };

                        let msg = Message {
                            id,
                            command: Some(outgoing.command),
                            igloo_command: None,
                            response: None,
                            floe_response: None,
                        };

                        if let Err(e) = Self::write_message(&mut stdout, &msg).await {
                            panic!("Failed to write outgoing command: {}", e);
                        }
                    }

                    result = Self::read_message(&mut stdin) => {
                        match result {
                            Ok(msg) => {
                                // responses to outgoing commands
                                if let (Some(id), Some(response)) = (msg.id, msg.response) {
                                    if let Some(sender) = pending_clone.lock().await.remove(&id) {
                                        let _ = sender.send(response);
                                    }
                                    continue;
                                }

                                // incoming commands from Igloo
                                if let Some(command) = msg.igloo_command {
                                    let response = handler(command).await;

                                    let response_msg = Message {
                                        id: msg.id,
                                        command: None,
                                        igloo_command: None,
                                        response: None,
                                        floe_response: Some(response),
                                    };

                                    if let Err(e) = Self::write_message(&mut stdout, &response_msg).await {
                                        panic!("Failed to write response: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                panic!("Failed to read message: {}", e);
                            }
                        }
                    }
                }
            }
        });

        Self {
            command_sender: cmd_tx,
            _task_handle: task_handle,
        }
    }

    async fn read_message(stdin: &mut BufReader<tokio::io::Stdin>) -> ProtocolResult<Message> {
        let mut line = String::new();
        stdin.read_line(&mut line).await?;
        Ok(serde_json::from_str(line.trim())?)
    }

    async fn write_message(stdout: &mut tokio::io::Stdout, msg: &Message) -> ProtocolResult<()> {
        let json = serde_json::to_string(msg)?;
        stdout.write_all(json.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
        Ok(())
    }

    pub async fn send_command(&self, command: FloeCommand) -> ProtocolResult<IglooResponse> {
        let (tx, rx) = oneshot::channel();

        let outgoing = OutgoingCommand {
            command,
            response_sender: Some(tx),
        };

        self.command_sender
            .send(outgoing)
            .map_err(|_| ProtocolError::ChannelSend)?;

        match rx.await {
            Ok(response) => Ok(response),
            Err(_) => Err(ProtocolError::RequestCanceled),
        }
    }

    pub async fn send_command_timeout(
        &self,
        command: FloeCommand,
        timeout: std::time::Duration,
    ) -> ProtocolResult<IglooResponse> {
        tokio::time::timeout(timeout, self.send_command(command)).await?
    }

    pub async fn log(&self, message: String) -> ProtocolResult<()> {
        let outgoing = OutgoingCommand {
            command: FloeCommand::Log(message),
            response_sender: None,
        };

        self.command_sender
            .send(outgoing)
            .map_err(|_| ProtocolError::ChannelSend)?;
        Ok(())
    }
}
