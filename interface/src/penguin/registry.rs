use super::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PenguinRegistry {
    pub libraries: HashMap<String, PenguinLibrary>,
}

#[derive(Debug, Clone, Default)]
pub struct PenguinLibrary {
    pub nodes: HashMap<String, PenguinNodeDefn>,
}

impl PenguinRegistry {
    pub fn new() -> Self {
        let mut libraries = HashMap::new();
        libraries.insert("Standard Library".to_string(), std_library());

        Self { libraries }
    }

    pub fn get_defn(&self, dref: &PenguinNodeDefnRef) -> Option<&PenguinNodeDefn> {
        self.libraries
            .get(&dref.lib_path)?
            .nodes
            .get(&dref.node_path)
    }
}

impl Default for PenguinRegistry {
    fn default() -> Self {
        Self::new()
    }
}
