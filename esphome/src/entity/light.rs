use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use igloo_interface::{ColorMode, Component, types::IglooColor};

impl EntityRegister for crate::api::ListEntitiesLightResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(3);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        comps.push(Component::Light);
        comps
    }
}

pub fn kelvin_to_mireds(kelvin: i64) -> f64 {
    1_000_000. / kelvin as f64
}

pub fn mireds_to_kelvin(mireds: f64) -> u16 {
    (1_000_000. / mireds).round() as u16
}

impl EntityUpdate for api::LightStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn comps(&self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(5);

        comps.push(Component::Color(IglooColor {
            r: self.red as f64,
            g: self.green as f64,
            b: self.blue as f64,
        }));

        comps.push(Component::Dimmer(self.brightness as f64));
        comps.push(Component::Switch(self.state));
        comps.push(Component::ColorTemperature(
            mireds_to_kelvin(self.color_temperature as f64) as i64,
        ));

        // ON_OFF = 1 << 0;
        // BRIGHTNESS = 1 << 1;
        // WHITE = 1 << 2;
        // COLOR_TEMPERATURE = 1 << 3;
        // COLD_WARM_WHITE = 1 << 4;
        // RGB = 1 << 5;

        // TODO FIXME is this right? Lowk i don't get the other ones

        if self.color_mode & (1 << 5) != 0 {
            comps.push(Component::ColorMode(ColorMode::RGB));
        } else if self.color_mode & (1 << 3) != 0 {
            comps.push(Component::ColorMode(ColorMode::Temperature));
        }

        comps
    }
}

#[inline]
pub async fn process(
    device: &mut Device,
    key: u32,
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::LightCommandRequest {
        key,
        has_transition_length: true,
        transition_length: 0,
        ..Default::default()
    };

    for comp in comps {
        use Component::*;
        match comp {
            Color(color) => {
                req.has_rgb = true;
                req.red = color.r as f32;
                req.green = color.g as f32;
                req.blue = color.b as f32;
            }

            Dimmer(val) => {
                // req.has_color_brightness = true;
                // req.color_brightness = val;
                req.has_brightness = true;
                req.brightness = val as f32;

                req.has_state = true;
                req.state = val > 0.;
            }

            Switch(state) => {
                req.has_state = true;
                req.state = state;
            }

            ColorTemperature(temp_kelvin) => {
                req.has_color_temperature = true;
                req.color_temperature = kelvin_to_mireds(temp_kelvin) as f32;
            }

            ColorMode(mode) => {
                use igloo_interface::ColorMode::*;
                req.has_color_mode = true;
                req.color_mode = match mode {
                    RGB => 35,
                    Temperature => 11,
                };
            }

            comp => {
                println!(
                    "Light got unexpected component '{comp:?}' during transaction. Skipping.."
                );
            }
        }
    }

    device
        .send_msg(MessageType::LightCommandRequest, &req)
        .await
}
