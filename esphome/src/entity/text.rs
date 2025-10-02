use crate::{api, entity::EntityUpdate};
use async_trait::async_trait;
use igloo_interface::{FloeWriterDefault, TextMode};
use super::{add_entity_category, add_icon, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesTextResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::device::EntityType::Text)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        writer.text_mode(&self.mode().as_igloo()).await?;
        writer.text_min_length(&self.min_length).await?;
        writer.text_max_length(&self.max_length).await?;
        writer.text_pattern(&self.pattern).await?;
        Ok(())
    }
}

impl api::TextMode {
    pub fn as_igloo(&self) -> TextMode {
        match self {
            api::TextMode::Text => TextMode::Text,
            api::TextMode::Password => TextMode::Password,
        }
    }
}

#[async_trait]
impl EntityUpdate for api::TextStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.text(&self.state).await
    }
}
