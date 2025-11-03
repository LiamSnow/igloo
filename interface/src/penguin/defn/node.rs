use crate::penguin::*;
use derive_more::Display;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Display, Serialize, Deserialize)]
#[display("{lib_path}.{node_path}")]
pub struct PenguinNodeDefnRef {
    pub lib_path: String,
    pub node_path: String,
    pub version: u8,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PenguinNodeDefn {
    pub title: String,
    pub desc: String,
    pub style: NodeStyle,
    pub inputs: IndexMap<PenguinPinID, PenguinPinDefn>,
    pub outputs: IndexMap<PenguinPinID, PenguinPinDefn>,
    pub features: Vec<NodeFeature>,
    pub version: u8,
    pub hide_search: bool,
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
pub enum NodeFeature {
    /// Expandable I/O with +/- buttons
    Variadic(NodeVariadicFeature),
    /// Adds a Query Input
    Query(NodeQueryFeature),
    /// Arbitrary Input
    Input(NodeInputFeature),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeVariadicFeature {
    pub prev: Option<String>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct NodeInputFeatureID(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInputFeature {
    pub r#type: PenguinType,
    /// Value is saved under this ID
    pub id: NodeInputFeatureID,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeQueryFeature {
    /// Base Name of NodeDefn
    /// For query config you need "base_int", "base_text", ..
    /// Then this will automatically switch between them
    pub base_name: String,
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

impl PenguinNodeDefnRef {
    pub fn new(lib_path: &str, node_path: &str, version: u8) -> Self {
        Self {
            lib_path: lib_path.to_string(),
            node_path: node_path.to_string(),
            version,
        }
    }
}

impl PenguinNodeDefn {
    pub fn variadic_feature(&self) -> Option<&NodeVariadicFeature> {
        self.features.iter().find_map(|cfg| {
            if let NodeFeature::Variadic(config) = cfg {
                Some(config)
            } else {
                None
            }
        })
    }

    pub fn num_input_features(&self) -> usize {
        let mut count = 0;
        for cfg in &self.features {
            if matches!(cfg, NodeFeature::Input(_)) {
                count += 1;
            }
        }
        count
    }

    pub fn input_features(&self) -> Vec<&NodeInputFeature> {
        self.features
            .iter()
            .filter_map(|cfg| {
                if let NodeFeature::Input(config) = cfg {
                    Some(config)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl NodeInputFeatureID {
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }
}
