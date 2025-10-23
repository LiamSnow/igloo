use std::collections::HashMap;

use dioxus::prelude::*;
use euclid::default::Point2D;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct NodeId(pub u16);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct WireId(pub u16);

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct PinId(pub String);

#[derive(Debug, Store, PartialEq)]
pub struct Node {
    pub title: String,
    pub inputs: HashMap<PinId, Pin>,
    pub outputs: HashMap<PinId, Pin>,
    pub position: Point2D<f64>,
}

#[derive(Debug, Store, PartialEq)]
pub struct Pin {
    pub typ: PinType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PinType {
    Flow,
    Value { subtype: String, color: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct PinRef {
    pub node_id: NodeId,
    pub pin_id: PinId,
}

#[derive(Clone, Debug, Store, PartialEq, Default)]
pub struct Wire {
    pub from_pin: PinRef,
    pub to_pin: PinRef,
    pub from_pos: Point2D<f64>,
    pub to_pos: Point2D<f64>,
    pub stroke: String,
    pub stroke_width: u8,
}
