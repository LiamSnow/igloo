use super::{EntityRegister, add_device_class, add_entity_category, add_icon, add_unit};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use igloo_interface::Component;

impl EntityRegister for api::ListEntitiesNumberResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(4);
        add_entity_category(&mut comps, self.entity_category());
        // add_number_mode(&mut comps, self.mode());
        add_icon(&mut comps, &self.icon);
        add_device_class(&mut comps, self.device_class);
        // add_f32_bounds(&mut comps, self.min_value, self.max_value, Some(self.step));
        add_unit(&mut comps, self.unit_of_measurement);
        comps
    }
}

impl EntityUpdate for api::NumberStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    fn comps(&self) -> Vec<Component> {
        vec![Component::Real(self.state as f64)]
    }
}

// impl api::NumberMode {
//     pub fn as_igloo(&self) -> NumberMode {
//         match self {
//             api::NumberMode::Auto => NumberMode::Auto,
//             api::NumberMode::Box => NumberMode::Box,
//             api::NumberMode::Slider => NumberMode::Slider,
//         }
//     }
// }

#[inline]
pub async fn process(
    device: &mut Device,
    key: u32,
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::NumberCommandRequest { key, state: 0.0 };

    for comp in comps {
        use Component::*;
        match comp {
            Real(state) => {
                req.state = state as f32;
            }

            comp => {
                println!("Number got unexpected component '{comp:?}' during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::NumberCommandRequest, &req)
        .await
}
