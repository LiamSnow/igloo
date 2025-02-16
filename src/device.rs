use std::sync::Arc;

use esphomebridge_rs::{connection::{noise::NoiseConnection, plain::PlainConnection}, device::Device};
use tokio::sync::RwLock;

use crate::{config::DeviceConfig, command::{CTLightCommand, IglooCommand, RGBLightCommand}};

pub type IglooDeviceArc = Arc<RwLock<IglooDevice>>;

pub enum IglooDevice {
    ESPHomePlain(Device<PlainConnection>),
    ESPHomeNoise(Device<NoiseConnection>)
}

impl From<DeviceConfig> for IglooDevice {
    fn from(value: DeviceConfig) -> Self {
        match value {
            DeviceConfig::ESPHome(config) => {
                let mut ip = config.ip;
                if !ip.contains(':') {
                    ip += ":6053";
                }

                if let Some(noise_psk) = config.noise_psk {
                    IglooDevice::ESPHomeNoise(
                        Device::new_noise(ip, noise_psk)
                    )
                }
                else if let Some(password) = config.password {
                    IglooDevice::ESPHomePlain(
                        Device::new_plain(ip, password)
                    )
                }
                else {
                    panic!("ESPHome device must have noise_psk or password!");
                }
            },
            DeviceConfig::Test() => panic!("oops"),
        }
    }
}

impl IglooDevice {
    pub async fn connect(&mut self) {
        match self {
            IglooDevice::ESPHomePlain(device) => device.connect().await.unwrap(),
            IglooDevice::ESPHomeNoise(device) => device.connect().await.unwrap(),
        }
    }

    pub async fn connect_arc(dev_arc: IglooDeviceArc) {
        let mut device = dev_arc.write().await;
        device.connect().await;
        println!("connected");
    }

    pub async fn command(&mut self, cmd: IglooCommand) {
        match cmd {
            IglooCommand::RGBLightCommand(cmd) => self.rgb_light_command(cmd).await,
            IglooCommand::CTLightCommand(cmd) => self.ct_light_command(cmd).await,
        }
    }

    pub async fn command_arc(dev_arc: IglooDeviceArc, cmd: IglooCommand) {
        let mut device = dev_arc.write().await;
        device.command(cmd).await;
    }

    pub async fn rgb_light_command(&mut self, cmd: RGBLightCommand) {
        match self {
            IglooDevice::ESPHomePlain(device) => if let Some(key) = device.first_light_key() {
                device.light_command(&cmd.to_esphome(key)).await.unwrap()
            },
            IglooDevice::ESPHomeNoise(device) => if let Some(key) = device.first_light_key() {
                device.light_command(&cmd.to_esphome(key)).await.unwrap()
            },
        }
    }

    pub async fn ct_light_command(&mut self, cmd: CTLightCommand) {
        match self {
            IglooDevice::ESPHomePlain(device) => if let Some(key) = device.first_light_key() {
                device.light_command(&cmd.to_esphome(key)).await.unwrap()
            },
            IglooDevice::ESPHomeNoise(device) => if let Some(key) = device.first_light_key() {
                device.light_command(&cmd.to_esphome(key)).await.unwrap()
            },
        }
    }

}
