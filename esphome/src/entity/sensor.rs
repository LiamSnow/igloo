use super::{
    EntityRegister, add_device_class, add_entity_category, add_icon, add_sensor_state_class,
    add_unit,
};
use crate::{api, entity::EntityUpdate};
use async_trait::async_trait;
use igloo_interface::FloeWriterDefault;

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesSensorResponse {
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
                crate::model::EntityType::Sensor,
            )
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_sensor_state_class(writer, self.state_class()).await?;
        add_icon(writer, &self.icon).await?;
        add_device_class(writer, self.device_class).await?;
        add_unit(writer, self.unit_of_measurement).await?;
        writer.sensor().await?;
        writer.accuracy_decimals(&self.accuracy_decimals).await?;
        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::SensorStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.float(&self.state).await
    }
}
