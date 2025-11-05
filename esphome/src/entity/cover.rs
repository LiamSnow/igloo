use async_trait::async_trait;
use igloo_interface::{
    CoverState, DESELECT_ENTITY, END_TRANSACTION, Position, Tilt, WRITE_COVER_STATE,
    WRITE_POSITION, WRITE_TILT, floe::FloeWriterDefault,
};

use super::{EntityRegister, add_device_class, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesCoverResponse {
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
                crate::model::EntityType::Cover,
            )
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
        writer.position(&(self.position as f64)).await?;
        writer.tilt(&(self.tilt as f64)).await?;
        writer
            .cover_state(&self.current_operation().as_igloo())
            .await
    }
}

fn cover_state_to_command(state: &CoverState) -> api::LegacyCoverCommand {
    match state {
        CoverState::Open => api::LegacyCoverCommand::Open,
        CoverState::Closed => api::LegacyCoverCommand::Close,
        CoverState::Opening => api::LegacyCoverCommand::Open,
        CoverState::Closing => api::LegacyCoverCommand::Close,
        CoverState::Stopped => api::LegacyCoverCommand::Stop,
        CoverState::Idle => api::LegacyCoverCommand::Stop,
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::CoverCommandRequest {
        key,
        ..Default::default()
    };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_POSITION => {
                let position: Position = borsh::from_slice(&payload)?;
                req.has_position = true;
                req.position = position as f32;
            }

            WRITE_TILT => {
                let tilt: Tilt = borsh::from_slice(&payload)?;
                req.has_tilt = true;
                req.tilt = tilt as f32;
            }

            WRITE_COVER_STATE => {
                let state: CoverState = borsh::from_slice(&payload)?;
                req.has_legacy_command = true;
                req.legacy_command = cover_state_to_command(&state).into();
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!("Cover got unexpected command {cmd_id} during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::CoverCommandRequest, &req)
        .await
}
