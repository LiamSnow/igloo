use crate::{ComponentType, ComponentValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub entities: Entities,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Entities(pub HashMap<String, Entity>);

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Entity(pub(crate) HashMap<ComponentType, ComponentValue>);

impl Entity {
    pub fn has(&self, typ: &ComponentType) -> bool {
        self.get(typ).is_some()
    }

    pub fn get(&self, typ: &ComponentType) -> Option<&ComponentValue> {
        self.0.get(typ)
    }

    pub fn get_mut(&mut self, typ: &ComponentType) -> Option<&mut ComponentValue> {
        self.0.get_mut(typ)
    }

    pub fn set(&mut self, val: ComponentValue) {
        let typ = val.get_type();
        self.0.insert(typ, val);
    }
}
