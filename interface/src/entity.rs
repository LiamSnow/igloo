use crate::{Component, ComponentType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Entities(pub HashMap<String, Entity>);

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Entity(pub(crate) HashMap<ComponentType, Component>);

impl Entities {
    pub fn insert(&mut self, name: String, value: Entity) -> Option<Entity> {
        self.0.insert(name, value)
    }

    pub fn get(&self, name: &str) -> Option<&Entity> {
        self.0.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Entity> {
        self.0.get_mut(name)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Entity {
    pub fn from<const N: usize>(comps: [Component; N]) -> Self {
        let mut me = Self::default();
        for comp in comps {
            me.set(comp);
        }
        me
    }

    pub fn has(&self, typ: &ComponentType) -> bool {
        self.get(typ).is_some()
    }

    pub fn get(&self, typ: &ComponentType) -> Option<&Component> {
        self.0.get(typ)
    }

    pub fn get_mut(&mut self, typ: &ComponentType) -> Option<&mut Component> {
        self.0.get_mut(typ)
    }

    pub fn set(&mut self, val: Component) {
        let typ = val.get_type();
        self.0.insert(typ, val);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
