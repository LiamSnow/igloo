use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use async_trait::async_trait;
use igloo_interface::{FloeWriterDefault, WRITE_TEXT, DESELECT_ENTITY, END_TRANSACTION};
use super::{add_entity_category, add_icon, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesSelectResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::model::EntityType::Select)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        writer.text_select().await?;
        writer.text_list(&self.options).await?;
        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::SelectStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.text(&self.state).await
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::SelectCommandRequest {
        key,
        state: String::new(),
    };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_TEXT => {
                let state: String = borsh::from_slice(&payload)?;
                req.state = state;
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!("Select got unexpected command {cmd_id} during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::SelectCommandRequest, &req)
        .await
}
