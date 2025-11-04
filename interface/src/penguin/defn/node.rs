use crate::penguin::*;
use derive_more::Display;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Display, Serialize, Deserialize)]
#[display("{lib_path}/{node_path}")]
pub struct PenguinNodeDefnRef {
    pub lib_path: String,
    pub node_path: String,
    pub version: u8,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PenguinNodeDefn {
    pub icon: String,
    pub desc: String,
    pub title_bar: Option<String>,
    pub icon_bg: bool,
    pub is_reroute: bool,
    pub inputs: IndexMap<PenguinPinID, PenguinPinDefn>,
    pub outputs: IndexMap<PenguinPinID, PenguinPinDefn>,
    pub version: u8,
    pub hide_search: bool,
    pub variadic_feature: Option<NodeVariadicFeature>,
    pub input_features: Vec<NodeInputFeature>,
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

impl PenguinNodeDefnRef {
    pub fn new(lib_path: &str, node_path: &str, version: u8) -> Self {
        Self {
            lib_path: lib_path.to_string(),
            node_path: node_path.to_string(),
            version,
        }
    }
}

impl NodeInputFeatureID {
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }
}
