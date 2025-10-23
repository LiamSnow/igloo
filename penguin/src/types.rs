use std::{collections::HashMap, rc::Rc};

use dioxus::prelude::*;
use euclid::default::Point2D;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(pub u16);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WireId(pub u16);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PinId(pub String);

#[derive(Debug, Store)]
pub struct Node {
    pub el: Option<Rc<MountedData>>,
    pub title: String,
    pub inputs: HashMap<PinId, Pin>,
    pub outputs: HashMap<PinId, Pin>,
    pub position: Point2D<f64>,
}

#[derive(Debug, Store)]
pub struct Pin {
    pub el: Option<Rc<MountedData>>,
    pub typ: PinType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PinType {
    Flow,
    Value { subtype: String, color: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PinRef {
    pub node_id: NodeId,
    pub pin_id: PinId,
}

#[derive(Debug, Store)]
pub struct Wire {
    pub el: Option<Rc<MountedData>>,
    pub from: PinRef,
    pub to: PinRef,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.inputs == other.inputs
            && self.outputs == other.outputs
            && self.position == other.position
    }
}

impl PartialEq for Pin {
    fn eq(&self, other: &Self) -> bool {
        self.typ == other.typ
    }
}

impl PartialEq for Wire {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to
    }
}
