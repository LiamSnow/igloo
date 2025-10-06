use super::{EntityRegister, add_device_class, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use async_trait::async_trait;
use igloo_interface::{FloeWriterDefault, WRITE_SWITCH, DESELECT_ENTITY, END_TRANSACTION};

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
                crate::model::EntityType::Switch,
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

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::SwitchCommandRequest {
        key,
        state: false,
    };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_SWITCH => {
                let state: bool = borsh::from_slice(&payload)?;
                req.state = state;
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!("Switch got unexpected command {cmd_id} during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::SwitchCommandRequest, &req)
        .await
}
