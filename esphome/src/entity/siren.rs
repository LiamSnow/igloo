use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use async_trait::async_trait;
use igloo_interface::{
    FloeWriterDefault,
    WRITE_SWITCH, WRITE_TEXT, WRITE_VOLUME, WRITE_UINT,
    DESELECT_ENTITY, END_TRANSACTION
};
use super::{add_entity_category, add_icon, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesSirenResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::model::EntityType::Siren)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        writer.text_select().await?;
        writer.text_list(&self.tones).await?;
        writer.siren().await?;
        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::SirenStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.bool(&self.state).await
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::SirenCommandRequest {
        key,
        ..Default::default()
    };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_SWITCH => {
                let state: bool = borsh::from_slice(&payload)?;
                req.has_state = true;
                req.state = state;
            }

            WRITE_TEXT => {
                let tone: String = borsh::from_slice(&payload)?;
                req.has_tone = true;
                req.tone = tone;
            }

            WRITE_VOLUME => {
                let volume: f32 = borsh::from_slice(&payload)?;
                req.has_volume = true;
                req.volume = volume;
            }

            WRITE_UINT => {
                let duration: u32 = borsh::from_slice(&payload)?;
                req.has_duration = true;
                req.duration = duration;
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!("Siren got unexpected command {cmd_id} during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::SirenCommandRequest, &req)
        .await
}
