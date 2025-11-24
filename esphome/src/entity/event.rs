use super::{EntityRegister, add_device_class, add_entity_category, add_icon};
use crate::api;
use igloo_interface::Component;

impl EntityRegister for api::ListEntitiesEventResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(4);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        add_device_class(&mut comps, self.device_class);
        comps.push(Component::TextList(self.event_types));
        comps
    }
}
