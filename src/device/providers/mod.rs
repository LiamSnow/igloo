use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use crate::{
    entity::{EntityState, TargetedEntityCommand},
    state::IglooState,
};

pub mod dummy;
pub mod esphome;
pub mod mqtt;
pub mod periodic;

#[derive(Debug, Deserialize, Serialize)]
pub enum ProviderConfig {
    ESPHome(esphome::Config),
    Dummy(dummy::Config),
    PeriodicCommand(periodic::Config),
    MQTT(mqtt::Config),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DeviceConfig {
    ESPHome(esphome::DeviceConfig),
    Dummy(dummy::DeviceConfig),
    PeriodicTask(periodic::DeviceConfig),
    MQTT(mqtt::DeviceConfig),
}

pub fn init(cfgs: Vec<ProviderConfig>) {
    for cfg in cfgs {
        match cfg {
            ProviderConfig::ESPHome(_cfg) => todo!(),
            ProviderConfig::Dummy(_cfg) => todo!(),
            ProviderConfig::PeriodicCommand(_cfg) => todo!(),
            ProviderConfig::MQTT(cfg) => mqtt::init_provider(cfg),
        }
    }
}

impl DeviceConfig {
    pub fn spawn(
        self,
        did: usize,
        selection: String,
        cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
        on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
    ) -> Option<oneshot::Sender<Arc<IglooState>>> {
        match self {
            DeviceConfig::ESPHome(cfg) => {
                tokio::spawn(esphome::task(
                    cfg,
                    did,
                    selection,
                    cmd_rx,
                    on_change_tx.clone(),
                ));
                None
            }
            DeviceConfig::Dummy(cfg) => {
                tokio::spawn(dummy::task(
                    cfg,
                    did,
                    selection,
                    cmd_rx,
                    on_change_tx.clone(),
                ));
                None
            }
            DeviceConfig::PeriodicTask(cfg) => {
                let (istate_tx, istate_rx) = oneshot::channel();
                tokio::spawn(periodic::task(
                    cfg,
                    did,
                    selection,
                    istate_rx,
                    cmd_rx,
                    on_change_tx.clone(),
                ));
                Some(istate_tx)
            }
            DeviceConfig::MQTT(cfg) => {
                let (istate_tx, istate_rx) = oneshot::channel();
                tokio::spawn(mqtt::task(
                    cfg,
                    did,
                    selection,
                    istate_rx,
                    cmd_rx,
                    on_change_tx.clone(),
                ));
                Some(istate_tx)
            },
        }
    }
}
