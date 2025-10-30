use borsh::{BorshDeserialize, BorshSerialize};

use crate::penguin::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize, PartialOrd, Ord)]
pub struct PenguinPinID(pub String);

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

    pub fn named_val(t: PenguinType) -> Self {
        Self::named(PenguinPinType::Value(t))
    }

    pub fn unnamed_val(t: PenguinType) -> Self {
        Self::unnamed(PenguinPinType::Value(t))
    }
}

impl PenguinPinID {
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }
}
