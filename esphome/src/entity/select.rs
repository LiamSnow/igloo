use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use igloo_interface::Component;

impl EntityRegister for api::ListEntitiesSelectResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(4);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        comps.push(Component::TextSelect);
        comps.push(Component::TextList(self.options));
        comps
    }
}

impl EntityUpdate for api::SelectStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    fn comps(&self) -> Vec<Component> {
        vec![Component::Text(self.state.clone())]
    }
}

#[inline]
pub async fn process(
    device: &mut Device,
    key: u32,
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::SelectCommandRequest {
        key,
        state: String::new(),
    };

    for comp in comps {
        use Component::*;
        match comp {
            Text(state) => {
                req.state = state;
            }

            comp => {
                println!("Select got unexpected component '{comp:?}' during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::SelectCommandRequest, &req)
        .await
}
