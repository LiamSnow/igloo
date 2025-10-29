use borsh::{BorshDeserialize, BorshSerialize};

use crate::penguin::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize, PartialOrd, Ord)]
pub struct PinID(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinDefn {
    pub r#type: PinDefnType,
    pub hide_name: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinDefnType {
    Flow,
    Value(PenguinType),
    /// Resolved at runtime (ex. used for query)
    DynValue,
}

impl PinDefn {
    pub fn named(r#type: PinDefnType) -> Self {
        Self {
            r#type,
            hide_name: false,
        }
    }

    pub fn unnamed(r#type: PinDefnType) -> Self {
        Self {
            r#type,
            hide_name: true,
        }
    }

    pub fn named_flow() -> Self {
        Self::named(PinDefnType::Flow)
    }

    pub fn unnamed_flow() -> Self {
        Self::unnamed(PinDefnType::Flow)
    }

    pub fn named_dyn() -> Self {
        Self::named(PinDefnType::DynValue)
    }

    pub fn unnamed_dyn() -> Self {
        Self::unnamed(PinDefnType::DynValue)
    }

    pub fn named_val(t: PenguinType) -> Self {
        Self::named(PinDefnType::Value(t))
    }

    pub fn unnamed_val(t: PenguinType) -> Self {
        Self::unnamed(PinDefnType::Value(t))
    }
}

impl PinID {
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }
}
