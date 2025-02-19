use std::{error::Error, sync::Arc};

use esphomebridge_rs::device::ESPHomeDevice;
use tokio::sync::RwLock;

use crate::providers::{DeviceConfig, self};

use super::command::DeviceCommand;

pub enum IglooDevice {
    ESPHome(ESPHomeDevice)
}

pub type IglooDeviceArc = Arc<RwLock<IglooDevice>>;


impl IglooDevice {
    pub fn make(config: DeviceConfig) -> Result<Self, Box<dyn Error>> {
        match config {
            DeviceConfig::ESPHome(config) => providers::esphome::make(config),
            DeviceConfig::HomeKit(_) => panic!("oops"),
        }
    }

    pub async fn execute(&mut self, cmd: DeviceCommand, subdevice_name: String) -> Result<(), Box<dyn Error>> {
        match self {
            IglooDevice::ESPHome(dev) => providers::esphome::execute(dev, cmd, subdevice_name).await,
        }
    }

    pub async fn execute_global(&mut self, cmd: DeviceCommand) -> Result<(), Box<dyn Error>> {
        match self {
            IglooDevice::ESPHome(dev) => providers::esphome::execute_global(dev, cmd).await,
        }
    }

    pub async fn execute_arc(dev_arc: IglooDeviceArc, cmd: DeviceCommand, subdevice_name: String) {
        let mut device = dev_arc.write().await;
        let res = device.execute(cmd, subdevice_name).await;
        if let Err(e) = res {
            println!("error executing arc: {e}");
        }
    }

    pub async fn execute_global_arc(dev_arc: IglooDeviceArc, cmd: DeviceCommand) {
        let mut device = dev_arc.write().await;
        let res = device.execute_global(cmd).await;
        if let Err(e) = res {
            println!("error executing arc: {e}");
        }
    }
}
