use std::collections::HashMap;

use dioxus::prelude::*;
use euclid::default::Point2D;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct NodeID(pub u16);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct WireID(pub u16);

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct PinID(pub String);

#[derive(Debug, Store, PartialEq)]
pub struct Node {
    pub title: String,
    pub inputs: HashMap<PinID, PinType>,
    pub outputs: HashMap<PinID, PinType>,
    pub pos: Point2D<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PinType {
    Flow,
    Value { subtype: String, color: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Wire {
    pub from_node: NodeID,
    pub from_pin: PinID,
    pub to_node: NodeID,
    pub to_pin: PinID,
    pub wire_type: PinType,
}

impl PinType {
    pub fn stroke(&self) -> &str {
        match self {
            PinType::Value { color, .. } => color,
            _ => "white",
        }
    }

    pub fn stroke_width(&self) -> u8 {
        match self {
            PinType::Flow => 4,
            _ => 2,
        }
    }
}
