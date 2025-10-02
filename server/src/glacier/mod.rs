use std::{error::Error, path::Path, sync::Arc};

use rustc_hash::{FxHashMap, FxHashSet};
use tokio::{
    fs,
    sync::{Mutex, mpsc},
};
use uuid::Uuid;

mod entity;
mod floe;

/// Floe Name, Persistent Device ID
pub type GlobalDeviceID = (String, String);

// TODO PERSISTENT
#[derive(Debug, Default)]
pub struct GlacierState {
    zones: FxHashMap<Uuid, Zone>,
    devices: FxHashMap<GlobalDeviceID, DeviceInfo>,
}

#[derive(Debug)]
pub struct Zone {
    pub name: String,
    pub devices: FxHashSet<GlobalDeviceID>,
}

#[derive(Debug)]
pub struct DeviceInfo {
    pub name: String,
    /// dont serialize this
    pub entity_names: Vec<String>,
}

pub async fn run() -> Result<Arc<Mutex<GlacierState>>, Box<dyn Error>> {
    let state = Arc::new(Mutex::new(GlacierState::default()));

    let (reg_dev_tx, mut reg_dev_rx) = mpsc::channel(50);

    let floes_path = Path::new("./floes");
    if !floes_path.exists() {
        fs::create_dir(floes_path).await?;
        println!("Created directory: ./floes");
    } else if !floes_path.is_dir() {
        panic!("./floes exists but is not a directory!");
    }

    let mut entries = fs::read_dir(floes_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name() else {
            continue;
        };
        let Some(name) = name.to_str() else {
            continue;
        };

        // TODO add query channel here, collect them and give to query engine
        floe::spawn(name.to_string(), reg_dev_tx.clone()).await?;
        // floe_handles.push(handle); TODO should we keep track of them?
    }

    let state_copy = state.clone();
    tokio::spawn(async move {
        while let Some((dev_id, dev_info)) = reg_dev_rx.recv().await {
            let mut state = state_copy.lock().await;

            match state.devices.get_mut(&dev_id) {
                Some(di) => {
                    println!(
                        "Existing device {} registered with entities {:#?}",
                        di.name, dev_info.entity_names
                    );
                    // if this device already exists, only update
                    // the entity_names because the device.name
                    // is persistent and can be changed by user
                    di.entity_names = dev_info.entity_names;
                }
                None => {
                    println!(
                        "New device {} registered with entities {:#?}",
                        dev_info.name, dev_info.entity_names
                    );
                    state.devices.insert(dev_id, dev_info);
                }
            }

            // TODO save to disk
        }
    });

    Ok(state)
}
