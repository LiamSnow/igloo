use igloo_interface::{ComponentUpdate, Device, FloeCommand, IglooCommand, InitPayload};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::glacier::state::FloeState;

use super::state::GlacierState;

#[derive(Error, Debug)]
pub enum GlacierError {
    #[error("Failed to spawn floe: {0}")]
    SpawnError(String),
    #[error("IO error: {0}")]
    Io(#[from] tokio::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
enum GlacierStateUpdate {
    AddDevice {
        floe_name: String,
        id: Uuid,
        device: Device,
    },
    ComponentUpdates {
        floe_name: String,
        updates: Vec<ComponentUpdate>,
    },
}

struct FloeHandle {
    name: String,
    process: Child,
    stdin: ChildStdin,
    reader_task: JoinHandle<()>,
}

impl FloeHandle {
    async fn send_command(&mut self, command: &IglooCommand) -> Result<(), GlacierError> {
        let json = serde_json::to_string(command)?;
        self.stdin.write_all(json.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn shutdown(mut self) {
        // soft shutdown
        drop(self.stdin);
        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), self.process.wait()).await;

        // force kill
        let _ = self.process.kill().await;

        self.reader_task.abort();
    }
}

pub struct Glacier {
    handles: Vec<FloeHandle>,
    update_processor: JoinHandle<()>,
    update_tx: mpsc::Sender<GlacierStateUpdate>,
}

impl Glacier {
    pub async fn new(floe_paths: Vec<&str>, state: GlacierState) -> Result<Self, GlacierError> {
        let (update_tx, update_rx) = mpsc::channel::<GlacierStateUpdate>(1000);

        let update_processor = tokio::spawn(Self::process_updates(update_rx, state));

        let mut handles = Vec::new();
        for path in floe_paths {
            let handle = Self::spawn_floe(path, update_tx.clone()).await?;
            handles.push(handle);
        }

        Ok(Glacier {
            handles,
            update_processor,
            update_tx,
        })
    }

    async fn spawn_floe(
        path: &str,
        update_tx: mpsc::Sender<GlacierStateUpdate>,
    ) -> Result<FloeHandle, GlacierError> {
        let floe_name = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        println!("[{}] Spawning floe from: {}", floe_name, path);

        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| GlacierError::SpawnError(format!("{}: {}", path, e)))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| GlacierError::SpawnError("Failed to get stdin".into()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| GlacierError::SpawnError("Failed to get stdout".into()))?;

        let reader_task =
            tokio::spawn(Self::read_floe_output(floe_name.clone(), stdout, update_tx));

        let mut handle = FloeHandle {
            name: floe_name.clone(),
            process: child,
            stdin,
            reader_task,
        };

        let init = IglooCommand::Init(InitPayload { config: None });
        handle.send_command(&init).await?;
        println!("[{}] Sent Init command", floe_name);

        Ok(handle)
    }

    async fn read_floe_output(
        floe_name: String,
        stdout: ChildStdout,
        tx: mpsc::Sender<GlacierStateUpdate>,
    ) {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF
                    println!("[{}] Floe disconnected", floe_name);
                    break;
                }
                Ok(_) => {
                    match serde_json::from_str::<FloeCommand>(&line) {
                        Ok(cmd) => {
                            let update = match cmd {
                                FloeCommand::AddDevice(id, device) => {
                                    GlacierStateUpdate::AddDevice {
                                        floe_name: floe_name.clone(),
                                        id,
                                        device,
                                    }
                                }
                                FloeCommand::ComponentUpdates(updates) => {
                                    GlacierStateUpdate::ComponentUpdates {
                                        floe_name: floe_name.clone(),
                                        updates,
                                    }
                                }
                                FloeCommand::Log(message) => {
                                    println!("[{floe_name}]: {message}");
                                    continue;
                                }
                                FloeCommand::CustomError(error) => {
                                    eprintln!("[{floe_name}]: ERROR {error}");
                                    continue;
                                }
                                FloeCommand::SaveConfig(_) => {
                                    // TODO
                                    continue;
                                }
                            };

                            if let Err(e) = tx.send(update).await {
                                eprintln!("[{floe_name}] Failed to send update: {e}");
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "[{floe_name}] Failed to parse FloeCommand: {e} - Line: {}",
                                line.trim()
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[{}] Error reading from floe: {}", floe_name, e);
                    break;
                }
            }
        }
    }

    async fn process_updates(mut rx: mpsc::Receiver<GlacierStateUpdate>, mut state: GlacierState) {
        println!("Update processor started");

        while let Some(update) = rx.recv().await {
            match update {
                GlacierStateUpdate::AddDevice {
                    floe_name,
                    id,
                    device,
                } => {
                    state
                        .floes
                        .entry(floe_name.clone())
                        .or_insert_with(|| FloeState {
                            devices: HashMap::new(),
                        });

                    if let Some(floe_state) = state.floes.get_mut(&floe_name) {
                        floe_state.devices.insert(id, device);
                    }

                    state.device_registry.insert(id, floe_name.clone());

                    println!("[{}] Added device: {}", floe_name, id);
                }
                GlacierStateUpdate::ComponentUpdates { floe_name, updates } => {
                    let floe_state = match state.floes.get_mut(&floe_name) {
                        Some(s) => s,
                        None => {
                            eprintln!("{floe_name} does not exist!");
                            continue;
                        }
                    };

                    for update in updates {
                        // FIXME this code hurts my eyes
                        match floe_state.devices.get_mut(&update.device) {
                            Some(dev) => match dev.entities.0.get_mut(&update.entity) {
                                Some(entity) => {
                                    entity.set(update.value);
                                }
                                None => {
                                    eprintln!(
                                        "Entity {} does not exist on device {} in Floe {floe_name}!",
                                        update.entity, update.device
                                    );
                                }
                            },
                            None => {
                                eprintln!(
                                    "Device {} does not exist in Floe {floe_name}!",
                                    update.device
                                );
                            }
                        }
                    }
                }
            }
        }

        println!("Update processor stopped");
    }

    pub async fn shutdown(self) {
        println!("Shutting down FloeManager...");

        drop(self.update_tx);

        for handle in self.handles {
            println!("[{}] Shutting down floe", handle.name);
            handle.shutdown().await;
        }

        let _ = self.update_processor.await;

        println!("FloeManager shutdown complete");
    }
}
