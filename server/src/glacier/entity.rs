use igloo_interface::{Component, ComponentType, MAX_SUPPORTED_COMPONENT};
use smallvec::SmallVec;

#[derive(Debug)]
pub struct Entity {
    /// stores the actual components
    components: SmallVec<[Component; 8]>,

    /// maps Component ID -> index in `.components`
    /// 0xFF = not present
    indices: [u8; MAX_SUPPORTED_COMPONENT as usize],
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            components: SmallVec::new(),
            indices: [0xFF; MAX_SUPPORTED_COMPONENT as usize],
        }
    }
}

impl Entity {
    #[inline(always)]
    pub fn get(&self, typ: ComponentType) -> Option<&Component> {
        let idx = self.indices[typ as usize];
        if idx != 0xFF {
            Some(&self.components[idx as usize])
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn has(&self, typ: ComponentType) -> bool {
        self.indices[typ as usize] != 0xFF
    }

    /// returns true if it made a new component
    pub fn set(&mut self, val: Component) -> bool {
        let type_id = val.get_type_id();
        let idx = self.indices[type_id as usize];
        if idx != 0xFF {
            // update existing
            self.components[idx as usize] = val;
            false
        } else {
            // add new
            let new_idx = self.components.len() as u8;
            self.components.push(val);
            self.indices[type_id as usize] = new_idx;
            true
        }
    }
}
