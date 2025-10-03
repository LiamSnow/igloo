use crate::device::{ConnectionParams, Device, DeviceError};
use futures_util::StreamExt;
use igloo_interface::{
    END_TRANSACTION, FloeWriterDefault, START_DEVICE_TRANSACTION, StartDeviceTransaction,
    StartRegistrationTransaction, floe_init,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    fs,
    sync::{Mutex, mpsc},
    task::JoinSet,
};
use uuid::Uuid;

pub mod connection;
pub mod device;
pub mod entity;
pub mod api {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}
pub mod model {
    include!(concat!(env!("OUT_DIR"), "/model.rs"));
}

pub const CONFIG_FILE: &str = "./data/config.json";

/// Eventually this will be described in the Floe.toml file
pub const ADD_DEVICE: u16 = 32;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Config {
    /// maps Persisnt Igloo Device ID -> Connection Params
    device_map: HashMap<String, ConnectionParams>,
}

// ! TODO: While this approach works, its kinda strange.
// I think really what should happen is we have 1 reader
// which reads the entire transaction, and ships those bytes over
// to each device
// This way if one device is unresponsive, we dont have to sit
// around waiting for it.

#[tokio::main]
async fn main() {
    let contents = fs::read_to_string(CONFIG_FILE).await.unwrap();
    let config: Config = serde_json::from_str(&contents).unwrap();

    let (writer, mut reader) = floe_init().await.unwrap();
    let shared_writer = Arc::new(Mutex::new(writer));

    let devices_tx = Arc::new(Mutex::new(HashMap::new()));

    // connect to devices in config
    let mut pending_devices = JoinSet::new();
    for (device_id, params) in config.device_map {
        connect_device(&mut pending_devices, device_id, params);
    }

    // handle when new devices connect
    let (add_device_tx, add_device_rx) = mpsc::channel(10);
    tokio::spawn(handle_pending_devices(
        pending_devices,
        add_device_rx,
        devices_tx.clone(),
        shared_writer.clone(),
    ));

    let mut cur_device_tx = None;

    loop {
        let Some(res) = reader.next().await else {
            println!("Socket closed. Shutting down..");
            break;
        };

        let (cmd_id, payload) = match res {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Frame read error: {e}");
                continue;
            }
        };

        match cmd_id {
            START_DEVICE_TRANSACTION => {
                let res: StartDeviceTransaction = borsh::from_slice(&payload).unwrap(); // FIXME unwrap
                let devices = devices_tx.lock().await;
                let Some(stream_tx) = devices.get(&res.device_idx) else {
                    eprintln!(
                        "Igloo requested to start a transaction for unknown device {}",
                        res.device_idx
                    );
                    continue;
                };
                cur_device_tx = Some(stream_tx.clone());
            }
            END_TRANSACTION => {
                cur_device_tx = None;
            }
            ADD_DEVICE => {
                let params: ConnectionParams = borsh::from_slice(&payload).unwrap();
                add_device_tx
                    .send((Uuid::now_v7().to_string(), params))
                    .await
                    .unwrap();
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

async fn handle_pending_devices(
    mut pending_devices: JoinSet<Result<(Device, String, String), DeviceError>>,
    mut add_device_rx: mpsc::Receiver<(String, ConnectionParams)>,
    devices_tx: Arc<Mutex<HashMap<u16, mpsc::Sender<(u16, Vec<u8>)>>>>,
    shared_writer: Arc<Mutex<FloeWriterDefault>>,
) {
    let mut next_device_idx = 0u16;

    loop {
        tokio::select! {
            // connect to new device
            Some((device_id, params)) = add_device_rx.recv() => {
                connect_device(&mut pending_devices, device_id, params);
            }

            // device has connected -> register
            Some(res) = pending_devices.join_next() => {
                match res {
                    Ok(Ok((mut device, device_id, initial_name))) => {
                        let mut writer = shared_writer.lock().await;

                        writer
                            .start_registration_transaction(&StartRegistrationTransaction {
                                device_id,
                                initial_name,
                                device_idx: next_device_idx,
                            })
                            .await
                            .unwrap();

                        device
                            .register_entities(&mut writer, next_device_idx)
                            .await
                            .unwrap();

                        writer.end_transaction().await.unwrap();
                        writer.flush().await.unwrap();
                        drop(writer);

                        let (stream_tx, stream_rx) = mpsc::channel(100);
                        {
                            devices_tx.lock().await.insert(next_device_idx, stream_tx);
                        }

                        let shared_writer_copy = shared_writer.clone();
                        tokio::spawn(async move {
                            device
                                .run(shared_writer_copy, stream_rx)
                                .await
                                .unwrap();
                        });

                        next_device_idx += 1;
                    }
                    Ok(Err(_)) => {} // already logged, maybe log here instead?
                    Err(e) => {
                        eprintln!("Device connection task panicked: {:?}", e);
                    }
                }
            }
        }
    }
}

fn connect_device(
    join_set: &mut JoinSet<Result<(Device, String, String), DeviceError>>,
    device_id: String,
    params: ConnectionParams,
) {
    join_set.spawn(async move {
        let name = params.name.clone();
        let mut device = Device::new(params);
        match device.connect().await {
            Ok(info) => Ok((device, device_id, name.unwrap_or(info.name))),
            Err(e) => {
                eprintln!("Failed to connect to device {}: {:?}", device_id, e);
                Err(e)
            }
        }
    });
}
