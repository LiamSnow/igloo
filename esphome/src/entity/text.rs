use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use async_trait::async_trait;
use igloo_interface::{FloeWriterDefault, TextMode, WRITE_TEXT, DESELECT_ENTITY, END_TRANSACTION};
use super::{add_entity_category, add_icon, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesTextResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::model::EntityType::Text)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        writer.text_mode(&self.mode().as_igloo()).await?;
        writer.text_min_length(&self.min_length).await?;
        writer.text_max_length(&self.max_length).await?;
        writer.text_pattern(&self.pattern).await?;
        Ok(())
    }
}

impl api::TextMode {
    pub fn as_igloo(&self) -> TextMode {
        match self {
            api::TextMode::Text => TextMode::Text,
            api::TextMode::Password => TextMode::Password,
        }
    }
}

#[async_trait]
impl EntityUpdate for api::TextStateResponse {
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
    let mut req = api::TextCommandRequest {
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
                println!("Text got unexpected command {cmd_id} during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::TextCommandRequest, &req)
        .await
}
