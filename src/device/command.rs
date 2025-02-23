use clap_derive::Args;
use tokio::task::JoinSet;

use crate::{cli::model::LightAction, map::{Selector, SelectorError, ZonesMap}, providers::IglooDevice};

#[derive(Debug, Clone)]
pub enum SubdeviceCommand {
    Light(LightAction),
}

#[derive(Debug, Clone, Default)]
pub struct LightState {
    pub on: bool,
    pub color_on: bool,
    pub color: Option<Color>,
    pub temp: Option<u32>,
    pub brightness: Option<u8>,
}

#[derive(Debug, Default, Clone, Args)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

pub struct ScopedSubdeviceCommand {
    pub selector: Selector,
    pub cmd: SubdeviceCommand
}

impl ScopedSubdeviceCommand {
    pub fn from_str(map: ZonesMap, selector_str: &str, cmd: SubdeviceCommand) -> Result<Self, SelectorError> {
        Ok(Self::new(Selector::from_str(map, selector_str)?, cmd))
    }

    pub fn new(selector: Selector, cmd: SubdeviceCommand) -> Self {
        Self { selector, cmd }
    }

    pub async fn execute(self) {
        match self.selector {
            Selector::All(map) => {
                let mut set = JoinSet::new();
                for (_, zone) in &*map {
                    for (_, dev_lock) in &**zone {
                        set.spawn(IglooDevice::execute_global_lock(dev_lock.clone(), self.cmd.clone()));
                    }
                }
                set.join_all().await;
			},
            Selector::Zone(zone) => {
                let mut set = JoinSet::new();
                for (_, dev_lock) in &*zone {
                    set.spawn(IglooDevice::execute_global_lock(dev_lock.clone(), self.cmd.clone()));
                }
                set.join_all().await;
			},
            Selector::Device(dev_lock) => {
                IglooDevice::execute_global_lock(dev_lock, self.cmd).await;
			},
            Selector::Subdevice(dev_lock, subdev_name) => {
                IglooDevice::execute_lock(dev_lock, self.cmd, &subdev_name).await;
			},
        }
    }
}

impl Selector {
    pub async fn get_light_state(&self) -> Option<LightState> {
        match self {
            Selector::All(map) => {
                let mut states = Vec::new();
                for (_, zone) in &**map {
                    for (_, dev_lock) in &**zone {
                        let dev = dev_lock.read().await;
                        states.push(IglooDevice::get_global_light_state(&dev));
                    }
                }
                LightState::avg(states)
			},
            Selector::Zone(zone) => {
                let mut states = Vec::new();
                for (_, dev_lock) in &**zone {
                    let dev = dev_lock.read().await;
                    states.push(IglooDevice::get_global_light_state(&dev));
                }
                LightState::avg(states)
			},
            Selector::Device(dev_lock) => {
                let dev = dev_lock.read().await;
                IglooDevice::get_global_light_state(&dev)
			},
            Selector::Subdevice(dev_lock, subdev_name) => {
                let dev = dev_lock.read().await;
                IglooDevice::get_light_state(&dev, subdev_name)
			},
        }
    }
}


impl LightState {
    /// im sorry
    pub fn avg(states: Vec<Option<Self>>) -> Option<Self> {
        let mut num = 0;
        let mut on_count = 0;
        let mut color_on_count = 0;

        let mut num_with_color = 0;
        let mut r_count = 0;
        let mut g_count = 0;
        let mut b_count = 0;

        let mut num_with_temp = 0;
        let mut temp_count = 0;

        let mut num_with_bright = 0;
        let mut bright_count = 0;

        for state_opt in states {
            if let Some(state) = state_opt {
                num += 1;
                on_count += state.on as u32;
                color_on_count += state.color_on as u32;

                if let Some(color) = state.color {
                    num_with_color += 1;
                    r_count += color.r as u32;
                    g_count += color.g as u32;
                    b_count += color.b as u32;
                }

                if let Some(temp) = state.temp {
                    num_with_temp += 1;
                    temp_count += temp;
                }

                if let Some(bright) = state.brightness {
                    num_with_bright += 1;
                    bright_count += bright as u32;
                }
            }
        }

        if num < 1 {
            return None
        }

        let mut me = Self {
            on: (on_count as f32 / num as f32) > 0.5,
            color_on: (color_on_count as f32 / num as f32) > 0.5,
            ..Default::default()
        };

        if num_with_color > 0 {
            me.color = Some(Color {
                r: (r_count as f32 / num_with_color as f32) as u8,
                g: (g_count as f32 / num_with_color as f32) as u8,
                b: (b_count as f32 / num_with_color as f32) as u8
            });
        }

        if num_with_temp > 0 {
            me.temp = Some(
                (temp_count as f32 / num_with_temp as f32) as u32
            );
        }

        if num_with_bright > 0 {
            me.brightness = Some(
                (bright_count as f32 / num_with_temp as f32) as u8
            );
        }

        Some(me)
    }
}
