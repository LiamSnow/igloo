use std::{path::Path, process::Stdio};

use futures_util::StreamExt;
use igloo_interface::{
    END_TRANSACTION, FloeCodec, FloeReaderDefault, FloeWriter, FloeWriterDefault,
    MAX_SUPPORTED_COMPONENT, REGISTER_ENTITY, RegisterEntity, START_DEVICE_TRANSACTION,
    START_REGISTRATION_TRANSACTION, StartDeviceTransaction, StartRegistrationTransaction,
    WHATS_UP_IGLOO, WhatsUpIgloo,
};
use rustc_hash::FxHashSet;
use smallvec::SmallVec;
use tokio::{
    fs,
    io::BufWriter,
    net::UnixListener,
    process::{Child, Command},
    sync::mpsc,
};
use tokio_util::codec::FramedRead;

use crate::glacier::{DeviceInfo, GlobalDeviceID, entity::Entity};

struct FloeManager {
    name: String,
    reg_dev_tx: mpsc::Sender<(GlobalDeviceID, DeviceInfo)>,
    writer: FloeWriterDefault,
    reader: FloeReaderDefault,
    state: LocalState,
}

#[derive(Debug, Default)]
struct LocalState {
    taken_ids: FxHashSet<String>,
    devices: Vec<Device>,
}

#[derive(Debug, Default)]
struct Device {
    entities: SmallVec<[Entity; 16]>,
    // TODO presence: BitSet,
}

// TODO remove unwraps and panics
pub async fn spawn(
    name: String,
    reg_dev_tx: mpsc::Sender<(GlobalDeviceID, DeviceInfo)>,
) -> Result<(), std::io::Error> {
    println!("Spawning Floe '{name}'");

    let cwd = format!("./floes/{name}");

    let data_path = format!("{cwd}/data");
    fs::create_dir_all(&data_path).await?;

    let socket_path = format!("{cwd}/floe.sock");
    let _ = fs::remove_file(&socket_path).await;
    let listener = UnixListener::bind(&socket_path)?;

    let mut process = Command::new(Path::new("./floe"))
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    println!("Spawned");

    proxy_stdio(&mut process, name.to_string());

    let (stream, _) = listener.accept().await?;
    let (reader, writer) = stream.into_split();
    let writer = FloeWriter(BufWriter::new(writer));
    let reader = FramedRead::new(reader, FloeCodec::new());

    tokio::spawn(async move {
        let man = FloeManager {
            name,
            reg_dev_tx,
            writer,
            reader,
            state: LocalState::default(),
        };
        man.run().await;
    });

    Ok(())
}

impl FloeManager {
    async fn run(mut self) {
        let max_supported_component = match self.reader.next().await {
            Some(Ok((WHATS_UP_IGLOO, payload))) => {
                let res: WhatsUpIgloo = borsh::from_slice(&payload).unwrap();

                if res.max_supported_component > MAX_SUPPORTED_COMPONENT {
                    panic!(
                        "Floe '{}' has a newer protocol than Igloo. Please upgrade Igloo",
                        self.name
                    )
                }

                println!("Floe '{}' initialized!!!", self.name);
                res.max_supported_component
            }
            Some(Ok((cmd_id, _))) => {
                panic!("Floe '{}' didn't init. Sent {cmd_id} instead.", self.name)
            }
            Some(Err(e)) => {
                panic!("Failed to read Floe '{}'s init message: {e}", self.name)
            }
            None => {
                panic!("Floe '{}' immediately closed the socket!", self.name)
            }
        };

        while let Some(res) = self.reader.next().await {
            let (cmd_id, payload) = match res {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error reading frame from Floe '{}': {e}", self.name);
                    continue;
                }
            };

            match cmd_id {
                START_REGISTRATION_TRANSACTION => {
                    let params: StartRegistrationTransaction = borsh::from_slice(&payload).unwrap();
                    self.handle_registration_transaction(params).await;
                }

                START_DEVICE_TRANSACTION => {
                    let params: StartDeviceTransaction = borsh::from_slice(&payload).unwrap();
                    self.handle_device_transaction(params).await;
                }

                cmd_id => {
                    eprintln!("Unexpected command {cmd_id} from Floe '{}'", self.name);
                }
            }
        }
    }

    async fn handle_registration_transaction(&mut self, params: StartRegistrationTransaction) {
        if params.device_idx as usize != self.state.devices.len() {
            panic!(
                "Floe '{}' malformed. Tried to register new device under idx={} but should have been {}",
                self.name,
                params.device_idx,
                self.state.devices.len()
            );
        }

        let mut device = Device::default();
        let mut entity_names = Vec::new();

        while let Some(res) = self.reader.next().await {
            let (cmd_id, payload) = match res {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error reading frame from Floe '{}': {e}", self.name);
                    continue;
                }
            };

            match cmd_id {
                REGISTER_ENTITY => {
                    let params: RegisterEntity = borsh::from_slice(&payload).unwrap();

                    if params.entity_idx as usize != device.entities.len() {
                        panic!(
                            "Floe '{}' malformed when registering device idx={}. Tried to register entity under idx={} but should have been {}",
                            self.name,
                            self.state.devices.len(),
                            params.entity_idx,
                            device.entities.len(),
                        );
                    }

                    entity_names.push(params.entity_name);
                    device.entities.push(Entity::default());
                }

                END_TRANSACTION => {
                    break;
                }

                _ => {}
            }
        }

        self.state.devices.push(device);

        let global_id = (self.name.clone(), params.device_id);
        let info = DeviceInfo {
            name: params.initial_name,
            entity_names,
        };

        self.reg_dev_tx.send((global_id, info)).await.unwrap();
    }

    async fn handle_device_transaction(&mut self, params: StartDeviceTransaction) {
        while let Some(res) = self.reader.next().await {
            let (cmd_id, payload) = match res {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error reading frame from Floe '{}': {e}", self.name);
                    continue;
                }
            };

            match cmd_id {
                END_TRANSACTION => {
                    return;
                }

                _ => {}
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
