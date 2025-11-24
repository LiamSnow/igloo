use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use igloo_interface::{Component, FanDirection, FanOscillation, FanSpeed};

impl EntityRegister for api::ListEntitiesFanResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(2);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        comps
    }
}

impl api::FanDirection {
    pub fn as_igloo(&self) -> FanDirection {
        match self {
            api::FanDirection::Forward => FanDirection::Forward,
            api::FanDirection::Reverse => FanDirection::Reverse,
        }
    }
}

impl api::FanSpeed {
    pub fn as_igloo(&self) -> FanSpeed {
        match self {
            api::FanSpeed::Low => FanSpeed::Low,
            api::FanSpeed::Medium => FanSpeed::Medium,
            api::FanSpeed::High => FanSpeed::High,
        }
    }
}

impl EntityUpdate for api::FanStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn comps(&self) -> Vec<Component> {
        vec![
            Component::FanSpeed(self.speed().as_igloo()),
            Component::Integer(self.speed_level as i64),
            Component::FanDirection(self.direction().as_igloo()),
            Component::Text(self.preset_mode.clone()),
            Component::FanOscillation(match self.oscillating {
                true => FanOscillation::On,
                false => FanOscillation::Off,
            }),
        ]
    }
}

fn fan_direction_to_api(direction: &FanDirection) -> api::FanDirection {
    match direction {
        FanDirection::Forward => api::FanDirection::Forward,
        FanDirection::Reverse => api::FanDirection::Reverse,
    }
}

#[inline]
pub async fn process(
    device: &mut Device,
    key: u32,
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::FanCommandRequest {
        key,
        ..Default::default()
    };

    for comp in comps {
        use Component::*;
        match comp {
            Switch(state) => {
                req.has_state = true;
                req.state = state;
            }

            Integer(speed_level) => {
                req.has_speed_level = true;
                req.speed_level = speed_level as i32;
            }

            FanOscillation(oscillation) => {
                req.has_oscillating = true;
                req.oscillating = !matches!(oscillation, igloo_interface::FanOscillation::Off);
            }

            FanDirection(direction) => {
                req.has_direction = true;
                req.direction = fan_direction_to_api(&direction).into();
            }

            Text(preset) => {
                req.has_preset_mode = true;
                req.preset_mode = preset;
            }

            comp => {
                println!("Fan got unexpected component '{comp:?}' during transaction. Skipping..");
            }
        }
    }

    device.send_msg(MessageType::FanCommandRequest, &req).await
}
