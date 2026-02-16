use crate::types::IglooType;
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Copy, Display, Serialize, Deserialize)]
pub enum PenguinPinType {
    /// execution flow
    #[display("Flow")]
    Flow,
    /// holds value
    #[display("Value({_0})")]
    Value(IglooType),
}

impl PenguinPinType {
    pub fn can_cast(self, to: Self) -> bool {
        match (self, to) {
            (PenguinPinType::Value(from), PenguinPinType::Value(to)) => from.can_cast(to),
            _ => false,
        }
    }

    pub fn cast_name(self, to: Self) -> Option<String> {
        match (self, to) {
            (PenguinPinType::Value(from), PenguinPinType::Value(to)) => from.cast_node_name(to),
            _ => None,
        }
    }

    pub fn can_connect_to(&self, target: PenguinPinType) -> bool {
        *self == target || self.can_cast(target)
    }

    pub fn stroke(&self) -> &str {
        match self {
            PenguinPinType::Flow => "white",
            PenguinPinType::Value(vt) => vt.color(),
        }
    }

    pub fn stroke_width(&self) -> u8 {
        match self {
            PenguinPinType::Flow => 4,
            PenguinPinType::Value(_) => 2,
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            PenguinPinType::Flow => "#ffffff",
            PenguinPinType::Value(vt) => vt.color(),
        }
    }
}
