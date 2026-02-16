use crate::{penguin::*, types::IglooType};
use derive_more::Display;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Display, Serialize, Deserialize)]
#[display("{lib_name}/{node_name}")]
pub struct PenguinNodeDefnRef {
    pub lib_name: String,
    pub node_name: String,
    pub version: u8,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PenguinNodeDefn {
    pub icon: String,
    pub desc: String,
    pub title_bar: Option<String>,
    pub icon_bg: bool,
    pub inputs: IndexMap<PenguinPinID, PenguinPinDefn>,
    pub outputs: IndexMap<PenguinPinID, PenguinPinDefn>,
    pub version: u8,
    pub hide_search: bool,
    pub variadic_feature: Option<NodeVariadicFeature>,
    pub input_features: Vec<NodeInputFeature>,
    pub query_feature: Option<NodeQueryFeature>,
    pub is_reroute: bool,
    pub is_section: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeVariadicFeature {
    pub prev: Option<String>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeQueryFeature {
    pub base: String,
    pub is_aggregate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeInputFeatureID(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInputFeature {
    pub value_type: IglooType,
    pub input_type: NodeInputType,
    /// Value is saved under this ID
    pub id: NodeInputFeatureID,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeInputType {
    Input,
    Select(Vec<String>),
    TextArea,
}

impl PenguinNodeDefnRef {
    pub fn new(lib_name: &str, node_name: &str, version: u8) -> Self {
        Self {
            lib_name: lib_name.to_string(),
            node_name: node_name.to_string(),
            version,
        }
    }
}

impl NodeInputFeatureID {
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }
}
