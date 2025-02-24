use core::str;
use std::{error::Error, time::Duration};

use esphomebridge_rs::{api::{self, LightStateResponse}, device::ESPHomeDevice, entity::EntityStateUpdateValue, error::DeviceError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{sync::mpsc::{self, Receiver, Sender}, time::timeout};

use crate::{cli::model::LightAction, command::{Color, LightState, RackSubdeviceCommand, SubdeviceCommand, SubdeviceState, SubdeviceStateUpdate}};

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

// matches aioesphomeapi
pub const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(20);
pub const KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(90);

pub fn new(config: ESPHomeDeviceConfig, dev_id: usize, update: Sender<SubdeviceStateUpdate>) -> Result<Sender<RackSubdeviceCommand>, Box<dyn Error>> {
    let (cmd_tx, cmd_rx) = mpsc::channel::<RackSubdeviceCommand>(5);
    let dev = make_device(config)?;
    tokio::spawn(spawn(dev, dev_id, cmd_rx, update));
    Ok(cmd_tx)
}

async fn spawn(mut dev: ESPHomeDevice, dev_id: usize, mut cmd_rx: Receiver<RackSubdeviceCommand>, update_tx: Sender<SubdeviceStateUpdate>) -> Result<(), ESPHomeError> {
    dev.connect().await?;

    println!("{dev_id} connected");

    //push state up
    let mut update_rx = dev.subscribe_states(5).await?;
    tokio::spawn(async move {
        while let Some(update) = update_rx.recv().await {
            if let Some(value) = esphome_state_to_igloo(update.value) {
                //TODO include type and category?
                let res = update_tx.send(SubdeviceStateUpdate {
                    dev_id,
                    subdev_name: update.subdev_name,
                    value
                }).await;

                //TODO
                if let Err(e) = res {
                    println!("esphome device state update error: {e}");
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
            },
            Err(_) => {
                //TODO log error
                dev.process_incoming().await?;
            }
            Ok(None) => {
                println!("{dev_id} Cmd channel closed"); //TODO
                break;
            }
        }
    }

    Ok(())
}

async fn handle_cmd(dev: &mut ESPHomeDevice, cmd: RackSubdeviceCommand) -> Result<(), ESPHomeError> {
    match cmd.cmd {
        SubdeviceCommand::Light(light_cmd) => {
            if let Some(subdev_name) = cmd.subdev_name {
                let entity = dev.entities.light.get(&subdev_name)
                    .ok_or(ESPHomeError::InvalidSubdevice(subdev_name))?;
                dev.light_command(&light_cmd.to_esphome(entity.key)).await?;
            }
            else {
                let mut esp_cmd = light_cmd.clone().to_esphome(0);
                dev.light_command_global(&mut esp_cmd).await?;
            }
        },
        SubdeviceCommand::Switch(state) => {
            if let Some(subdev_name) = cmd.subdev_name {
                let entity = dev.entities.switch.get(&subdev_name)
                    .ok_or(ESPHomeError::InvalidSubdevice(subdev_name))?;
                dev.switch_command(&api::SwitchCommandRequest {
                    key: entity.key,
                    state: state.into(),
                }).await?;
            }
            else {
                let mut esp_cmd = api::SwitchCommandRequest { key: 0, state: state.into() };
                dev.switch_command_global(&mut esp_cmd).await?;
            }
        }
    }

    Ok(())
}

fn make_device(config: ESPHomeDeviceConfig) -> Result<ESPHomeDevice, ESPHomeError> {
    let ip = if config.ip.contains(':') { config.ip } else { config.ip + ":6053" };
    if let Some(noise_psk) = config.noise_psk {
        Ok(ESPHomeDevice::new_noise(ip, noise_psk))
    }
    else if let Some(password) = config.password {
        Ok(ESPHomeDevice::new_plain(ip, password))
    }
    else {
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

impl From<LightStateResponse> for LightState {
    fn from(value: LightStateResponse) -> Self {
        Self {
            on: value.state,
            temp: Some(value.color_temperature as u32),
            brightness: Some((value.brightness * 100.) as u8),
            color: Some(Color {
                r: (value.red * 255.) as u8,
                g: (value.green * 255.) as u8,
                b: (value.blue * 255.) as u8,
            }),
            color_on: value.color_mode == 35
        }
    }
}

fn esphome_state_to_igloo(value: EntityStateUpdateValue) -> Option<SubdeviceState> {
    Some(match value {
        EntityStateUpdateValue::Light(v) => SubdeviceState::Light(v.into()),
        _ => return None
    })
}
