use esphome::{ESPHomeConfig, ESPHomeDeviceConfig};
use homekit::{HomeKitConfig, HomeKitDeviceConfig};
use serde::{Deserialize, Serialize};

pub mod esphome;
pub mod homekit;

#[derive(Debug, Deserialize, Serialize)]
pub enum ProviderConfig {
    ESPHome(ESPHomeConfig),
    HomeKit(HomeKitConfig)
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DeviceConfig {
    ESPHome(ESPHomeDeviceConfig),
    HomeKit(HomeKitDeviceConfig),
}
