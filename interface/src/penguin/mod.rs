use std::collections::HashMap;

pub mod core;

#[derive(Debug, Clone, Default)]
pub struct NodeDefnRef {
    pub library: String,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct PenguinLibrary {
    pub name: String,
    pub nodes: HashMap<String, NodeDefn>,
}

#[derive(Debug, Clone, Default)]
pub struct NodeDefn {
    pub style: NodeStyle,
    pub desc: String,
    pub inputs: Vec<PinDefn>,
    pub outputs: Vec<PinDefn>,
    pub cfg: Vec<NodeConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeStyle {
    /// icon, title
    Normal(String, String),
    /// background
    Compact(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeConfig {
    /// adds a button to add pins of PinType
    AddPin(AddPinConfig),
    /// adds a button to remove pins of PinType
    RemovePin(RemovePinConfig),
    /// adds a query input
    Query(QueryConfig),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddPinConfig {
    pub r#type: PinType,
    /// must match to a PinType::Phantom(id)
    pub phantom_id: u8,
    pub max: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovePinConfig {
    pub r#type: PinType,
    /// must match to a PinType::Phantom(id)
    pub phantom_id: u8,
    pub min: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryConfig {
    /// must match to a PinType::DynValue(id)
    pub dyn_value_id: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinDefn {
    pub name: String,
    pub r#type: PinType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinType {
    Flow,
    Value(ValueType),
    /// allows more pins to be attached variably
    /// value is ID referenced by `cfg`
    Phantom(u8),
    /// Value pin with variable ValueType
    /// value is ID referenced by `cfg`
    DynValue(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    Integer,
    Real,
    Text,
    Boolean,
    Color,
}

impl ValueType {
    pub fn color(&self) -> &'static str {
        match self {
            ValueType::Text => "#9b59b6",
            ValueType::Boolean => "#e74c3c",
            ValueType::Integer => "#3498db",
            ValueType::Real => "#27ae60",
            ValueType::Color => "#f39c12",
        }
    }
}

impl PinDefn {
    pub fn new(name: &str, r#type: PinType) -> Self {
        Self {
            name: name.to_string(),
            r#type,
        }
    }
}

impl NodeStyle {
    pub fn normal(icon: &str, title: &str) -> Self {
        Self::Normal(icon.to_string(), title.to_string())
    }

    pub fn compact(background: &str) -> Self {
        Self::Compact(background.to_string())
    }
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self::normal("", "")
    }
}
