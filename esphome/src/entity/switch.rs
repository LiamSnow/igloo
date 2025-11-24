use super::{EntityRegister, add_device_class, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use igloo_interface::Component;

impl EntityRegister for api::ListEntitiesSwitchResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(3);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        add_device_class(&mut comps, self.device_class);
        comps
    }
}

impl EntityUpdate for api::SwitchStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn comps(&self) -> Vec<Component> {
        vec![Component::Switch(self.state)]
    }
}

#[inline]
pub async fn process(
    device: &mut Device,
    key: u32,
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::SwitchCommandRequest { key, state: false };

    for comp in comps {
        use Component::*;
        match comp {
            Switch(state) => {
                req.state = state;
            }

            comp => {
                println!(
                    "Switch got unexpected component '{comp:?}' during transaction. Skipping.."
                );
            }
        }
    }

    device
        .send_msg(MessageType::SwitchCommandRequest, &req)
        .await
}
