use core::str;
use std::{error::Error, time::Duration};

use esphomebridge_rs::{api, device::ESPHomeDevice, entity::EntityCategory};
use serde::{Deserialize, Serialize};

use crate::{cli::model::LightAction, device::{command::DeviceCommand, device::IglooDeviceLock}};

use super::IglooDevice;

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

pub async fn connect(dev_lock: IglooDeviceLock) -> Result<(), Box<dyn Error>> {
    let clone = dev_lock.clone();
    let mut idev = clone.write().await;
    match *idev {
        IglooDevice::ESPHome(ref mut dev) => dev.connect().await?,
    }
    drop(idev);

    tokio::spawn(async move {
        // == ESPHome server's keepalive (has tolerance of 2.5x)
        // TODO when we add logging, should we reduce this so the TCP buffer does fill up to much?
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let mut idev = dev_lock.write().await;
            match *idev {
                IglooDevice::ESPHome(ref mut dev) => dev.process_incoming().await.unwrap(), //FIXME unwrap
            }
        }
    });

    Ok(())
}

pub async fn execute(dev: &mut ESPHomeDevice, cmd: DeviceCommand, subdevice_name: String) -> Result<(), Box<dyn Error>> {
    match cmd {
        DeviceCommand::Light(cmd) => if let Some(entity) = dev.entities.light.get(&subdevice_name) {
            dev.light_command(&cmd.to_esphome(entity.key)).await?
        },
    }
    Ok(())
}

pub async fn execute_global(dev: &mut ESPHomeDevice, cmd: DeviceCommand) -> Result<(), Box<dyn Error>> {
    match cmd {
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

pub fn list_subdevs(dev: &ESPHomeDevice) -> Vec<String> {
    let mut names = Vec::new();
    for (entity_type, entities) in dev.entities.to_hashmap() {
        for (entity_name, entity) in entities {
            if entity.category != EntityCategory::None {
                names.push(format!("{entity_type} {entity_name} ({})", entity.category));
            }
            else {
                names.push(format!("{entity_type} {entity_name}"));
            }
        }
    }
    for (_, service) in &dev.services {
        names.push(format!("service {}", service.name));
    }
    names
}

pub fn describe(dev: &ESPHomeDevice) -> String {
    let mut s = String::new();

    s += &format!("connected: {}", dev.connected);
    //TODO ?? show entites and services here?

    s
}

pub async fn subscribe_logs(_dev: &mut ESPHomeDevice) {
    //TODO
}

impl LightAction {
    pub fn to_esphome(self, key: u32) -> api::LightCommandRequest {
        let mut cmd = api::LightCommandRequest {
            key,
            has_transition_length: true,
            transition_length: 0,
            ..Default::default()
        };

        match self {
            LightAction::On => {
                cmd.has_state = true;
                cmd.state = true;
            },
            LightAction::Off => {
                cmd.has_state = true;
                cmd.state = false;
            },
            LightAction::Color(color) => {
                cmd.has_rgb = true;
                //RGB are relative values (IE red % = red / (red + blue + green))
                cmd.red = color.r as f32 / 255.; //so we dont need to / 255
                cmd.green = color.g as f32 / 255.;
                cmd.blue = color.b as f32 / 255.;
            },
            LightAction::Temperature { temp } => {
                cmd.has_color_temperature = true;
                cmd.color_temperature = temp as f32;
            },
            LightAction::Brightness { brightness } => {
                cmd.has_brightness = true;
                cmd.brightness = brightness as f32 / 100.;
            },
        }

        cmd
    }
}
