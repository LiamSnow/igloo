use clap_derive::Args;
use serde::Serialize;

use crate::cli::model::{LightAction, SwitchState};

#[derive(Debug, Clone)]
pub enum SubdeviceCommand {
    Light(LightAction),
    Switch(SwitchState)
}

impl From<LightAction> for SubdeviceCommand {
    fn from(value: LightAction) -> Self {
        Self::Light(value)
    }
}

impl From<SwitchState> for SubdeviceCommand {
    fn from(value: SwitchState) -> Self {
        Self::Switch(value)
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum SubdeviceState {
    Light(LightState),
    Switch(SwitchState)
}

#[derive(Debug, Clone)]
pub struct SubdeviceStateUpdate {
    pub dev_id: usize,
    pub subdev_name: String,
    pub value: SubdeviceState
}

impl From<LightState> for SubdeviceState {
    fn from(value: LightState) -> Self {
        Self::Light(value)
    }
}

impl From<SwitchState> for SubdeviceState {
    fn from(value: SwitchState) -> Self {
        Self::Switch(value)
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LightState {
    pub on: bool,
    pub color_on: bool,
    pub color: Option<Color>,
    pub temp: Option<u32>,
    pub brightness: Option<u8>,
}

#[derive(Debug, Default, Clone, Args, Serialize, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    /// Fast HSL to RGB with S=100%, L=50%, and hue=0-255
    pub fn from_hue8(hue: u8) -> Self {
        Color {
            r: match hue {
                0..=42 => 255,
                43..=84 => (85 - hue) * 6,
                85..=169 => 0,
                170..=212 => (hue - 170) * 6,
                _ => 255,
            },
            g: match hue {
                0..=42 => hue * 6,
                43..=127 => 255,
                128..=169 => (170 - hue) * 6,
                _ => 0,
            },
            b: match hue {
                0..=84 => 0,
                85..=127 => (hue - 85) * 6,
                128..=212 => 255,
                _ => (255 - hue) * 6,
            }
        }
    }
}

//TODO name this
pub struct RackSubdeviceCommand {
    pub subdev_name: Option<String>,
    pub cmd: SubdeviceCommand
}

impl LightState {
    /// Average a set of colors
    /// Only avgs Some colors, if there are no colors it returns None (same for temp and bri)
    pub fn avg(states: Vec<&Self>) -> Self {
        let num = states.len();
        let (mut on_count, mut color_on_sum) = (0, 0);
        let (mut total_color, mut color_sum) = (0,(0,0,0));
        let (mut total_temp, mut temp_sum) = (0, 0);
        let (mut total_bright, mut bright_sum) = (0, 0);

        for state in states {
            on_count += state.on as u32;
            color_on_sum += state.color_on as u32;
            if let Some(color) = &state.color {
                total_color += 1;
                color_sum.0 += color.r as u32;
                color_sum.1 += color.g as u32;
                color_sum.2 += color.b as u32;
            }
            if let Some(temp) = state.temp {
                total_temp += 1;
                temp_sum += temp;
            }
            if let Some(bright) = state.brightness {
                total_bright += 1;
                bright_sum += bright as u32;
            }
        }

        Self {
            on: (on_count as f32 / num as f32) > 0.5,
            color_on: (color_on_sum as f32 / num as f32) > 0.5,
            color: if total_color > 0 { Some(Color{
                r: (color_sum.0 as f32 / total_color as f32) as u8,
                g: (color_sum.1 as f32 / total_color as f32) as u8,
                b: (color_sum.2 as f32 / total_color as f32) as u8
            })} else { None },
            temp: if total_temp > 0 {
                Some((temp_sum as f32 / total_temp as f32) as u32)
            } else { None },
            brightness: if total_bright > 0 {
                Some((bright_sum as f32 / total_temp as f32) as u8)
            } else { None },
        }
    }
}
