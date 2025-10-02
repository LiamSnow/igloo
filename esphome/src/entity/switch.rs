use super::{EntityRegister, add_device_class, add_entity_category, add_icon};
use crate::{api, entity::EntityUpdate};
use async_trait::async_trait;
use igloo_interface::FloeWriterDefault;

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesSwitchResponse {
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
                crate::device::EntityType::Switch,
            )
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        add_device_class(writer, self.device_class).await?;
        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::SwitchStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.switch(&self.state).await
    }
}
