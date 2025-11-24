use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use igloo_interface::{Component, MediaState};

impl EntityRegister for api::ListEntitiesMediaPlayerResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(2);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        comps
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

impl EntityUpdate for api::MediaPlayerStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn comps(&self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(3);
        comps.push(Component::Volume(self.volume as f64));
        comps.push(Component::Muted(self.muted));
        comps.push(Component::MediaState(self.state().as_igloo()));
        comps
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
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::MediaPlayerCommandRequest {
        key,
        ..Default::default()
    };

    for comp in comps {
        use Component::*;
        match comp {
            Volume(volume) => {
                req.has_volume = true;
                req.volume = volume as f32;
            }

            Muted(muted) => {
                req.has_command = true;
                req.command = if muted {
                    api::MediaPlayerCommand::Mute
                } else {
                    api::MediaPlayerCommand::Unmute
                }
                .into();
            }

            MediaState(state) => {
                req.has_command = true;
                req.command = media_state_to_command(&state).into();
            }

            comp => {
                println!(
                    "MediaPlayer got unexpected component '{comp:?}' during transaction. Skipping.."
                );
            }
        }
    }

    device
        .send_msg(MessageType::MediaPlayerCommandRequest, &req)
        .await
}
