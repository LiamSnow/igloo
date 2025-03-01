use core::str;
use std::{collections::HashMap, time::Duration};

use esphomebridge_rs::{
    api::{self, LightStateResponse},
    device::ESPHomeDevice,
    entity::EntityStateUpdateValue,
    error::DeviceError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{sync::mpsc, time::timeout};

use crate::{
    cli::model::LightAction,
    command::{
        LightState, SubdeviceCommand, SubdeviceState, SubdeviceType,
        TargetedSubdeviceCommand, RGBF32,
    },
    map::SubdeviceStateUpdate,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct ESPHomeConfig {
    pub default_port: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ESPHomeDeviceConfig {
    pub ip: String,
    pub password: Option<String>,
    pub noise_psk: Option<String>,
}

#[derive(Error, Debug)]
pub enum ESPHomeError {
    #[error("Device must have a noise_psk or password!")]
    MissingAuth,
    #[error("`{0}`")]
    ESPHome(DeviceError),
    #[error("Invalid Subdevice/Entity `{0}`")]
    InvalidSubdevice(String),
}

impl From<DeviceError> for ESPHomeError {
    fn from(value: DeviceError) -> Self {
        Self::ESPHome(value)
    }
}

// ON_OFF = 1 << 0;
// BRIGHTNESS = 1 << 1;
// WHITE = 1 << 2;
// COLOR_TEMPERATURE = 1 << 3;
// COLD_WARM_WHITE = 1 << 4;
// RGB = 1 << 5;

// matches aioesphomeapi
pub const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(20);
pub const KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(90);

pub async fn task(
    config: ESPHomeDeviceConfig,
    did: usize,
    selector: String,
    mut cmd_rx: mpsc::Receiver<TargetedSubdeviceCommand>,
    on_update_of_type: Option<HashMap<SubdeviceType, Vec<mpsc::Sender<SubdeviceStateUpdate>>>>,
    on_subdev_update: Option<HashMap<String, mpsc::Sender<SubdeviceState>>>,
) -> Result<(), ESPHomeError> {
    let mut dev = make_device(config)?;
    dev.connect().await?;

    println!("{selector} ({did}) connected");

    //push state up
    let mut update_rx = dev.subscribe_states(5).await?;
    tokio::spawn(async move {
        while let Some(update) = update_rx.recv().await {
            if let Some(value) = esphome_state_to_igloo(update.value) {
                let subdev_type = value.get_type();

                if let Some(ref on_update_of_type) = on_update_of_type {
                    if let Some(on_update_of_type) = on_update_of_type.get(&subdev_type) {
                        let u = SubdeviceStateUpdate {
                            did,
                            sid: update.entity_index,
                            value: value.clone(),
                        };
                        for tx in on_update_of_type {
                            tx.send(u.clone()).await.unwrap(); //FIXME
                        }
                    }
                }

                if let Some(ref on_subdev_update) = on_subdev_update {
                    if let Some(tx) = on_subdev_update.get(&update.entity_name) {
                        tx.send(value).await.unwrap(); //FIXME
                    }
                }
            }
        }
    });

    //TODO keep alive
    loop {
        match timeout(Duration::from_millis(100), cmd_rx.recv()).await {
            Ok(Some(cmd)) => {
                //TODO replace ? with log
                handle_cmd(&mut dev, cmd).await?;
            }
            Err(_) => {
                //TODO log error
                dev.process_incoming().await?;
            }
            Ok(None) => {
                //TODO log
                break;
            }
        }
    }

    Ok(())
}

async fn handle_cmd(
    dev: &mut ESPHomeDevice,
    cmd: TargetedSubdeviceCommand,
) -> Result<(), ESPHomeError> {
    match cmd.cmd {
        SubdeviceCommand::Light(light_cmd) => {
            if let Some(subdev_name) = cmd.subdev_name {
                let key = dev
                    .get_light_key_from_name(&subdev_name)
                    .ok_or(ESPHomeError::InvalidSubdevice(subdev_name))?;
                dev.light_command(&light_cmd.to_esphome(key)).await?;
            } else {
                let mut esp_cmd = light_cmd.clone().to_esphome(0);
                dev.light_command_global(&mut esp_cmd).await?;
            }
        }
        SubdeviceCommand::Switch(state) => {
            if let Some(subdev_name) = cmd.subdev_name {
                let key = dev
                    .get_switch_key_from_name(&subdev_name)
                    .ok_or(ESPHomeError::InvalidSubdevice(subdev_name))?;
                dev.switch_command(&api::SwitchCommandRequest {
                    key,
                    state: state.into(),
                })
                .await?;
            } else {
                let mut esp_cmd = api::SwitchCommandRequest {
                    key: 0,
                    state: state.into(),
                };
                dev.switch_command_global(&mut esp_cmd).await?;
            }
        }
    }

    Ok(())
}

fn make_device(config: ESPHomeDeviceConfig) -> Result<ESPHomeDevice, ESPHomeError> {
    let ip = if config.ip.contains(':') {
        config.ip
    } else {
        config.ip + ":6053"
    };
    if let Some(noise_psk) = config.noise_psk {
        Ok(ESPHomeDevice::new_noise(ip, noise_psk))
    } else if let Some(password) = config.password {
        Ok(ESPHomeDevice::new_plain(ip, password))
    } else {
        Err(ESPHomeError::MissingAuth)
    }
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
            LightAction::On => {}
            LightAction::Off => {
                cmd.state = false;
            }
            LightAction::Color { hue } => {
                if let Some(hue) = hue {
                    cmd.has_rgb = true;
                    let rgb = RGBF32::from_hue(hue);
                    cmd.red = rgb.r as f32;
                    cmd.green = rgb.g as f32;
                    cmd.blue = rgb.b as f32;
                }
                else {
                    cmd.has_color_mode = true;
                    cmd.color_mode = 35; //FIXME
                }
            }
            LightAction::Temperature { temp } => {
                if let Some(temp) = temp {
                    cmd.has_color_temperature = true;
                    cmd.color_temperature = temp as f32;
                }
                else {
                    cmd.has_color_mode = true;
                    cmd.color_mode = 11; //FIXME
                }
            }
            LightAction::Brightness { brightness } => {
                cmd.has_brightness = true;
                cmd.brightness = brightness as f32 / 100.;
            }
        }

        cmd
    }
}

impl From<LightStateResponse> for LightState {
    fn from(value: LightStateResponse) -> Self {
        Self {
            on: value.state,
            //TODO use supported color modes
            temp: Some(value.color_temperature as u32),
            brightness: Some((value.brightness * 100.) as u8),
            //TODO use supported color modes
            hue: Some(RGBF32 {
                r: value.red,
                g: value.green,
                b: value.blue,
            }.to_hue()),
            color_on: (value.color_mode & 32) == 32,
        }
    }
}

fn esphome_state_to_igloo(value: EntityStateUpdateValue) -> Option<SubdeviceState> {
    Some(match value {
        EntityStateUpdateValue::Light(v) => SubdeviceState::Light(v.into()),
        _ => return None,
    })
}
