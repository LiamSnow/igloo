use clap_derive::Args;
use esphomebridge_rs::api;

use crate::cli::model::LightAction;

#[derive(Debug, Clone)]
pub enum DeviceCommand {
    Connect,
    Light(LightAction),
}

#[derive(Debug, Default, Clone, Args)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
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
                cmd.color_temperature = temp;
            },
            LightAction::Brightness { brightness } => {
                cmd.has_brightness = true;
                cmd.brightness = brightness;
            },
        }

        cmd
    }
}
