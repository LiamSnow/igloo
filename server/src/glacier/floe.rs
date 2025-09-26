use igloo_interface::{FloeCommand, IglooCommand, InitPayload};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::glacier::GlacierError;
use crate::glacier::tree::DeviceTreeUpdate;

pub struct FloeHandle {
    pub name: String,
    pub process: Child,
    pub stdin: ChildStdin,
    pub reader_task: JoinHandle<()>,
}

// TODO remove
#[derive(Deserialize, Serialize, Clone)]
pub struct ConnectionParams {
    pub ip: String,
    pub noise_psk: Option<String>,
    pub password: Option<String>,
}

impl FloeHandle {
    pub async fn new(
        name: &str,
        update_tx: mpsc::Sender<DeviceTreeUpdate>,
    ) -> Result<Self, GlacierError> {
        println!("Spawning Floe: {name}");

        // FIXME should be reading binary name from `Floe.toml`

        let path = format!("./floes/{name}/floe");

        let mut child = Command::new(Path::new(&path))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| GlacierError::SpawnError(format!("{path}: {e}")))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| GlacierError::SpawnError("Failed to get stdin".into()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| GlacierError::SpawnError("Failed to get stdout".into()))?;

        let reader_task = tokio::spawn(reader_task(name.to_string(), stdout, update_tx));

        let mut handle = FloeHandle {
            name: name.to_string(),
            process: child,
            stdin,
            reader_task,
        };

        let config = fs::read_to_string(format!("./floes/{name}/config.txt"))
            .await
            .ok();
        let init = IglooCommand::Init(InitPayload { config });
        handle.send_command(&init).await?;
        println!("Sent init command to Floe {name}");

        // TODO remove
        if name == "ESPHome" {
            handle
                .send_command(&IglooCommand::Custom(
                    "add_device".to_string(),
                    serde_json::to_string(&ConnectionParams {
                        ip: "192.168.1.18:6053".to_string(),
                        noise_psk: Some("GwsvILrvcN/BHAG9m7Hgzcqzc4Dx9neT/1RfEDmsecw=".to_string()),
                        password: None,
                    })
                    .unwrap(),
                ))
                .await?;
        }

        Ok(handle)
    }

    pub async fn send_command(&mut self, command: &IglooCommand) -> Result<(), GlacierError> {
        let json = serde_json::to_string(command)?;
        self.stdin.write_all(json.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    pub async fn shutdown(mut self) {
        // soft shutdown
        drop(self.stdin);
        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), self.process.wait()).await;

        // force kill
        let _ = self.process.kill().await;

        self.reader_task.abort();
    }
}

/// reads commands from Floe and forwards to `tx`
async fn reader_task(floe_name: String, stdout: ChildStdout, tx: mpsc::Sender<DeviceTreeUpdate>) {
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF
                println!("Floe {floe_name} disconnected");
                break;
            }
            Ok(_) => match serde_json::from_str::<FloeCommand>(&line) {
                Ok(cmd) => {
                    if !handle_cmd(cmd, &floe_name, &tx).await {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Failed to parse FloeCommand from {floe_name}: {e} - Line: {}",
                        line.trim()
                    );
                }
            },
            Err(e) => {
                eprintln!("Error reading from Floe {floe_name}: {e}");
                break;
            }
        }
    }
}

async fn handle_cmd(
    cmd: FloeCommand,
    floe_name: &str,
    tx: &mpsc::Sender<DeviceTreeUpdate>,
) -> bool {
    let update = match cmd {
        FloeCommand::AddDevice(id, name, entities) => DeviceTreeUpdate::AddDevice {
            floe_name: floe_name.to_string(),
            id,
            name,
            entities,
        },
        FloeCommand::ComponentUpdates(updates) => DeviceTreeUpdate::ComponentUpdates {
            floe_name: floe_name.to_string(),
            updates,
        },
        FloeCommand::Log(message) => {
            println!("[{floe_name}]: {message}");
            return true;
        }
        FloeCommand::CustomError(error) => {
            eprintln!("[{floe_name}]: ERROR {error}");
            return true;
        }
        FloeCommand::SaveConfig(contents) => {
            // TODO return result instead of unwrap
            fs::write(format!("./floes/{floe_name}/config.txt"), contents)
                .await
                .unwrap();
            return true;
        }
    };

    if let Err(e) = tx.send(update).await {
        eprintln!("[{floe_name}] Failed to send update: {e}");
        return false;
    }

    true
}
