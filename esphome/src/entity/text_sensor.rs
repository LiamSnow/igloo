use super::{EntityRegister, add_device_class, add_entity_category, add_icon};
use crate::{api, entity::EntityUpdate};
use igloo_interface::Component;

impl EntityRegister for api::ListEntitiesTextSensorResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(4);
        comps.push(Component::Sensor);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        add_device_class(&mut comps, self.device_class);
        comps
    }
}

impl EntityUpdate for api::TextSensorStateResponse {
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
