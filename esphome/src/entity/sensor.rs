use super::{
    EntityRegister, add_device_class, add_entity_category, add_icon, add_sensor_state_class,
    add_unit,
};
use crate::{api, entity::EntityUpdate};
use igloo_interface::Component;

impl EntityRegister for api::ListEntitiesSensorResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(7);
        comps.push(Component::Sensor);
        add_entity_category(&mut comps, self.entity_category());
        add_sensor_state_class(&mut comps, self.state_class());
        add_icon(&mut comps, &self.icon);
        add_device_class(&mut comps, self.device_class);
        add_unit(&mut comps, self.unit_of_measurement);
        comps.push(Component::AccuracyDecimals(self.accuracy_decimals as i64));
        comps
    }
}

impl EntityUpdate for api::SensorStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    fn comps(&self) -> Vec<Component> {
        vec![Component::Real(self.state as f64)]
    }
}
