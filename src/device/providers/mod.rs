use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{cli::model::Cli, entity::{EntityState, TargetedEntityCommand}};

pub mod esphome;
pub mod dummy;
pub mod periodic;
pub mod mqtt;

#[derive(Debug, Deserialize, Serialize)]
pub enum ProviderConfig {
    ESPHome(esphome::Config),
    Dummy(dummy::Config),
    PeriodicCommand(periodic::Config),
    MQTT(mqtt::Config)
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DeviceConfig {
    ESPHome(esphome::DeviceConfig),
    Dummy(dummy::DeviceConfig),
    PeriodicTask(periodic::DeviceConfig),
    MQTT(mqtt::DeviceConfig)
}

impl DeviceConfig {
    pub fn spawn(
        self,
        did: usize,
        selector: String,
        back_cmd_tx: mpsc::Sender<Cli>,
        cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
        on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
    ) {
        match self {
            DeviceConfig::ESPHome(cfg) => {
                tokio::spawn(esphome::task(
                    cfg,
                    did,
                    selector,
                    back_cmd_tx.clone(),
                    cmd_rx,
                    on_change_tx.clone(),
                ));
            }
            DeviceConfig::Dummy(cfg) => {
                tokio::spawn(dummy::task(
                    cfg,
                    did,
                    selector,
                    back_cmd_tx.clone(),
                    cmd_rx,
                    on_change_tx.clone(),
                ));
            }
            DeviceConfig::PeriodicTask(cfg) => {
                tokio::spawn(periodic::task(
                    cfg,
                    did,
                    selector,
                    back_cmd_tx.clone(),
                    cmd_rx,
                    on_change_tx.clone(),
                ));
            }
            DeviceConfig::MQTT(_cfg) => todo!(),
        }
    }
}
