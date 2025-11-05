use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use async_trait::async_trait;
use igloo_interface::{
    DESELECT_ENTITY, Date, END_TRANSACTION, WRITE_DATE, floe::FloeWriterDefault,
};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesDateResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::model::EntityType::Date)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::DateStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer
            .date(&Date {
                year: self.year as u16,
                month: self.month as u8,
                day: self.day as u8,
            })
            .await
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::DateCommandRequest {
        key,
        year: 0,
        month: 0,
        day: 0,
    };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_DATE => {
                let date: Date = borsh::from_slice(&payload)?;
                req.year = date.year as u32;
                req.month = date.month as u32;
                req.day = date.day as u32;
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!("Date got unexpected command {cmd_id} during transaction. Skipping..");
            }
        }
    }

    device.send_msg(MessageType::DateCommandRequest, &req).await
}
