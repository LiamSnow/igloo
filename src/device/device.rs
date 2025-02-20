use std::{error::Error, sync::Arc};

use tokio::sync::RwLock;

use crate::providers::{self, DeviceConfig, DeviceType, IglooDevice};

use super::command::DeviceCommand;

pub type IglooDeviceLock = Arc<RwLock<IglooDevice>>;

impl IglooDevice {
    pub fn make(config: DeviceConfig) -> Result<Self, Box<dyn Error>> {
        match config {
            DeviceConfig::ESPHome(config) => providers::esphome::make(config),
            DeviceConfig::HomeKit(_) => panic!("oops"),
        }
    }

    pub async fn connect(dev_lock: IglooDeviceLock) {
        let dev = dev_lock.read().await;
        let typ: DeviceType = dev.get_type();
        drop(dev);

        let res = match typ {
            DeviceType::ESPHome => providers::esphome::connect(dev_lock.clone()).await,
        };
        if let Err(e) = res {
            println!("error connecting igloo device lock: {e}"); //FIXME
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

    pub async fn execute_lock(dev_lock: IglooDeviceLock, cmd: DeviceCommand, subdevice_name: String) {
        let mut dev = dev_lock.write().await;
        let res = dev.execute(cmd, subdevice_name).await;
        if let Err(e) = res {
            println!("error executing on igloo device lock: {e}"); //FIXME
        }
    }

    pub async fn execute_global_lock(dev_lock: IglooDeviceLock, cmd: DeviceCommand) {
        let mut dev = dev_lock.write().await;
        let res = dev.execute_global(cmd).await;
        if let Err(e) = res {
            println!("error global executing on igloo device lock: {e}"); //FIXME
        }
    }

    //TODO struct?
    pub fn list_subdevs(&self) -> Vec<String> {
        match self {
            IglooDevice::ESPHome(dev) => providers::esphome::list_subdevs(dev),
        }
    }

    //TODO struct?
    pub fn describe(&self) -> String {
        match self {
            IglooDevice::ESPHome(dev) => providers::esphome::describe(dev),
        }
    }

    //TODO streaming?
    pub async fn subscribe_logs(&mut self) {
        match self {
            IglooDevice::ESPHome(dev) => providers::esphome::subscribe_logs(dev).await,
        }
    }
}
