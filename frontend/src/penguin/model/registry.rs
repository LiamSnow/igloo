use std::collections::{HashMap, HashSet};

use crate::penguin::coordinates::{WorldPoint, WorldRect, WorldSize};

pub type NodeRegistry = HashMap<u32, NodeDefn>;

#[derive(Clone, PartialEq, Eq)]
pub struct NodeDefn {
    pub icon: String,
    pub title: String,
    pub desc: String,
    pub vio: NodeVIODefn,
    pub typ: NodeType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeType {
    /// execution start
    ///  - inputs:  0 flow, N values
    ///  - outputs: 1 flow, N values
    Trigger,

    /// performers an action
    ///  - inputs:  1 flow, N values
    ///  - outputs: 1 flow, N values
    Action,

    /// modifies execution flow
    ///  - inputs:  N flow, N values
    ///  - outputs: N flow, N values
    /// (# inp flows, # out)
    ControlFlow(u8, u8),

    /// standard math definition of a function
    ///  - inputs:  0 flow, N values
    ///  - outputs: 0 flow, N values
    Function,

    /// can be variable
    ///  - inputs:  0 flow, 0 values
    ///  - outputs: 0 flow, 1 values
    Variable,

    /// can be variable or constant
    ///  - inputs:  0 flow, 0 values
    ///  - outputs: 0 flow, 1 values
    Constant,
}

#[derive(Clone, PartialEq, Eq)]
pub struct NodeVIODefn {
    pub inp: Vec<(String, ValueType)>,
    pub out: Vec<(String, ValueType)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    Number,
    String,
    Boolean,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PinType {
    Flow,
    Value(ValueType),
}

impl NodeType {
    /// returns (# input pins, # output pins)
    pub fn get_flow_pin_counts(&self) -> (u8, u8) {
        match &self {
            NodeType::Trigger => (0, 1),
            NodeType::Action => (1, 1),
            NodeType::ControlFlow(inp, out) => (*inp, *out),
            NodeType::Function => (0, 0),
            NodeType::Variable => (0, 0),
            NodeType::Constant => (0, 0),
        }
    }
}

impl ValueType {
    pub fn color(&self) -> &'static str {
        match self {
            ValueType::Number => "#3498db",  // blue
            ValueType::String => "#9b59b6",  // purple
            ValueType::Boolean => "#e74c3c", // red
        }
    }
}

impl PinType {
    pub fn is_flow(&self) -> bool {
        matches!(self, PinType::Flow)
    }

    #[allow(dead_code)]
    pub fn is_value(&self) -> bool {
        matches!(self, PinType::Value(_))
    }

    pub fn color(&self) -> &'static str {
        match self {
            PinType::Flow => "#ffffff",
            PinType::Value(value_type) => value_type.color(),
        }
    }

    pub fn is_compatible_with(&self, other: &PinType) -> bool {
        match (self, other) {
            (PinType::Flow, PinType::Flow) => true,
            (PinType::Value(a), PinType::Value(b)) => a == b,
            _ => false,
        }
    }
}
