use crate::api;
use igloo_interface::{Component, SensorStateClass, Unit};

pub mod alarm_control_panel;
pub mod binary_sensor;
pub mod button;
pub mod camera;
pub mod climate;
pub mod cover;
pub mod date;
pub mod date_time;
pub mod event;
pub mod fan;
pub mod light;
pub mod lock;
pub mod media_player;
pub mod number;
pub mod select;
pub mod sensor;
pub mod siren;
pub mod switch;
pub mod text;
pub mod text_sensor;
pub mod time;
pub mod update;
pub mod valve;

pub trait EntityUpdate {
    fn key(&self) -> u32;
    fn should_skip(&self) -> bool {
        false
    }
    fn comps(&self) -> Vec<Component>;
}

pub trait EntityRegister {
    fn comps(self) -> Vec<Component>;
}

pub fn add_entity_category(comps: &mut Vec<Component>, category: api::EntityCategory) {
    match category {
        api::EntityCategory::None => {}
        api::EntityCategory::Config => {
            comps.push(Component::Config);
        }
        api::EntityCategory::Diagnostic => {
            comps.push(Component::Diagnostic);
        }
    }
}

pub fn add_icon(comps: &mut Vec<Component>, icon: &String) {
    if !icon.is_empty() {
        comps.push(Component::Icon(icon.to_string()))
    }
}

pub fn add_unit(comps: &mut Vec<Component>, unit_str: String) {
    // TODO log error, something else? if parsing failed..?
    if !unit_str.is_empty()
        && let Ok(unit) = Unit::try_from(unit_str)
    {
        comps.push(Component::Unit(unit));
    }
}

pub fn add_device_class(comps: &mut Vec<Component>, device_class: String) {
    if !device_class.is_empty() {
        comps.push(Component::DeviceClass(device_class));
    }
}

pub fn add_sensor_state_class(comps: &mut Vec<Component>, state_class: api::SensorStateClass) {
    match state_class {
        api::SensorStateClass::StateClassNone => {}
        api::SensorStateClass::StateClassMeasurement => {
            comps.push(Component::SensorStateClass(SensorStateClass::Measurement))
        }
        api::SensorStateClass::StateClassTotalIncreasing => comps.push(
            Component::SensorStateClass(SensorStateClass::TotalIncreasing),
        ),
        api::SensorStateClass::StateClassTotal => {
            comps.push(Component::SensorStateClass(SensorStateClass::Total))
        }
    }
}
