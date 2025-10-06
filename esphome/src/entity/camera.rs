use async_trait::async_trait;
use igloo_interface::FloeWriterDefault;
use super::{add_entity_category, add_icon, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesCameraResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::model::EntityType::Camera)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        Ok(())
    }
}