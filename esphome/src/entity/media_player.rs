use crate::{api, entity::EntityUpdate};
use async_trait::async_trait;
use igloo_interface::{FloeWriterDefault, MediaState};
use super::{add_entity_category, add_icon, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesMediaPlayerResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::device::EntityType::MediaPlayer)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        Ok(())
    }
}

impl api::MediaPlayerState {
    fn as_igloo(&self) -> MediaState {
        match self {
            api::MediaPlayerState::None => MediaState::Unknown,
            api::MediaPlayerState::Idle => MediaState::Idle,
            api::MediaPlayerState::Playing => MediaState::Playing,
            api::MediaPlayerState::Paused => MediaState::Paused,
        }
    }
}

#[async_trait]
impl EntityUpdate for api::MediaPlayerStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.volume(&self.volume).await?;
        writer.muted(&self.muted).await?;
        writer.media_state(&self.state().as_igloo()).await
    }
}
