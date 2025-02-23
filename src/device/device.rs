use std::{error::Error, sync::Arc};

use serde::Serialize;
use tokio::sync::RwLock;

use crate::providers::{self, DeviceConfig, DeviceType, IglooDevice};

use super::command::{LightState, SubdeviceCommand};

pub type IglooDeviceLock = Arc<RwLock<IglooDevice>>;

impl IglooDevice {
    pub fn make(config: DeviceConfig) -> Result<Self, Box<dyn Error>> {
        match config {
            DeviceConfig::ESPHome(config) => providers::esphome::make(config),
            DeviceConfig::HomeKit(_) => panic!("oops"),
        }
    }

    pub async fn connect(dev_lock: IglooDeviceLock) {
        let typ;
        {
            let dev = dev_lock.read().await;
            typ = dev.get_type();
        }

        let res = match typ {
            DeviceType::ESPHome => providers::esphome::connect(dev_lock.clone()).await,
        };
        if let Err(e) = res {
            println!("error connecting igloo device lock: {e}"); //FIXME
        }
    }

    pub async fn execute(&mut self, cmd: SubdeviceCommand, subdevice_name: &str) -> Result<(), Box<dyn Error>> {
        match self {
            IglooDevice::ESPHome(dev) => providers::esphome::execute(dev, cmd, subdevice_name).await,
        }
    }

    /// Execute a subdevice command on all subdevices of that type
    pub async fn execute_global(&mut self, cmd: SubdeviceCommand) -> Result<(), Box<dyn Error>> {
        match self {
            IglooDevice::ESPHome(dev) => providers::esphome::execute_global(dev, cmd).await,
        }
    }

    pub async fn execute_lock(dev_lock: IglooDeviceLock, cmd: SubdeviceCommand, subdevice_name: &str) {
        let mut dev = dev_lock.write().await;
        let res = dev.execute(cmd, subdevice_name).await;
        if let Err(e) = res {
            println!("error executing on igloo device lock: {e}"); //FIXME
        }
    }

    pub fn get_light_state(&self, subdevice_name: &str) -> Option<LightState> {
        match self {
            IglooDevice::ESPHome(dev) => providers::esphome::get_light_state(dev, subdevice_name),
        }
    }
    pub fn get_global_light_state(&self) -> Option<LightState> {
        match self {
            IglooDevice::ESPHome(dev) => providers::esphome::get_global_light_state(dev),
        }
    }

    /// Execute a subdevice command on all subdevices of that type
    pub async fn execute_global_lock(dev_lock: IglooDeviceLock, cmd: SubdeviceCommand) {
        let mut dev = dev_lock.write().await;
        let res = dev.execute_global(cmd).await;
        if let Err(e) = res {
            println!("error global executing on igloo device lock: {e}"); //FIXME
        }
    }

    //TODO struct?
    pub fn list_subdevs(&self) -> Vec<SubdeviceInfo> {
        match self {
            IglooDevice::ESPHome(dev) => providers::esphome::list_subdevs(dev),
        }
    }

    //TODO struct?
    pub fn describe(&self) -> DeviceInfo {
        DeviceInfo {
            typ: self.get_type()
        }
    }

    //TODO streaming?
    pub async fn subscribe_logs(&mut self) {
        match self {
            IglooDevice::ESPHome(dev) => providers::esphome::subscribe_logs(dev).await,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    typ: DeviceType
}

#[derive(Debug, Serialize)]
pub struct SubdeviceInfo {
    pub typ: String,
    pub name: String,
    pub is_diagnostic: bool
}
