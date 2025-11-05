use crate::device::{ConnectionParams, Device};
use futures_util::StreamExt;
use igloo_interface::{
    CreateDevice, DEVICE_CREATED, DeviceCreated, END_TRANSACTION, START_TRANSACTION,
    StartTransaction, floe::floe_init,
};
use ini::Ini;
use std::{collections::HashMap, error::Error, sync::Arc};
use tokio::{
    fs,
    sync::{Mutex, mpsc},
};

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

/// Eventually this will be described in the Floe.toml file
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

    let (writer, mut reader) = floe_init().await.expect("Failed to initialize Floe");
    let shared_writer = Arc::new(Mutex::new(writer));

    let mut devices_tx = HashMap::new();

    // connect to devices in config
    for (device_id, params) in config.devices.clone() {
        let (stream_tx, stream_rx) = mpsc::channel(100);
        devices_tx.insert(device_id, stream_tx);
        let mut device = Device::new(device_id, params);
        let shared_writer_copy = shared_writer.clone();
        tokio::spawn(async move {
            let did = device.id;
            if let Err(e) = device.connect().await {
                eprintln!("Error connecting device ID={did}: {e}");
                return;
            }
            if let Err(e) = device.run(shared_writer_copy, stream_rx).await {
                eprintln!("Error running device ID={did}: {e}");
            }
        });
    }

    let pending_creation: Arc<Mutex<HashMap<String, Device>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let mut cur_device_tx = None;

    while let Some(res) = reader.next().await {
        let (cmd_id, payload) = match res {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Frame read error: {e}");
                continue;
            }
        };

        match cmd_id {
            START_TRANSACTION => {
                let res: StartTransaction = borsh::from_slice(&payload)
                    .expect("Failed to parse StartTransaction. Crashing..");
                let Some(stream_tx) = devices_tx.get(&res.device_id) else {
                    eprintln!(
                        "Igloo requested to start a transaction for unknown device {}",
                        res.device_id
                    );
                    continue;
                };
                cur_device_tx = Some(stream_tx.clone());
            }
            END_TRANSACTION => {
                cur_device_tx = None;
            }
            DEVICE_CREATED => {
                let Ok(params) = borsh::from_slice::<DeviceCreated>(&payload) else {
                    eprintln!("Failed to parse DeviceCreated. Skipping..");
                    continue;
                };

                // pull out pending device
                let mut pc = pending_creation.lock().await;
                let Some(mut device) = pc.remove(&params.name) else {
                    eprintln!("Igloo sent DeviceCreated for unknown device. Skipping..");
                    continue;
                };
                drop(pc);

                // save to disk
                config.devices.insert(params.id, device.params.clone());
                config.save().await.unwrap();

                // give actual ID now
                device.id = params.id;

                // run
                let (stream_tx, stream_rx) = mpsc::channel(100);
                devices_tx.insert(params.id, stream_tx);
                let shared_writer_copy = shared_writer.clone();
                tokio::spawn(async move {
                    device.run(shared_writer_copy, stream_rx).await.unwrap(); // TODO log?
                });
            }
            ADD_DEVICE => {
                let Ok(params) = borsh::from_slice::<ConnectionParams>(&payload) else {
                    eprintln!("Failed to parse ConnectionParams for AddDevice command. Skipping..");
                    continue;
                };

                let mut device = Device::new(0, params);
                let shared_writer_copy = shared_writer.clone();
                let pending_creation_copy = pending_creation.clone();
                tokio::spawn(async move {
                    let info = device.connect().await.unwrap();
                    let mut writer = shared_writer_copy.lock().await;
                    writer
                        .create_device(&CreateDevice {
                            name: info.name.clone(),
                        })
                        .await
                        .unwrap();
                    drop(writer);
                    let mut pc = pending_creation_copy.lock().await;
                    pc.insert(info.name, device);
                    drop(pc);
                });
            }
            _ => {
                if let Some(stream_tx) = &mut cur_device_tx {
                    match stream_tx.try_send((cmd_id, payload)) {
                        Ok(_) => {}
                        Err(mpsc::error::TrySendError::Full(_)) => {
                            eprintln!("Device is slow during transaction. Cancelling!");
                            cur_device_tx = None;
                        }
                        Err(mpsc::error::TrySendError::Closed(_)) => {
                            eprintln!("Device disconnected during transaction");
                            cur_device_tx = None;
                        }
                    }
                } else {
                    eprintln!("Got unexpected command {cmd_id} while not in transaction!");
                }
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
