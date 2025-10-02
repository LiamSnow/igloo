use crate::{api, entity::EntityUpdate};
use async_trait::async_trait;
use igloo_interface::{FloeWriterDefault, ValveState};
use super::{add_entity_category, add_icon, add_device_class, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesValveResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::device::EntityType::Valve)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        add_device_class(writer, self.device_class).await?;
        writer.valve().await?;
        Ok(())
    }
}

impl api::ValveOperation {
    fn as_igloo(&self) -> ValveState {
        match self {
            api::ValveOperation::Idle => ValveState::Idle,
            api::ValveOperation::IsOpening => ValveState::Opening,
            api::ValveOperation::IsClosing => ValveState::Closing,
        }
    }
}

#[async_trait]
impl EntityUpdate for api::ValveStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.position(&self.position).await?;
        writer
            .valve_state(&self.current_operation().as_igloo())
            .await
    }
}
