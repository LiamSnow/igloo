use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use igloo_interface::Component;

impl EntityRegister for api::ListEntitiesTextResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(2);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        // add_text_mode(&mut comps, self.mode().as_igloo());
        // add_text_min_length(&mut comps, self.min_length);
        // add_text_max_length(&mut comps, self.max_length);
        // add_text_pattern(&mut comps, self.pattern);
        comps
    }
}

// impl api::TextMode {
//     pub fn as_igloo(&self) -> TextMode {
//         match self {
//             api::TextMode::Text => TextMode::Text,
//             api::TextMode::Password => TextMode::Password,
//         }
//     }
// }

impl EntityUpdate for api::TextStateResponse {
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
    let mut req = api::TextCommandRequest {
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
                println!("Text got unexpected component '{comp:?}' during transaction. Skipping..");
            }
        }
    }

    device.send_msg(MessageType::TextCommandRequest, &req).await
}
