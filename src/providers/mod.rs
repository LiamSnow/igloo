use serde::{Deserialize, Serialize};

pub mod esphome;
pub mod homekit;
pub mod dummy_variable;
pub mod periodic_task;
pub mod mqtt;

#[derive(Debug, Deserialize, Serialize)]
pub enum ProviderConfig {
    ESPHome(esphome::Config),
    HomeKit(homekit::Config),
    DummyVariable(dummy_variable::Config),
    PeriodicTask(periodic_task::Config),
    MQTT(mqtt::Config)
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DeviceConfig {
    ESPHome(esphome::DeviceConfig),
    HomeKit(homekit::DeviceConfig),
    DummyVariable(dummy_variable::DeviceConfig),
    PeriodicTask(periodic_task::DeviceConfig),
    MQTT(mqtt::DeviceConfig)
}
