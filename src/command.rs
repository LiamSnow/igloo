use clap_derive::Args;
use serde::Serialize;

use crate::cli::model::{LightAction, SwitchState};

#[derive(Debug, Clone)]
pub enum SubdeviceCommand {
    Light(LightAction),
    Switch(SwitchState)
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

#[derive(Debug, Default, Clone, Args, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

pub struct RackSubdeviceCommand {
    pub subdev_name: Option<String>,
    pub cmd: SubdeviceCommand
}

impl LightState {
    /// im sorry
    pub fn avg(states: Vec<&Self>) -> Self {
        let num = states.len();
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

        for state in states {
            on_count += state.on as u32;
            color_on_count += state.color_on as u32;

            if let Some(color) = &state.color {
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

        me
    }
}
