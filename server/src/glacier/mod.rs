use igloo_interface::ComponentUpdate;
use thiserror::Error;
use tokio::sync::mpsc;

mod floe;
mod query;
mod supervisor;
mod tree;

pub use supervisor::GlacierSupervisor;

use crate::glacier::tree::{DeviceTree, DeviceTreeUpdate};

#[derive(Error, Debug)]
pub enum GlacierError {
    #[error("Failed to spawn floe: {0}")]
    SpawnError(String),
    #[error("IO error: {0}")]
    Io(#[from] tokio::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Glacier is the owner of the device tree/state, it:
///  - Takes device tree updates from Floes
///  - Takes external query and modification requests
pub struct Glacier {
    rx: mpsc::Receiver<DeviceTreeUpdate>,
    tree: DeviceTree,
}

impl Glacier {
    pub(self) fn new(rx: mpsc::Receiver<DeviceTreeUpdate>, state: DeviceTree) -> Self {
        Self { rx, tree: state }
    }

    pub async fn run(mut self) {
        while let Some(update) = self.rx.recv().await {
            let res = match update {
                DeviceTreeUpdate::AddDevice {
                    floe_name,
                    id,
                    name,
                    entities,
                } => self.tree.add_device(floe_name, id, name, entities).await,
                DeviceTreeUpdate::ComponentUpdates { floe_name, updates } => {
                    self.handle_component_updates(floe_name, updates).await;
                    continue;
                }
            };

            if let Err(e) = res {
                eprintln!("Glacier error: {e}");
            }
        }
    }

    async fn handle_component_updates(&mut self, floe_name: String, updates: Vec<ComponentUpdate>) {
        // TODO FIXME remoe this
        println!("{floe_name} sends updates: {updates:#?}");

        for update in updates {
            let Some(dev) = self.tree.get_dev_mut(&update.device) else {
                eprintln!(
                    "{floe_name} tried to update non-existant device {}",
                    update.device
                );
                continue;
            };

            if dev.floe_name != floe_name {
                eprintln!(
                    "{floe_name} tried to update device {} which it doesn't own!",
                    dev.floe_name
                );
                continue;
            }

            let Some(entity) = dev.entities.get_mut(&update.entity) else {
                eprintln!(
                    "{floe_name} tried to update non-existant entity {} on device {}",
                    update.entity, update.device
                );
                continue;
            };

            for value in update.values {
                entity.set(value);
            }
        }
    }
}
