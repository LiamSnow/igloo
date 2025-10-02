use async_trait::async_trait;
use igloo_interface::FloeWriterDefault;
use super::{add_entity_category, add_icon, add_device_class, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesButtonResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::device::EntityType::Button)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        add_device_class(writer, self.device_class).await?;
        Ok(())
    }
}