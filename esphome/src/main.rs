use crate::device::{ConnectionParams, Device};
use futures_util::StreamExt;
use igloo_interface::ipc::{self, IglooMessage};
use ini::Ini;
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::{collections::HashMap, error::Error, sync::Arc};
use tokio::{fs, sync::Mutex};

pub mod connection;
pub mod device;
pub mod entity;
pub mod api {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}
pub mod model {
    include!(concat!(env!("OUT_DIR"), "/model.rs"));
}

pub const CONFIG_FILE: &str = "./data/config.ini";

/// Eventually this will be described in the Igloo.toml file
pub const ADD_DEVICE: u16 = 32;

#[derive(Debug, Default, Clone)]
pub struct Config {
    /// maps Persisnt Igloo Device ID -> Connection Params
    devices: HashMap<u64, ConnectionParams>,
}

pub type CommandAndPayload = (u16, Vec<u8>);

#[tokio::main]
async fn main() {
    let mut config = Config::load().await.unwrap();

    let (mut writer, mut reader) = ipc::connect()
        .await
        .expect("Failed to initialize Extension");

    // writer task
    let (write_tx, write_rx) = kanal::bounded_async(100);
    tokio::spawn(async move {
        loop {
            let msg = match write_rx.recv().await {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("Error reading from write_rx: {e}");
                    break;
                }
            };

            if let Err(e) = writer.write(&msg).await {
                eprintln!("Error writing message to Igloo: {e}");
            }
        }

        println!("Write task shutdown");
    });

    // Device ID -> Device Channel
    let mut device_txs = HashMap::with_capacity_and_hasher(20, FxBuildHasher);

    // connect to devices in config
    for (device_id, params) in config.devices.clone() {
        let (device_tx, deivce_rx) = kanal::bounded_async(50);
        device_txs.insert(device_id, device_tx);
        let mut device = Device::new(device_id, params);
        let write_tx_1 = write_tx.clone();
        tokio::spawn(async move {
            let did = device.id;
            if let Err(e) = device.connect().await {
                eprintln!("Error connecting to device ID={did}: {e}");
                return;
            }
            println!("Device ID={did} connected.");
            if let Err(e) = device.run(write_tx_1, deivce_rx).await {
                eprintln!("Error running device ID={did}: {e}");
            }
        });
    }

    let pending_creation: Arc<Mutex<FxHashMap<String, Device>>> = Arc::new(Mutex::new(
        HashMap::with_capacity_and_hasher(5, FxBuildHasher),
    ));

    while let Some(res) = reader.next().await {
        let msg = match res {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error reading message, skipping: {e}");
                continue;
            }
        };

        use IglooMessage::*;
        match msg {
            DeviceCreated(name, did) => {
                // pull out pending device
                let mut pc = pending_creation.lock().await;
                let Some(mut device) = pc.remove(&name) else {
                    eprintln!("Igloo sent DeviceCreated for unknown device. Skipping..");
                    continue;
                };
                drop(pc);

                // save to disk
                config.devices.insert(did, device.params.clone());
                config.save().await.unwrap();

                // give actual ID now
                device.id = did;

                // run
                let (device_tx, device_rx) = kanal::bounded_async(50);
                device_txs.insert(did, device_tx);
                let write_tx_1 = write_tx.clone();
                tokio::spawn(async move {
                    let res = device.run(write_tx_1, device_rx).await;
                    if let Err(e) = res {
                        eprintln!("Device {did} crashed: {e}");
                    }
                });
            }

            WriteComponents {
                device: did,
                entity,
                comps,
            } => {
                let Some(device) = device_txs.get(&did) else {
                    eprintln!("Igloo sent write for unknown device '{did}'. Skipping..");
                    continue;
                };

                if let Err(e) = device.send((entity, comps)).await {
                    eprintln!("Error sending message to device '{did}': {e}");
                }
            }

            Custom { name, params } => {
                if name != "Add Device" {
                    eprintln!("Unknown custom command '{name}'. Skipping..");
                    continue;
                }

                let Some(ip) = params.get("ip") else {
                    eprintln!(
                        "Parameter 'ip' must be specified for custom command 'Add Device'. Skipping.."
                    );
                    continue;
                };

                let params = ConnectionParams {
                    ip: ip.clone(),
                    noise_psk: params.get("noise_psk").cloned(),
                    password: params.get("password").cloned(),
                    name: params.get("name").cloned(),
                };

                let mut device = Device::new(0, params);
                let write_tx_1 = write_tx.clone();
                let pending_creation_1 = pending_creation.clone();
                tokio::spawn(async move {
                    let info = device.connect().await.unwrap();
                    write_tx_1
                        .send(IglooMessage::CreateDevice(info.name.clone()))
                        .await
                        .unwrap();
                    let mut pc = pending_creation_1.lock().await;
                    pc.insert(info.name, device);
                    drop(pc);
                });
            }

            WhatsUpIgloo { .. } | CreateDevice(..) | RegisterEntity { .. } => {
                eprintln!("Igloo unexpectedly sent client message. Skipping..");
            }
        }
    }
}

impl Config {
    async fn load() -> Result<Self, Box<dyn Error>> {
        let mut me = Self::default();

        let content = fs::read_to_string(CONFIG_FILE).await?;
        let ini = Ini::load_from_str(&content)?;

        for did_str in ini.sections() {
            let Some(did_str) = did_str else { continue };
            let did: u64 = did_str.parse()?;
            let section = ini.section(Some(did_str)).unwrap();

            me.devices.insert(
                did,
                ConnectionParams {
                    ip: section.get("ip").ok_or("Mising 'ip'")?.to_string(),
                    name: section.get("name").map(|o| o.to_string()),
                    noise_psk: section.get("noise_psk").map(|o| o.to_string()),
                    password: section.get("password").map(|o| o.to_string()),
                },
            );
        }

        Ok(me)
    }

    async fn save(&self) -> Result<(), Box<dyn Error>> {
        let mut ini = Ini::new();

        for (id, params) in &self.devices {
            let mut section = ini.with_section(Some(id.to_string()));
            section.set("ip", &params.ip);
            if let Some(name) = &params.name {
                section.set("name", name);
            }
            if let Some(noise_psk) = &params.noise_psk {
                section.set("noise_psk", noise_psk);
            }
            if let Some(password) = &params.password {
                section.set("password", password);
            }
        }

        let mut buf = Vec::new();
        ini.write_to(&mut buf)?;
        fs::write(CONFIG_FILE, buf).await?;

        Ok(())
    }
}
