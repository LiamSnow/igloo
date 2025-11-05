use super::{EntityRegister, add_entity_category, add_icon};
use async_trait::async_trait;
use igloo_interface::floe::FloeWriterDefault;

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesCameraResponse {
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
                crate::model::EntityType::Camera,
            )
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        Ok(())
    }
}
