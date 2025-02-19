use std::error::Error;

use esphomebridge_rs::device::ESPHomeDevice;
use serde::{Deserialize, Serialize};

use crate::device::{command::DeviceCommand, device::IglooDevice};

#[derive(Debug, Deserialize, Serialize)]
pub struct ESPHomeConfig {
    pub default_port: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ESPHomeDeviceConfig {
    pub ip: String,
    pub password: Option<String>,
    pub noise_psk: Option<String>
}

pub fn make(config: ESPHomeDeviceConfig) -> Result<IglooDevice, Box<dyn Error>> {
    let mut ip = config.ip;
    if !ip.contains(':') {
        ip += ":6053";
    }

    let dev;
    if let Some(noise_psk) = config.noise_psk {
        dev = ESPHomeDevice::new_noise(ip, noise_psk);
    }
    else if let Some(password) = config.password {
        dev = ESPHomeDevice::new_plain(ip, password);
    }
    else {
        return Err("ESPHome device must have noise_psk or password!".into());
    }

    Ok(IglooDevice::ESPHome(dev))
}

pub async fn execute(dev: &mut ESPHomeDevice, cmd: DeviceCommand, subdevice_name: String) -> Result<(), Box<dyn Error>> {
    match cmd {
        DeviceCommand::Connect => {
            dev.connect().await?
        },
        DeviceCommand::Light(cmd) => if let Some(entity) = dev.entities.light.get(&subdevice_name) {
            dev.light_command(&cmd.to_esphome(entity.key)).await?
        },
    }
    Ok(())
}

pub async fn execute_global(dev: &mut ESPHomeDevice, cmd: DeviceCommand) -> Result<(), Box<dyn Error>> {
    match cmd {
        DeviceCommand::Connect => dev.connect().await?,
        DeviceCommand::Light(cmd) => {
            let mut esp_cmd = cmd.clone().to_esphome(0);
            for key in dev.get_primary_light_keys() {
                esp_cmd.key = key;
                dev.light_command(&esp_cmd).await?
            }
        },
    }
    Ok(())
}
