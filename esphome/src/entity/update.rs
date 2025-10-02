use crate::{api, entity::EntityUpdate};
use async_trait::async_trait;
use igloo_interface::FloeWriterDefault;
use serde_json::json;
use super::{add_entity_category, add_icon, add_device_class, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesUpdateResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::device::EntityType::Update)
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

        let content = json!({
            "title": self.title,
            "current_version": self.current_version,
            "latest_version": self.latest_version,
            "release_summary": self.release_summary,
            "release_url": self.release_url
        });

        writer.bool(&self.in_progress).await?;
        writer.text(&content.to_string()).await?;

        if self.has_progress {
            writer.float(&self.progress).await?;
        }

        Ok(())
    }
}
