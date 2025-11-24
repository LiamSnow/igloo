use super::{EntityRegister, add_device_class, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use igloo_interface::{Component, CoverState};

impl EntityRegister for api::ListEntitiesCoverResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(4);
        comps.push(Component::Cover);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        add_device_class(&mut comps, self.device_class);
        comps
    }
}

impl api::CoverOperation {
    pub fn as_igloo(&self) -> CoverState {
        match self {
            api::CoverOperation::Idle => CoverState::Idle,
            api::CoverOperation::IsOpening => CoverState::Opening,
            api::CoverOperation::IsClosing => CoverState::Closing,
        }
    }
}

impl EntityUpdate for api::CoverStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn comps(&self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(3);
        comps.push(Component::Position(self.position as f64));
        comps.push(Component::Tilt(self.tilt as f64));
        comps.push(Component::CoverState(self.current_operation().as_igloo()));
        comps
    }
}

fn cover_state_to_command(state: &CoverState) -> api::LegacyCoverCommand {
    match state {
        CoverState::Open => api::LegacyCoverCommand::Open,
        CoverState::Closed => api::LegacyCoverCommand::Close,
        CoverState::Opening => api::LegacyCoverCommand::Open,
        CoverState::Closing => api::LegacyCoverCommand::Close,
        CoverState::Stopped => api::LegacyCoverCommand::Stop,
        CoverState::Idle => api::LegacyCoverCommand::Stop,
    }
}

#[inline]
pub async fn process(
    device: &mut Device,
    key: u32,
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::CoverCommandRequest {
        key,
        ..Default::default()
    };

    for comp in comps {
        use Component::*;
        match comp {
            Position(position) => {
                req.has_position = true;
                req.position = position as f32;
            }

            Tilt(tilt) => {
                req.has_tilt = true;
                req.tilt = tilt as f32;
            }

            CoverState(state) => {
                req.has_legacy_command = true;
                req.legacy_command = cover_state_to_command(&state).into();
            }

            comp => {
                println!(
                    "Cover got unexpected component '{comp:?}' during transaction. Skipping.."
                );
            }
        }
    }

    device
        .send_msg(MessageType::CoverCommandRequest, &req)
        .await
}
