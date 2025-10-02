use crate::{api, entity::EntityUpdate};
use async_trait::async_trait;
use igloo_interface::FloeWriterDefault;
use super::{add_entity_category, add_icon, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesSelectResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::device::EntityType::Select)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        writer.text_select().await?;
        writer.text_list(&self.options).await?;
        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::SelectStateResponse {
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
