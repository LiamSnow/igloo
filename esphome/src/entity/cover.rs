use async_trait::async_trait;
use igloo_interface::{CoverState, FloeWriterDefault};

use crate::{api, entity::EntityUpdate};
use super::{add_entity_category, add_icon, add_device_class, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesCoverResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::device::EntityType::Cover)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        add_device_class(writer, self.device_class).await?;
        writer.sensor().await?;
        Ok(())
    }
}

impl api::CoverOperation {
    pub fn as_igloo(&self) -> CoverState {
        match self {
            api::CoverOperation::Idle => CoverState::Idle,
            api::CoverOperation::IsOpening => CoverState::Opening,
            api::CoverOperation::IsClosing => CoverState::Closing,
        }
    }
}

#[async_trait]
impl EntityUpdate for api::CoverStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.position(&self.position).await?;
        writer.tilt(&self.tilt).await?;
        writer
            .cover_state(&self.current_operation().as_igloo())
            .await
    }
}
