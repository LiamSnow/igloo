use esphome::{ESPHomeConfig, ESPHomeDeviceConfig};
use esphomebridge_rs::device::ESPHomeDevice;
use homekit::{HomeKitConfig, HomeKitDeviceConfig};
use serde::{Deserialize, Serialize};

pub mod esphome;
pub mod homekit;

pub enum IglooDevice {
    ESPHome(ESPHomeDevice)
}

#[derive(Debug, Serialize)]
pub enum DeviceType {
    ESPHome
}

impl IglooDevice {
    pub fn get_type(&self) -> DeviceType {
        match self {
            Self::ESPHome(_) => DeviceType::ESPHome
        }
    }
}

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
