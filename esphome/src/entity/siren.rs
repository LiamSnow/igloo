use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use igloo_interface::Component;

impl EntityRegister for api::ListEntitiesSirenResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(5);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        comps.push(Component::TextSelect);
        comps.push(Component::TextList(self.tones));
        comps.push(Component::Siren);
        comps
    }
}

impl EntityUpdate for api::SirenStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn comps(&self) -> Vec<Component> {
        vec![Component::Boolean(self.state)]
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::SirenCommandRequest {
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

            Text(tone) => {
                req.has_tone = true;
                req.tone = tone;
            }

            Volume(volume) => {
                req.has_volume = true;
                req.volume = volume as f32;
            }

            Integer(duration) => {
                req.has_duration = true;
                req.duration = duration as u32;
            }

            comp => {
                println!("Siren got unexpected component '{comp:?}' during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::SirenCommandRequest, &req)
        .await
}
