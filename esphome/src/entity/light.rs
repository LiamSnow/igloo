use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use async_trait::async_trait;
use igloo_interface::{
    Color, ColorMode, ColorTemperature, DESELECT_ENTITY, Dimmer, END_TRANSACTION, Switch,
    WRITE_COLOR, WRITE_COLOR_MODE, WRITE_COLOR_TEMPERATURE, WRITE_DIMMER, WRITE_SWITCH,
    floe::FloeWriterDefault,
};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesLightResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(
                writer,
                &self.name,
                self.key,
                crate::model::EntityType::Light,
            )
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        writer.light().await?;
        Ok(())
    }
}

pub fn kelvin_to_mireds(kelvin: i64) -> f64 {
    1_000_000. / kelvin as f64
}

pub fn mireds_to_kelvin(mireds: f64) -> u16 {
    (1_000_000. / mireds).round() as u16
}

#[async_trait]
impl EntityUpdate for api::LightStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer
            .color(&Color {
                r: self.red as f64,
                g: self.green as f64,
                b: self.blue as f64,
            })
            .await?;
        writer.dimmer(&(self.brightness as f64)).await?;
        writer.switch(&self.state).await?;
        writer
            .color_temperature(&(mireds_to_kelvin(self.color_temperature as f64) as i64))
            .await?;

        // ON_OFF = 1 << 0;
        // BRIGHTNESS = 1 << 1;
        // WHITE = 1 << 2;
        // COLOR_TEMPERATURE = 1 << 3;
        // COLD_WARM_WHITE = 1 << 4;
        // RGB = 1 << 5;

        // TODO FIXME is this right? Lowk i don't get the other ones

        if self.color_mode & (1 << 5) != 0 {
            writer.color_mode(&ColorMode::RGB).await?;
        } else if self.color_mode & (1 << 3) != 0 {
            writer.color_mode(&ColorMode::Temperature).await?;
        }

        Ok(())
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::LightCommandRequest {
        key,
        has_transition_length: true,
        transition_length: 0,
        ..Default::default()
    };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_COLOR => {
                let color: Color = borsh::from_slice(&payload)?;
                req.has_rgb = true;
                req.red = color.r as f32;
                req.green = color.g as f32;
                req.blue = color.b as f32;
            }

            WRITE_DIMMER => {
                let val: Dimmer = borsh::from_slice(&payload)?;
                // req.has_color_brightness = true;
                // req.color_brightness = val;
                req.has_brightness = true;
                req.brightness = val as f32;

                req.has_state = true;
                req.state = val > 0.;
            }

            WRITE_SWITCH => {
                let state: Switch = borsh::from_slice(&payload)?;
                req.has_state = true;
                req.state = state;
            }

            WRITE_COLOR_TEMPERATURE => {
                let temp_kelvin: ColorTemperature = borsh::from_slice(&payload)?;
                req.has_color_temperature = true;
                req.color_temperature = kelvin_to_mireds(temp_kelvin) as f32;
            }

            WRITE_COLOR_MODE => {
                let mode: ColorMode = borsh::from_slice(&payload)?;
                req.has_color_mode = true;
                req.color_mode = match mode {
                    ColorMode::RGB => 35,
                    ColorMode::Temperature => 11,
                };
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!("Light got unexpected command {cmd_id} during transactino. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::LightCommandRequest, &req)
        .await
}
