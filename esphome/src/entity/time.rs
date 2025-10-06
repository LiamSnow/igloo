use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use async_trait::async_trait;
use igloo_interface::{DESELECT_ENTITY, END_TRANSACTION, FloeWriterDefault, Time, WRITE_TIME};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesTimeResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::model::EntityType::Time)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::TimeStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer
            .time(&Time {
                hour: self.hour as u8,
                minute: self.minute as u8,
                second: self.second as u8,
            })
            .await
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::TimeCommandRequest {
        key,
        hour: 0,
        minute: 0,
        second: 0,
    };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_TIME => {
                let time: Time = borsh::from_slice(&payload)?;
                req.hour = time.hour as u32;
                req.minute = time.minute as u32;
                req.second = time.second as u32;
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!("Time got unexpected command {cmd_id} during transaction. Skipping..");
            }
        }
    }

    device.send_msg(MessageType::TimeCommandRequest, &req).await
}
