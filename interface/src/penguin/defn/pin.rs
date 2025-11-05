use serde::{Deserialize, Serialize};

use crate::{
    penguin::{graph::PenguinNodeID, *},
    types::IglooType,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PenguinPinID(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct PenguinPinRef {
    pub node_id: PenguinNodeID,
    pub id: PenguinPinID,
    pub is_output: bool,
    pub r#type: PenguinPinType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PenguinPinDefn {
    pub r#type: PenguinPinType,
    pub hide_name: bool,
}

impl PenguinPinDefn {
    pub fn named(r#type: PenguinPinType) -> Self {
        Self {
            r#type,
            hide_name: false,
        }
    }

    pub fn unnamed(r#type: PenguinPinType) -> Self {
        Self {
            r#type,
            hide_name: true,
        }
    }

    pub fn named_flow() -> Self {
        Self::named(PenguinPinType::Flow)
    }

    pub fn unnamed_flow() -> Self {
        Self::unnamed(PenguinPinType::Flow)
    }

    pub fn named_val(t: IglooType) -> Self {
        Self::named(PenguinPinType::Value(t))
    }

    pub fn unnamed_val(t: IglooType) -> Self {
        Self::unnamed(PenguinPinType::Value(t))
    }
}

impl PenguinPinID {
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl PenguinPinRef {
    pub fn can_connect_to(&self, other: &Self) -> bool {
        let compatible = if self.is_output {
            self.r#type.can_connect_to(other.r#type)
        } else {
            other.r#type.can_connect_to(self.r#type)
        };

        self.is_output != other.is_output && compatible && self.node_id != other.node_id
    }

    pub fn cast_name(&self, end_type: PenguinPinType) -> Option<String> {
        if self.is_output {
            self.r#type.cast_name(end_type)
        } else {
            end_type.cast_name(self.r#type)
        }
    }

    pub fn find_compatible<'a>(
        &self,
        defn: &'a PenguinNodeDefn,
    ) -> Option<(&'a PenguinPinID, &'a PenguinPinDefn)> {
        let t = if self.is_output {
            &defn.inputs
        } else {
            &defn.outputs
        };

        for (id, defn) in t {
            if defn.r#type == self.r#type {
                return Some((id, defn));
            }
        }

        None
    }
}
