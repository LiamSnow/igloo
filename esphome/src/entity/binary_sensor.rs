use async_trait::async_trait;
use igloo_interface::floe::FloeWriterDefault;

use crate::{api, entity::EntityUpdate, model::EntityType};

use super::{EntityRegister, add_device_class, add_entity_category, add_icon};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesBinarySensorResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, EntityType::BinarySensor)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        add_device_class(writer, self.device_class).await?;
        writer.sensor().await?;
        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::BinarySensorStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.boolean(&self.state).await
    }
}
