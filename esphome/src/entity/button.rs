use super::{EntityRegister, add_device_class, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    model::MessageType,
};
use igloo_interface::Component;

impl EntityRegister for api::ListEntitiesButtonResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(3);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        add_device_class(&mut comps, self.device_class);
        comps
    }
}

#[inline]
pub async fn process(
    device: &mut Device,
    key: u32,
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let req = api::ButtonCommandRequest { key };

    for comp in comps {
        println!("Button got unexpected component '{comp:?}' during transaction. Skipping..");
    }

    device
        .send_msg(MessageType::ButtonCommandRequest, &req)
        .await
}
