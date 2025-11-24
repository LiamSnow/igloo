use super::{EntityRegister, add_device_class, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use igloo_interface::{Component, ValveState};

impl EntityRegister for api::ListEntitiesValveResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(4);
        comps.push(Component::Valve);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        add_device_class(&mut comps, self.device_class);
        comps
    }
}

impl api::ValveOperation {
    fn as_igloo(&self) -> ValveState {
        match self {
            api::ValveOperation::Idle => ValveState::Idle,
            api::ValveOperation::IsOpening => ValveState::Opening,
            api::ValveOperation::IsClosing => ValveState::Closing,
        }
    }
}

impl EntityUpdate for api::ValveStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn comps(&self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(2);
        comps.push(Component::Position(self.position as f64));
        comps.push(Component::ValveState(self.current_operation().as_igloo()));
        comps
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::ValveCommandRequest {
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

            ValveState(state) => {
                use igloo_interface::ValveState::*;
                match state {
                    Idle => req.stop = true,
                    Opening | Closing => {}
                }
            }

            comp => {
                println!(
                    "Valve got unexpected component '{comp:?}' during transaction. Skipping.."
                );
            }
        }
    }

    device
        .send_msg(MessageType::ValveCommandRequest, &req)
        .await
}
