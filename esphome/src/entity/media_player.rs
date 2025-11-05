use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use async_trait::async_trait;
use igloo_interface::{
    DESELECT_ENTITY, END_TRANSACTION, MediaState, Muted, Volume, WRITE_MEDIA_STATE, WRITE_MUTED,
    WRITE_VOLUME, floe::FloeWriterDefault,
};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesMediaPlayerResponse {
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
                crate::model::EntityType::MediaPlayer,
            )
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
        writer.volume(&(self.volume as f64)).await?;
        writer.muted(&self.muted).await?;
        writer.media_state(&self.state().as_igloo()).await
    }
}

fn media_state_to_command(state: &MediaState) -> api::MediaPlayerCommand {
    match state {
        MediaState::Playing => api::MediaPlayerCommand::Play,
        MediaState::Paused => api::MediaPlayerCommand::Pause,
        MediaState::Idle => api::MediaPlayerCommand::Stop,
        MediaState::Unknown => api::MediaPlayerCommand::Stop,
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::MediaPlayerCommandRequest {
        key,
        ..Default::default()
    };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_VOLUME => {
                let volume: Volume = borsh::from_slice(&payload)?;
                req.has_volume = true;
                req.volume = volume as f32;
            }

            WRITE_MUTED => {
                let muted: Muted = borsh::from_slice(&payload)?;
                req.has_command = true;
                req.command = if muted {
                    api::MediaPlayerCommand::Mute
                } else {
                    api::MediaPlayerCommand::Unmute
                }
                .into();
            }

            WRITE_MEDIA_STATE => {
                let state: MediaState = borsh::from_slice(&payload)?;
                req.has_command = true;
                req.command = media_state_to_command(&state).into();
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!(
                    "MediaPlayer got unexpected command {cmd_id} during transaction. Skipping.."
                );
            }
        }
    }

    device
        .send_msg(MessageType::MediaPlayerCommandRequest, &req)
        .await
}
