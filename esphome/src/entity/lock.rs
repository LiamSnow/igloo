use async_trait::async_trait;
use igloo_interface::{
    DESELECT_ENTITY, END_TRANSACTION, FloeWriterDefault, LockState, WRITE_LOCK_STATE, WRITE_TEXT,
};

use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesLockResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::model::EntityType::Lock)
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

fn lock_state_to_command(state: &LockState) -> api::LockCommand {
    match state {
        LockState::Locked => api::LockCommand::LockLock,
        LockState::Unlocked => api::LockCommand::LockUnlock,
        LockState::Jammed => api::LockCommand::LockUnlock,
        LockState::Locking => api::LockCommand::LockLock,
        LockState::Unlocking => api::LockCommand::LockUnlock,
        LockState::Unknown => api::LockCommand::LockUnlock,
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::LockCommandRequest {
        key,
        command: api::LockCommand::LockLock.into(),
        has_code: false,
        code: String::new(),
    };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_TEXT => {
                let code: String = borsh::from_slice(&payload)?;
                req.has_code = true;
                req.code = code;
            }

            WRITE_LOCK_STATE => {
                let state: LockState = borsh::from_slice(&payload)?;
                req.command = lock_state_to_command(&state).into();
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!("Lock got unexpected command {cmd_id} during transaction. Skipping..");
            }
        }
    }

    device.send_msg(MessageType::LockCommandRequest, &req).await
}
