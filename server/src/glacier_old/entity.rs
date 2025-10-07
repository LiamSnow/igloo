use igloo_interface::{Component, ComponentType, MAX_SUPPORTED_COMPONENT};
use smallvec::SmallVec;

use crate::glacier::query::QueryFilter;

#[derive(Debug)]
pub struct Entity {
    pub name: String,

    /// stores the actual components
    components: SmallVec<[Component; 8]>,

    /// maps Component ID -> index in `.components`
    /// 0xFF = not present
    indices: [u8; MAX_SUPPORTED_COMPONENT as usize],
}

impl Entity {
    pub fn new(name: String) -> Self {
        Self {
            name,
            components: SmallVec::new(),
            indices: [0xFF; MAX_SUPPORTED_COMPONENT as usize],
        }
    }

    #[inline(always)]
    pub fn get(&self, typ: ComponentType) -> Option<&Component> {
        let idx = self.indices[typ as usize];
        if idx != 0xFF {
            Some(&self.components[idx as usize])
        } else {
            None
        }
    }

    /// returns the [ComponentType] if it inserted
    pub fn set(&mut self, val: Component) -> Option<ComponentType> {
        let typ = val.get_type();
        let type_id = typ as usize;
        let idx = self.indices[type_id];
        if idx != 0xFF {
            // update existing
            self.components[idx as usize] = val;
            None
        } else {
            // add new
            let new_idx = self.components.len() as u8;
            self.components.push(val);
            self.indices[type_id] = new_idx;
            Some(typ)
        }
    }
}

impl HasComponent for Entity {
    #[inline(always)]
    fn has(&self, typ: ComponentType) -> bool {
        self.indices[typ as usize] != 0xFF
    }
}

pub trait HasComponent {
    fn has(&self, typ: ComponentType) -> bool;

    fn matches_filter(&self, filter: &QueryFilter) -> bool {
        match filter {
            QueryFilter::With(typ) => self.has(*typ),
            QueryFilter::Without(typ) => !self.has(*typ),
            QueryFilter::And(parts) => {
                let (lhs, rhs) = parts.as_ref();
                self.matches_filter(lhs) && self.matches_filter(rhs)
            }
            QueryFilter::Or(parts) => {
                let (lhs, rhs) = parts.as_ref();
                self.matches_filter(lhs) || self.matches_filter(rhs)
            }
        }
    }
}
