use std::{error::Error, path::Path, sync::Arc};

use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use tokio::{
    fs,
    sync::{Mutex, mpsc},
};
use uuid::Uuid;

use crate::glacier::query::{Area, LocalArea, LocalQuery, Query};

mod entity;
mod floe;
pub mod query;

pub const STATE_FILE: &str = "state.toml";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GlacierState {
    zones: FxHashMap<Uuid, Zone>,
    devices: FxHashMap<String, DeviceInfo>,
    /// maps Floe idx -> query port
    #[serde(skip)]
    floes: Vec<Floe>,
}

#[derive(Debug)]
pub struct Floe {
    query_tx: mpsc::Sender<LocalQuery>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Zone {
    pub name: String,
    devices: FxHashSet<String>,
    /// (floe_idx, dev_idx)
    /// for fast dispatching
    #[serde(skip)]
    idxs: SmallVec<[(usize, u16); 20]>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    #[serde(skip)]
    pub idx: u16,
    #[serde(skip)]
    pub entity_names: Vec<String>,
    #[serde(skip)]
    floe_idx: usize,
}

impl GlacierState {
    pub async fn dispatch_query(&self, req: Query) {
        // TODO this _should_ be parallelized at some point
        match req.area {
            Area::All => {
                // dispatch to every floe
                for floe in &self.floes {
                    let req = LocalQuery {
                        filter: req.filter.clone(),
                        kind: req.kind.clone(),
                        area: LocalArea::All,
                        started_at: req.started_at,
                    };
                    floe.query_tx.send(req).await.unwrap();
                }
            }
            Area::Zone(persistent_id) => {
                let Some(zone) = self.zones.get(&persistent_id) else {
                    panic!("Invalid zone ID {persistent_id}");
                };

                // dispatch command for every device
                // (may send multiple commands to same floe)
                for (floe_idx, dev_idx) in &zone.idxs {
                    let req = LocalQuery {
                        filter: req.filter.clone(),
                        kind: req.kind.clone(),
                        area: LocalArea::Device(*dev_idx),
                        started_at: req.started_at,
                    };
                    self.floes[*floe_idx].query_tx.send(req).await.unwrap();
                }
            }
            Area::Device(persistent_id) => {
                let Some(device) = self.devices.get(&persistent_id) else {
                    panic!("Invalid device ID {persistent_id}");
                };

                let req = LocalQuery {
                    filter: req.filter.clone(),
                    kind: req.kind.clone(),
                    area: LocalArea::Device(device.idx),
                    started_at: req.started_at,
                };
                self.floes[device.floe_idx]
                    .query_tx
                    .send(req)
                    .await
                    .unwrap();
            }
            Area::Entity(persistent_id, entity_name) => {
                let Some(device) = self.devices.get(&persistent_id) else {
                    panic!("Invalid device ID {persistent_id}");
                };

                let entity_idx = device
                    .entity_names
                    .iter()
                    .position(|name| *name == entity_name)
                    .unwrap_or_else(|| panic!("Invalid entity name {entity_name}"));

                let req = LocalQuery {
                    filter: req.filter.clone(),
                    kind: req.kind.clone(),
                    area: LocalArea::Entity(device.idx, entity_idx as u16),
                    started_at: req.started_at,
                };

                self.floes[device.floe_idx]
                    .query_tx
                    .send(req)
                    .await
                    .unwrap();
            }
        }
    }
}

pub async fn run() -> Result<Arc<Mutex<GlacierState>>, Box<dyn Error>> {
    let mut state = GlacierState::load().await;

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

        let (query_tx, query_rx) = mpsc::channel(10);

        floe::spawn(
            name.to_string(),
            state.floes.len(),
            reg_dev_tx.clone(),
            query_rx,
        )
        .await?;

        state.floes.push(Floe { query_tx });
    }

    state.save().await;

    let state = Arc::new(Mutex::new(state));
    let state_copy = state.clone();
    tokio::spawn(async move {
        while let Some((dev_id, dev_info)) = reg_dev_rx.recv().await {
            let mut state = state_copy.lock().await;
            state.register_device(dev_id, dev_info).await;
        }
    });

    Ok(state)
}

impl GlacierState {
    pub async fn load() -> Self {
        match fs::try_exists(STATE_FILE).await {
            Ok(true) => match fs::read_to_string(STATE_FILE).await {
                Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
                    panic!("Failed to parse state file: {}", e);
                }),
                Err(e) => {
                    panic!("Failed to read state file: {}", e);
                }
            },
            Ok(false) | Err(_) => {
                let state = Self::default();
                state.save().await;
                state
            }
        }
    }

    async fn save(&self) {
        let json = toml::to_string_pretty(self).expect("Failed to serialize state");
        fs::write(STATE_FILE, json)
            .await
            .expect("Failed to write state file");
    }

    pub async fn add_zone(&mut self, id: Uuid, zone: Zone) {
        self.zones.insert(id, zone);
        // TODO FIXME whenever they add a device, we need to populate device_idxs
        self.save().await;
    }

    async fn register_device(&mut self, id: String, new: DeviceInfo) {
        match self.devices.get_mut(&id) {
            Some(existing) => {
                println!("Existing device {} registered", existing.name);

                *existing = DeviceInfo {
                    name: existing.name.clone(),
                    idx: new.idx,
                    entity_names: new.entity_names,
                    floe_idx: new.floe_idx,
                };

                // we know that only existing devices can exist in zones
                // so we only need to run this for this case
                for zone in self.zones.values_mut() {
                    if zone.devices.contains(&id) {
                        zone.idxs.push((new.floe_idx, new.idx));
                    }
                }
            }
            None => {
                println!("New device {} registered", new.name);
                self.devices.insert(id, new);
            }
        }
        self.save().await;
    }
}
