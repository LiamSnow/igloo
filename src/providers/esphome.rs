use core::str;
use std::{error::Error, time::Duration};

use esphomebridge_rs::{api::{self, LightStateResponse}, device::ESPHomeDevice, entity::ENTITY_CATEGORY_NONE};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{cli::model::LightAction, device::{command::{Color, LightState, SubdeviceCommand}, device::{IglooDeviceLock, SubdeviceInfo}}};

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

// matches aioesphomeapi
pub const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(20);
pub const KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(90);

pub fn make(config: ESPHomeDeviceConfig) -> Result<IglooDevice, Box<dyn Error>> {
    let mut ip = config.ip;
    if !ip.contains(':') {
        ip += ":6053";
    }

    let (tx, mut rx) = mpsc::channel::<String>(32);

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
    {
        let mut idev = clone.write().await;
        match *idev {
            IglooDevice::ESPHome(ref mut dev) => {
                dev.connect().await?;
                dev.subscribe_states().await?;
            },
        }
    }

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(KEEP_ALIVE_INTERVAL);
        loop {
            interval.tick().await;
            let mut idev = dev_lock.write().await;
            match *idev {
                IglooDevice::ESPHome(ref mut dev) => {
                    dev.ping_no_wait().await.unwrap();

                    //FIXME what if device never pinged?
                    if let Some(last_ping) = dev.last_ping {
                        if last_ping.elapsed().unwrap() > KEEP_ALIVE_TIMEOUT {
                            //TODO handle this? at least include devices name
                            println!("Device has timed out -> Reconnecting");
                            dev.force_disconnect().await.unwrap();
                            dev.connect().await.unwrap();
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

pub async fn execute(dev: &mut ESPHomeDevice, cmd: SubdeviceCommand, subdevice_name: &str) -> Result<(), Box<dyn Error>> {
    match cmd {
        SubdeviceCommand::Light(cmd) => if let Some(entity) = dev.entities.light.get(subdevice_name) {
            dev.light_command(&cmd.to_esphome(entity.key)).await?
        },
    }
    Ok(())
}

pub async fn execute_global(dev: &mut ESPHomeDevice, cmd: SubdeviceCommand) -> Result<(), Box<dyn Error>> {
    match cmd {
        SubdeviceCommand::Light(cmd) => {
            let mut esp_cmd = cmd.clone().to_esphome(0);
            for key in dev.get_primary_light_keys() {
                esp_cmd.key = key;
                dev.light_command(&esp_cmd).await?
            }
        },
    }
    Ok(())
}

pub fn get_light_state(dev: &ESPHomeDevice, subdevice_name: &str) -> Option<LightState> {
    let entity = dev.entities.light.get(subdevice_name)?;
    let state = dev.states.light.get(&entity.key)?;
    let state = state.as_ref()?;
    Some(state.into())
}

pub fn get_global_light_state(dev: &ESPHomeDevice) -> Option<LightState> {
    let mut states: Vec<LightState> = Vec::new();
    for key in dev.get_primary_light_keys() {
        if let Some(Some(state)) = dev.states.light.get(&key) {
            states.push(state.into());
        }
    }

    None
}

pub fn list_subdevs(dev: &ESPHomeDevice) -> Vec<SubdeviceInfo> {
    let mut names = Vec::new();

    for entity in dev.entities.get_all() {
        names.push(SubdeviceInfo {
            typ: entity.typ.to_string(),
            name: entity.object_id.to_string(),
            is_diagnostic: entity.category != ENTITY_CATEGORY_NONE
        })
    }

    for (_, service) in &dev.services {
        names.push(SubdeviceInfo {
            typ: "service".to_string(),
            name: service.name.clone(),
            is_diagnostic: false
        })
    }
    names
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
            has_state: true,
            state: true,
            ..Default::default()
        };

        match self {
            LightAction::On => {},
            LightAction::Off => {
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

impl From<&LightStateResponse> for LightState {
    fn from(value: &LightStateResponse) -> Self {
        Self {
            on: value.state,
            temp: Some(value.color_temperature as u32),
            brightness: Some(value.brightness as u8),
            color: Some(Color {
                r: (value.red * 255.) as u8,
                g: (value.green * 255.) as u8,
                b: (value.blue * 255.) as u8,
            }),
            color_on: value.color_mode == 35
        }
    }
}

