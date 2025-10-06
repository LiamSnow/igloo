use super::{EntityRegister, add_device_class, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use async_trait::async_trait;
use igloo_interface::{
    DESELECT_ENTITY, END_TRANSACTION, FloeWriterDefault, ValveState, WRITE_POSITION,
    WRITE_VALVE_STATE,
};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesValveResponse {
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
                crate::model::EntityType::Valve,
            )
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

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::ValveCommandRequest {
        key,
        ..Default::default()
    };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_POSITION => {
                let position: f32 = borsh::from_slice(&payload)?;
                req.has_position = true;
                req.position = position;
            }

            WRITE_VALVE_STATE => {
                let state: ValveState = borsh::from_slice(&payload)?;
                match state {
                    ValveState::Idle => req.stop = true,
                    ValveState::Opening | ValveState::Closing => {}
                }
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!("Valve got unexpected command {cmd_id} during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::ValveCommandRequest, &req)
        .await
}
