use super::{EntityRegister, add_entity_category, add_icon};
use crate::api;
use igloo_interface::Component;

impl EntityRegister for api::ListEntitiesCameraResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(2);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        comps
    }
}
