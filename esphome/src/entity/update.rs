use super::{EntityRegister, add_device_class, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
};
use async_trait::async_trait;
use igloo_interface::floe::FloeWriterDefault;

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesUpdateResponse {
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
                crate::model::EntityType::Update,
            )
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        add_device_class(writer, self.device_class).await?;
        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::UpdateStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        // FIXME I think the best way to handle update is by making
        // more entities for a clearer representation
        // But maybe this is good for reducing less used entities IDK

        let content = format!(
            "title:{},current_version:{},latest_version:{},release_summary:{},release_url:{}",
            self.title,
            self.current_version,
            self.latest_version,
            self.release_summary,
            self.release_url
        );

        writer.boolean(&self.in_progress).await?;
        writer.text(&content.to_string()).await?;

        if self.has_progress {
            writer.real(&(self.progress as f64)).await?;
        }

        Ok(())
    }
}

pub async fn process(
    _device: &mut Device,
    _key: u32,
    _commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    eprintln!("ESPHOME UPDATE ENTITY IS NOT IMPLEMENTED");
    Ok(())

    // TODO how should we be handling this?
    // Lowkey I dont think it should even be in the ECS
    // and just custom commands?

    // Maybe make a trigger entity?
    // let req = api::UpdateCommandRequest {
    //     key,
    //     command: api::UpdateCommand::Update.into(),
    // };

    // for (cmd_id, _payload) in commands {
    //     match cmd_id {
    //         DESELECT_ENTITY | END_TRANSACTION => {
    //             unreachable!();
    //         }

    //         _ => {
    //             println!("Update got unexpected command {cmd_id} during transaction. Skipping..");
    //         }
    //     }
    // }

    // device
    //     .send_msg(MessageType::UpdateCommandRequest, &req)
    //     .await
}
