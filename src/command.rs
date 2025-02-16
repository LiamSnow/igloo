use esphomebridge_rs::api;

#[derive(Debug, Clone)]
pub enum IglooCommand {
    RGBLightCommand(RGBLightCommand),
    CTLightCommand(CTLightCommand),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

#[derive(Debug, Clone)]
pub struct RGBLightCommand {
    pub state: Option<bool>,
    pub brightness: Option<f32>,
    pub color: Option<Color>,
}

#[derive(Debug, Clone)]
pub struct CTLightCommand {
    pub state: Option<bool>,
    pub brightness: Option<f32>,
    pub temp: Option<f32>,
}

impl RGBLightCommand {
    pub fn to_esphome(self, key: u32) -> api::LightCommandRequest {
        api::LightCommandRequest {
            key,
            has_state: self.state.is_some(),
            state: self.state.unwrap_or_default(),
            has_brightness: self.brightness.is_some(), //TODO remove?
            brightness: self.brightness.unwrap_or_default(),
            has_color_mode: false,
            color_mode: 0,
            has_color_brightness: self.brightness.is_some(),
            color_brightness: self.brightness.unwrap_or_default(),
            has_rgb: self.color.is_some(),
            //RGB are relative values (IE red % = red / (red + blue + green))
            red: self.color.unwrap_or_default().r as f32, //so we dont need to / 255
            green: self.color.unwrap_or_default().g as f32,
            blue: self.color.unwrap_or_default().b as f32,
            has_white: false,
            white: 0.,
            has_color_temperature: false,
            color_temperature: 6536.,
            has_cold_white: false,
            cold_white: 1.,
            has_warm_white: false,
            warm_white: 0.,
            has_transition_length: true,
            transition_length: 0,
            has_flash_length: false,
            flash_length: 0,
            has_effect: false,
            effect: "".to_string()
        }
    }
}

impl CTLightCommand {
    pub fn to_esphome(self, key: u32) -> api::LightCommandRequest {
        api::LightCommandRequest {
            key,
            has_state: self.state.is_some(),
            state: self.state.unwrap_or_default(),
            has_brightness: self.brightness.is_some(),
            brightness: self.brightness.unwrap_or_default(),
            has_color_mode: false,
            color_mode: 0,
            has_color_brightness: false,
            color_brightness: 1.,
            has_rgb: false,
            red: 0.,
            blue: 0.,
            green: 0.,
            has_white: false,
            white: 0.,
            has_color_temperature: self.temp.is_some(),
            color_temperature: self.temp.unwrap_or_default(),
            has_cold_white: false,
            cold_white: 1.,
            has_warm_white: false,
            warm_white: 0.,
            has_transition_length: true,
            transition_length: 0,
            has_flash_length: false,
            flash_length: 0,
            has_effect: false,
            effect: "".to_string()
        }
    }
}
