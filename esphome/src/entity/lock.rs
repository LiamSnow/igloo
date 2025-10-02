use async_trait::async_trait;
use igloo_interface::{FloeWriterDefault, LockState};

use crate::{api, entity::EntityUpdate};
use super::{add_entity_category, add_icon, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesLockResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::device::EntityType::Lock)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        writer.text(&self.code_format).await?;
        Ok(())
    }
}

impl api::LockState {
    fn as_igloo(&self) -> LockState {
        match self {
            api::LockState::None => LockState::Unknown,
            api::LockState::Locked => LockState::Locked,
            api::LockState::Unlocked => LockState::Unlocked,
            api::LockState::Jammed => LockState::Jammed,
            api::LockState::Locking => LockState::Locking,
            api::LockState::Unlocking => LockState::Unlocking,
        }
    }
}

#[async_trait]
impl EntityUpdate for api::LockStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.lock_state(&self.state().as_igloo()).await
    }
}
