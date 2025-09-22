// THIS IS GENERATED CODE - DO NOT MODIFY
// Generated from components.rs by build.rs

use crate::Entity;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Int(pub i32);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Float(pub f64);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bool(pub bool);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Text(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Object(pub HashMap<String, ComponentValue>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct List(pub Vec<ComponentValue>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Light;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Switch(pub bool);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dimmer(pub u8);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Unit {
    Seconds,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
#[repr(u16)]
pub enum ComponentType {
    Int,
    Float,
    Bool,
    Text,
    Object,
    List,
    Light,
    Switch,
    Dimmer,
    Color,
    Unit,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ComponentValue {
    Int(Int),
    Float(Float),
    Bool(Bool),
    Text(Text),
    Object(Object),
    List(List),
    Light(Light),
    Switch(Switch),
    Dimmer(Dimmer),
    Color(Color),
    Unit(Unit),
}

impl ComponentValue {
    pub fn get_type(&self) -> ComponentType {
        match self {
            ComponentValue::Int(_) => ComponentType::Int,
            ComponentValue::Float(_) => ComponentType::Float,
            ComponentValue::Bool(_) => ComponentType::Bool,
            ComponentValue::Text(_) => ComponentType::Text,
            ComponentValue::Object(_) => ComponentType::Object,
            ComponentValue::List(_) => ComponentType::List,
            ComponentValue::Light(_) => ComponentType::Light,
            ComponentValue::Switch(_) => ComponentType::Switch,
            ComponentValue::Dimmer(_) => ComponentType::Dimmer,
            ComponentValue::Color(_) => ComponentType::Color,
            ComponentValue::Unit(_) => ComponentType::Unit,
        }
    }
}

impl Entity {
    pub fn int(&self) -> Option<&Int> {
        match self.0.get(&ComponentType::Int) {
            Some(ComponentValue::Int(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn int_mut(&mut self) -> Option<&mut Int> {
        match self.0.get_mut(&ComponentType::Int) {
            Some(ComponentValue::Int(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn set_int(&mut self, val: Int) {
        self.0.insert(ComponentType::Int, ComponentValue::Int(val));
    }

    pub fn has_int(&self) -> bool {
        self.int().is_some()
    }

    pub fn float(&self) -> Option<&Float> {
        match self.0.get(&ComponentType::Float) {
            Some(ComponentValue::Float(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn float_mut(&mut self) -> Option<&mut Float> {
        match self.0.get_mut(&ComponentType::Float) {
            Some(ComponentValue::Float(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn set_float(&mut self, val: Float) {
        self.0
            .insert(ComponentType::Float, ComponentValue::Float(val));
    }

    pub fn has_float(&self) -> bool {
        self.float().is_some()
    }

    pub fn bool(&self) -> Option<&Bool> {
        match self.0.get(&ComponentType::Bool) {
            Some(ComponentValue::Bool(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn bool_mut(&mut self) -> Option<&mut Bool> {
        match self.0.get_mut(&ComponentType::Bool) {
            Some(ComponentValue::Bool(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn set_bool(&mut self, val: Bool) {
        self.0
            .insert(ComponentType::Bool, ComponentValue::Bool(val));
    }

    pub fn has_bool(&self) -> bool {
        self.bool().is_some()
    }

    pub fn text(&self) -> Option<&Text> {
        match self.0.get(&ComponentType::Text) {
            Some(ComponentValue::Text(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn text_mut(&mut self) -> Option<&mut Text> {
        match self.0.get_mut(&ComponentType::Text) {
            Some(ComponentValue::Text(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn set_text(&mut self, val: Text) {
        self.0
            .insert(ComponentType::Text, ComponentValue::Text(val));
    }

    pub fn has_text(&self) -> bool {
        self.text().is_some()
    }

    pub fn object(&self) -> Option<&Object> {
        match self.0.get(&ComponentType::Object) {
            Some(ComponentValue::Object(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn object_mut(&mut self) -> Option<&mut Object> {
        match self.0.get_mut(&ComponentType::Object) {
            Some(ComponentValue::Object(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn set_object(&mut self, val: Object) {
        self.0
            .insert(ComponentType::Object, ComponentValue::Object(val));
    }

    pub fn has_object(&self) -> bool {
        self.object().is_some()
    }

    pub fn list(&self) -> Option<&List> {
        match self.0.get(&ComponentType::List) {
            Some(ComponentValue::List(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn list_mut(&mut self) -> Option<&mut List> {
        match self.0.get_mut(&ComponentType::List) {
            Some(ComponentValue::List(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn set_list(&mut self, val: List) {
        self.0
            .insert(ComponentType::List, ComponentValue::List(val));
    }

    pub fn has_list(&self) -> bool {
        self.list().is_some()
    }

    pub fn light(&self) -> Option<&Light> {
        match self.0.get(&ComponentType::Light) {
            Some(ComponentValue::Light(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn light_mut(&mut self) -> Option<&mut Light> {
        match self.0.get_mut(&ComponentType::Light) {
            Some(ComponentValue::Light(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn set_light(&mut self, val: Light) {
        self.0
            .insert(ComponentType::Light, ComponentValue::Light(val));
    }

    pub fn has_light(&self) -> bool {
        self.light().is_some()
    }

    pub fn switch(&self) -> Option<&Switch> {
        match self.0.get(&ComponentType::Switch) {
            Some(ComponentValue::Switch(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn switch_mut(&mut self) -> Option<&mut Switch> {
        match self.0.get_mut(&ComponentType::Switch) {
            Some(ComponentValue::Switch(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn set_switch(&mut self, val: Switch) {
        self.0
            .insert(ComponentType::Switch, ComponentValue::Switch(val));
    }

    pub fn has_switch(&self) -> bool {
        self.switch().is_some()
    }

    pub fn dimmer(&self) -> Option<&Dimmer> {
        match self.0.get(&ComponentType::Dimmer) {
            Some(ComponentValue::Dimmer(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn dimmer_mut(&mut self) -> Option<&mut Dimmer> {
        match self.0.get_mut(&ComponentType::Dimmer) {
            Some(ComponentValue::Dimmer(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn set_dimmer(&mut self, val: Dimmer) {
        self.0
            .insert(ComponentType::Dimmer, ComponentValue::Dimmer(val));
    }

    pub fn has_dimmer(&self) -> bool {
        self.dimmer().is_some()
    }

    pub fn color(&self) -> Option<&Color> {
        match self.0.get(&ComponentType::Color) {
            Some(ComponentValue::Color(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn color_mut(&mut self) -> Option<&mut Color> {
        match self.0.get_mut(&ComponentType::Color) {
            Some(ComponentValue::Color(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn set_color(&mut self, val: Color) {
        self.0
            .insert(ComponentType::Color, ComponentValue::Color(val));
    }

    pub fn has_color(&self) -> bool {
        self.color().is_some()
    }

    pub fn unit(&self) -> Option<&Unit> {
        match self.0.get(&ComponentType::Unit) {
            Some(ComponentValue::Unit(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn unit_mut(&mut self) -> Option<&mut Unit> {
        match self.0.get_mut(&ComponentType::Unit) {
            Some(ComponentValue::Unit(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }
    }

    pub fn set_unit(&mut self, val: Unit) {
        self.0
            .insert(ComponentType::Unit, ComponentValue::Unit(val));
    }
    pub fn has_unit(&self) -> bool {
        self.unit().is_some()
    }
}
