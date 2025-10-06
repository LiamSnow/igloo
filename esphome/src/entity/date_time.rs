use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use async_trait::async_trait;
use igloo_interface::{
    FloeWriterDefault,
    WRITE_DATE_TIME, DESELECT_ENTITY, END_TRANSACTION
};
use super::{add_entity_category, add_icon, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesDateTimeResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::model::EntityType::DateTime)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::DateTimeStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.date_time(&self.epoch_seconds).await
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::DateTimeCommandRequest {
        key,
        epoch_seconds: 0,
    };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_DATE_TIME => {
                let epoch_seconds: u32 = borsh::from_slice(&payload)?;
                req.epoch_seconds = epoch_seconds;
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!("DateTime got unexpected command {cmd_id} during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::DateTimeCommandRequest, &req)
        .await
}
