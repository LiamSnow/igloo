use igloo_interface::{FloeCommand, FloeResponse, IglooCommand, IglooResponse, IglooResponseError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock, mpsc, oneshot};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::glacier::GlacierState;

#[derive(Error, Debug)]
pub enum FloeManagerError {
    #[error("IO error")]
    Io(#[from] tokio::io::Error),
    #[error("JSON serialization error")]
    Json(#[from] serde_json::Error),
    #[error("Failed to spawn process after {attempts} attempts")]
    SpawnFailed { attempts: u32 },
    #[error("Channel send failed")]
    ChannelSend,
    #[error("Process terminated unexpectedly")]
    ProcessTerminated,
}

pub type FloeResult<T> = Result<T, FloeManagerError>;

#[derive(Serialize, Deserialize)]
pub struct Message {
    id: Option<Uuid>,
    command: Option<FloeCommand>,
    igloo_command: Option<IglooCommand>,
    response: Option<IglooResponse>,
    floe_response: Option<FloeResponse>,
}

pub struct OutgoingIglooCommand {
    command: IglooCommand,
    response_sender: Option<oneshot::Sender<FloeResponse>>,
}

pub struct FloeManager {
    state: Arc<RwLock<GlacierState>>,
    provider_name: String,
    command_sender: mpsc::UnboundedSender<OutgoingIglooCommand>,
    _task_handle: JoinHandle<()>,
    _child: Child,
}

impl FloeManager {
    pub async fn new(binary_path: &str, state: Arc<RwLock<GlacierState>>) -> FloeResult<Self> {
        let provider_name = std::path::Path::new(binary_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut child = Self::spawn_with_retry(binary_path, 3).await?;

        let mut child_stdin =
            child
                .stdin
                .take()
                .ok_or(FloeManagerError::Io(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "Failed to get child stdin",
                )))?;

        let child_stdout = child
            .stdout
            .take()
            .ok_or(FloeManagerError::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Failed to get child stdout",
            )))?;

        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<OutgoingIglooCommand>();
        let pending: Arc<Mutex<HashMap<Uuid, oneshot::Sender<FloeResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let pending_clone = pending.clone();
        let state_clone = state.clone();
        let provider_name_clone = provider_name.clone();

        let task_handle = tokio::spawn(async move {
            let mut stdout_reader = BufReader::new(child_stdout);

            loop {
                tokio::select! {
                    // -> Floe
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
                            command: None,
                            igloo_command: Some(outgoing.command),
                            response: None,
                            floe_response: None,
                        };

                        if let Err(e) = Self::write_message(&mut child_stdin, &msg).await {
                            panic!("Failed to write command to floe: {}", e);
                        }
                    }

                    // <- Floe
                    result = Self::read_message(&mut stdout_reader) => {
                        match result {
                            Ok(msg) => {
                                // their responses
                                if let (Some(id), Some(response)) = (msg.id, msg.floe_response) {
                                    if let Some(sender) = pending_clone.lock().await.remove(&id) {
                                        let _ = sender.send(response);
                                    }
                                    continue;
                                }

                                // incoming commands
                                if let Some(command) = msg.command {
                                    let response = Self::handle_floe_command(
                                        command,
                                        &state_clone,
                                        &provider_name_clone
                                    ).await;

                                    let response_msg = Message {
                                        id: msg.id,
                                        command: None,
                                        igloo_command: None,
                                        response: Some(response),
                                        floe_response: None,
                                    };

                                    if let Err(e) = Self::write_message(&mut child_stdin, &response_msg).await {
                                        panic!("Failed to write response to floe: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                panic!("Failed to read message from floe: {}", e);
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            state,
            provider_name,
            command_sender: cmd_tx,
            _task_handle: task_handle,
            _child: child,
        })
    }

    async fn spawn_with_retry(binary_path: &str, max_attempts: u32) -> FloeResult<Child> {
        for attempt in 1..=max_attempts {
            match Command::new(binary_path)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::inherit())
                .spawn()
            {
                Ok(child) => {
                    println!("Successfully spawned floe: {}", binary_path);
                    return Ok(child);
                }
                Err(e) => {
                    eprintln!(
                        "Attempt {}/{} failed to spawn {}: {}",
                        attempt, max_attempts, binary_path, e
                    );
                    if attempt == max_attempts {
                        return Err(FloeManagerError::SpawnFailed {
                            attempts: max_attempts,
                        });
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
        unreachable!()
    }

    async fn handle_floe_command(
        command: FloeCommand,
        state: &Arc<RwLock<GlacierState>>,
        provider_name: &str,
    ) -> IglooResponse {
        match command {
            FloeCommand::AddDevice(uuid, device) => {
                let mut state_guard = state.write().await;

                state_guard.devices.insert(uuid, device);
                state_guard
                    .providers
                    .insert(uuid, provider_name.to_string());

                Ok(())
            }
            FloeCommand::Update(update) => {
                let state_guard = state.read().await;

                if state_guard.devices.contains_key(&update.device) {
                    println!(
                        "Update for device {}: {} = {:?}",
                        update.device, update.entity, update.value
                    );
                    // TODO: actually update state
                    Ok(())
                } else {
                    Err(IglooResponseError::InvalidDevice(update.device))
                }
            }
            FloeCommand::SaveConfig(config) => {
                println!("Provider {} saved config: {}", provider_name, config);
                // TODO: save config for them
                Ok(())
            }
            FloeCommand::Log(message) => {
                println!("[{}] {}", provider_name, message);
                Ok(())
            }
        }
    }

    async fn read_message(
        reader: &mut BufReader<tokio::process::ChildStdout>,
    ) -> FloeResult<Message> {
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        if line.trim().is_empty() {
            return Err(FloeManagerError::ProcessTerminated);
        }
        Ok(serde_json::from_str(line.trim())?)
    }

    async fn write_message(
        writer: &mut tokio::process::ChildStdin,
        msg: &Message,
    ) -> FloeResult<()> {
        let json = serde_json::to_string(msg)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn send_command(&self, command: IglooCommand) -> FloeResult<FloeResponse> {
        let (tx, rx) = oneshot::channel();

        let outgoing = OutgoingIglooCommand {
            command,
            response_sender: Some(tx),
        };

        self.command_sender
            .send(outgoing)
            .map_err(|_| FloeManagerError::ChannelSend)?;

        match rx.await {
            Ok(response) => Ok(response),
            Err(_) => Err(FloeManagerError::ChannelSend),
        }
    }

    pub async fn send_command_timeout(
        &self,
        command: IglooCommand,
        timeout: Duration,
    ) -> FloeResult<FloeResponse> {
        tokio::time::timeout(timeout, self.send_command(command))
            .await
            .map_err(|_| {
                FloeManagerError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Command timed out",
                ))
            })?
    }

    pub async fn ping(&self) -> FloeResult<FloeResponse> {
        self.send_command(IglooCommand::Ping).await
    }
}
