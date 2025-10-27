use std::{collections::HashMap, sync::Arc};

use derive_more::Display;

use crate::Color;

pub mod core;

#[derive(Clone)]
pub struct PenguinRegistry {
    pub libraries: Arc<HashMap<String, PenguinLibrary>>,
}

#[derive(Debug, Clone, Default, PartialEq, Display)]
#[display("{library}.{name}")]
pub struct NodeDefnRef {
    pub library: String,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct PenguinLibrary {
    pub name: String,
    pub nodes: HashMap<String, NodeDefn>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeDefn {
    pub title: String,
    pub desc: String,
    pub style: NodeStyle,
    pub inputs: Vec<PinDefn>,
    pub outputs: Vec<PinDefn>,
    pub cfg: Vec<NodeConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeStyle {
    /// icon
    Normal(String),
    /// background
    Background(String),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeConfig {
    /// add button and remove button
    AddRemovePin(AddRemovePinConfig),
    /// adds a query input
    Query(QueryConfig),
    Input(InputConfig),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputConfig {
    pub r#type: ValueType,
    /// value is saved under this ID
    pub id: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddRemovePinConfig {
    pub r#type: PinType,
    /// must match to a PinType::Phantom(id)
    pub phantom_id: u8,
    pub min: u8,
    pub max: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryConfig {
    /// must match to a PinType::DynValue(id)
    pub dyn_value_id: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinDefn {
    pub name: String,
    pub r#type: PinDefnType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinDefnType {
    Flow,
    Value(ValueType),
    /// allows more pins to be attached variably
    /// value is ID referenced by `cfg`
    Phantom(u8),
    /// Value pin with variable ValueType
    /// value is ID referenced by `cfg`
    DynValue(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum PinType {
    Flow,
    Value(ValueType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum ValueType {
    #[display("Integer")]
    Int,
    #[display("Real")]
    Real,
    #[display("Text")]
    Text,
    #[display("Boolean")]
    Bool,
    #[display("Color")]
    Color,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueData {
    Int(i64),
    Real(f64),
    Text(String),
    Bool(bool),
    Color(Color),
}

impl ValueType {
    pub fn color(&self) -> &'static str {
        match self {
            ValueType::Text => "#9b59b6",
            ValueType::Bool => "#e74c3c",
            ValueType::Int => "#3498db",
            ValueType::Real => "#27ae60",
            ValueType::Color => "#f39c12",
        }
    }

    pub fn can_cast(self, to: Self) -> bool {
        match to {
            ValueType::Int => self != ValueType::Color,
            ValueType::Real => self != ValueType::Color,
            ValueType::Text => true,
            ValueType::Bool => self != ValueType::Color,
            ValueType::Color => false,
        }
    }

    pub fn cast_name(self, to: Self) -> Option<String> {
        if !self.can_cast(to) {
            return None;
        }
        Some(format!(
            "cast_{}_to_{}",
            self.to_string().to_lowercase(),
            to.to_string().to_lowercase()
        ))
    }
}

impl PinType {
    pub fn can_cast(self, to: Self) -> bool {
        match (self, to) {
            (PinType::Value(from), PinType::Value(to)) => from.can_cast(to),
            _ => false,
        }
    }

    pub fn cast_name(self, to: Self) -> Option<String> {
        match (self, to) {
            (PinType::Value(from), PinType::Value(to)) => from.cast_name(to),
            _ => None,
        }
    }

    pub fn can_connect_to(&self, target: PinType) -> bool {
        *self == target || self.can_cast(target)
    }
}

impl PenguinRegistry {
    pub fn new() -> Self {
        let mut libraries = HashMap::new();
        libraries.insert("std".to_string(), core::std_library());

        Self {
            libraries: Arc::new(libraries),
        }
    }

    pub fn get_defn(&self, library: &str, name: &str) -> Option<&NodeDefn> {
        self.libraries.get(library)?.nodes.get(name)
    }

    pub fn filter_nodes(
        &self,
        search_query: &str,
        wire_filter: Option<PinType>,
    ) -> Vec<(String, String, NodeDefn)> {
        let query = search_query.to_lowercase();

        self.libraries
            .values()
            .flat_map(|lib| {
                lib.nodes
                    .iter()
                    .map(move |(name, defn)| (lib.name.clone(), name.clone(), defn.clone()))
            })
            .filter(|(lib_name, node_name, defn)| {
                let matches_search = query.is_empty()
                    || defn.title.to_lowercase().contains(&query)
                    || node_name.to_lowercase().contains(&query)
                    || lib_name.to_lowercase().contains(&query);

                if !matches_search {
                    return false;
                }

                if let Some(wire_type) = wire_filter {
                    defn.has_compatible_input(wire_type)
                } else {
                    true
                }
            })
            .collect()
    }
}

impl Default for PenguinRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PinDefn {
    pub fn new(name: &str, r#type: PinDefnType) -> Self {
        Self {
            name: name.to_string(),
            r#type,
        }
    }
}

impl NodeStyle {
    pub fn normal(icon: &str) -> Self {
        Self::Normal(icon.to_string())
    }

    pub fn background(background: &str) -> Self {
        Self::Background(background.to_string())
    }

    pub fn none() -> Self {
        Self::None
    }
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self::none()
    }
}

impl NodeDefnRef {
    pub fn new(library: &str, name: &str) -> Self {
        Self {
            library: library.to_string(),
            name: name.to_string(),
        }
    }
}

impl PinType {
    pub fn stroke(&self) -> &str {
        match self {
            PinType::Flow => "white",
            PinType::Value(vt) => vt.color(),
        }
    }

    pub fn stroke_width(&self) -> u8 {
        match self {
            PinType::Flow => 4,
            PinType::Value(_) => 2,
        }
    }
}

impl Color {
    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.trim_start_matches('#');
        if s.len() != 6 {
            return None;
        }
        Some(Color {
            r: u8::from_str_radix(&s[0..2], 16).ok()?,
            g: u8::from_str_radix(&s[2..4], 16).ok()?,
            b: u8::from_str_radix(&s[4..6], 16).ok()?,
        })
    }
}

impl NodeDefn {
    pub fn get_phantom_config(&self, phantom_id: u8) -> Option<&AddRemovePinConfig> {
        self.cfg.iter().find_map(|cfg| {
            if let NodeConfig::AddRemovePin(config) = cfg
                && config.phantom_id == phantom_id
            {
                return Some(config);
            }
            None
        })
    }

    pub fn resolve_pin_type(&self, pin_defn_type: &PinDefnType) -> Option<PinType> {
        match pin_defn_type {
            PinDefnType::Flow => Some(PinType::Flow),
            PinDefnType::Value(vt) => Some(PinType::Value(*vt)),
            PinDefnType::Phantom(phantom_id) => {
                self.get_phantom_config(*phantom_id).map(|cfg| cfg.r#type)
            }
            PinDefnType::DynValue(_) => None, // TODO FIXME implement
        }
    }

    pub fn find_compatible_inputs(&self, wire_type: PinType) -> Vec<(usize, PinType)> {
        self.inputs
            .iter()
            .enumerate()
            .filter_map(|(idx, pin)| {
                let pin_type = self.resolve_pin_type(&pin.r#type)?;
                if pin_type == wire_type || wire_type.can_cast(pin_type) {
                    Some((idx, pin_type))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn has_compatible_input(&self, wire_type: PinType) -> bool {
        self.inputs.iter().any(|pin| {
            if let Some(pin_type) = self.resolve_pin_type(&pin.r#type) {
                pin_type == wire_type || wire_type.can_cast(pin_type)
            } else {
                false
            }
        })
    }
}
