use super::*;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct PenguinRegistry {
    pub libraries: Arc<HashMap<String, PenguinLibrary>>,
}

#[derive(Debug, Clone, Default)]
pub struct PenguinLibrary {
    pub name: String,
    pub nodes: HashMap<String, NodeDefn>,
}

impl PenguinRegistry {
    pub fn new() -> Self {
        let mut libraries = HashMap::new();
        libraries.insert("std".to_string(), std_library());

        Self {
            libraries: Arc::new(libraries),
        }
    }

    pub fn get_defn(&self, dref: &NodeDefnRef) -> Option<&NodeDefn> {
        self.libraries.get(&dref.library)?.nodes.get(&dref.name)
    }
}

impl Default for PenguinRegistry {
    fn default() -> Self {
        Self::new()
    }
}
