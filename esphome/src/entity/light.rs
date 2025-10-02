use super::{EntityRegister, add_entity_category, add_icon};
use crate::{api, entity::EntityUpdate};
use async_trait::async_trait;
use igloo_interface::{Color, ColorMode, FloeWriterDefault};

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
                crate::device::EntityType::Light,
            )
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        writer.light().await?;
        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::LightStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer
            .color(&Color {
                r: (self.red * 255.) as u8,
                g: (self.green * 255.) as u8,
                b: (self.blue * 255.) as u8,
            })
            .await?;
        writer.dimmer(&self.brightness).await?;
        writer.switch(&self.state).await?;
        writer
            .color_temperature(&(self.color_temperature as u16))
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
