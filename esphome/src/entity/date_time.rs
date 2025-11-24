use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use igloo_interface::Component;

impl EntityRegister for api::ListEntitiesDateTimeResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(2);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        comps
    }
}

impl EntityUpdate for api::DateTimeStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    fn comps(&self) -> Vec<Component> {
        vec![Component::Timestamp(self.epoch_seconds as i64)]
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::DateTimeCommandRequest {
        key,
        epoch_seconds: 0,
    };

    for comp in comps {
        use Component::*;
        match comp {
            Timestamp(epoch_seconds) => {
                req.epoch_seconds = epoch_seconds as u32;
            }

            comp => {
                println!("DateTime got unexpected component '{comp:?}' during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::DateTimeCommandRequest, &req)
        .await
}
