use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use igloo_interface::{Component, types::IglooDate};

impl EntityRegister for api::ListEntitiesDateResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(2);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        comps
    }
}

impl EntityUpdate for api::DateStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    fn comps(&self) -> Vec<Component> {
        vec![Component::Date(IglooDate {
            year: self.year as u16,
            month: self.month as u8,
            day: self.day as u8,
        })]
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::DateCommandRequest {
        key,
        year: 0,
        month: 0,
        day: 0,
    };

    for comp in comps {
        use Component::*;
        match comp {
            Date(date) => {
                req.year = date.year as u32;
                req.month = date.month as u32;
                req.day = date.day as u32;
            }

            comp => {
                println!("Date got unexpected component '{comp:?}' during transaction. Skipping..");
            }
        }
    }

    device.send_msg(MessageType::DateCommandRequest, &req).await
}
