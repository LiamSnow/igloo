use async_trait::async_trait;
use igloo_interface::{FloeWriterDefault, DESELECT_ENTITY, END_TRANSACTION};
use super::{add_entity_category, add_icon, add_device_class, EntityRegister};
use crate::{
    api,
    device::{Device, DeviceError},
    model::MessageType,
};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesButtonResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::model::EntityType::Button)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        add_device_class(writer, self.device_class).await?;
        Ok(())
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let req = api::ButtonCommandRequest { key };

    for (cmd_id, _payload) in commands {
        match cmd_id {
            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!("Button got unexpected command {cmd_id} during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::ButtonCommandRequest, &req)
        .await
}