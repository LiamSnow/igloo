use serde::{Deserialize, Serialize};

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
