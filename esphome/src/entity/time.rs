use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use igloo_interface::{Component, types::IglooTime};

impl EntityRegister for api::ListEntitiesTimeResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(2);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        comps
    }
}

impl EntityUpdate for api::TimeStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    fn comps(&self) -> Vec<Component> {
        vec![Component::Time(IglooTime {
            hour: self.hour as u8,
            minute: self.minute as u8,
            second: self.second as u8,
        })]
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::TimeCommandRequest {
        key,
        hour: 0,
        minute: 0,
        second: 0,
    };

    for comp in comps {
        use Component::*;
        match comp {
            Time(time) => {
                req.hour = time.hour as u32;
                req.minute = time.minute as u32;
                req.second = time.second as u32;
            }

            comp => {
                println!("Time got unexpected component '{comp:?}' during transaction. Skipping..");
            }
        }
    }

    device.send_msg(MessageType::TimeCommandRequest, &req).await
}
