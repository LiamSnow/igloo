use crate::penguin::*;
use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;
use indexmap::IndexMap;

#[derive(Debug, Clone, Default, PartialEq, Display, BorshSerialize, BorshDeserialize)]
#[display("{library}.{name}")]
pub struct NodeDefnRef {
    pub library: String,
    pub name: String,
    pub version: u8,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeDefn {
    pub title: String,
    pub desc: String,
    pub style: NodeStyle,
    pub inputs: IndexMap<PinID, PinDefn>,
    pub outputs: IndexMap<PinID, PinDefn>,
    pub cfg: Vec<NodeConfig>,
    pub version: u8,
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
    /// Expandable I/O with +/- buttons
    Variadic(VariadicConfig),
    /// Adds a Query Input
    Query(QueryConfig),
    /// Arbitrary Input
    Input(InputConfig),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariadicConfig {
    pub prev: Option<String>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, BorshSerialize, BorshDeserialize, Hash)]
pub struct InputID(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputConfig {
    pub r#type: PenguinType,
    /// Value is saved under this ID
    pub id: InputID,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryConfig {
    /// Pin ID of a PinDefnType::DynValue
    pub dyn_pin: PinID,
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

impl NodeDefn {
    pub fn get_variadic_config(&self) -> Option<&VariadicConfig> {
        self.cfg.iter().find_map(|cfg| {
            if let NodeConfig::Variadic(config) = cfg {
                Some(config)
            } else {
                None
            }
        })
    }
}

impl InputID {
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }
}
