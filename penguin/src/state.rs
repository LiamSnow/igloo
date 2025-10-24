use crate::types::*;
use dioxus::prelude::*;
use euclid::default::Point2D;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct WiringData {
    pub start_node: NodeId,
    pub start_pin: PinId,
    pub is_output: bool,
    pub wire_type: PinType,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridSettings {
    pub enabled: bool,
    pub snap: bool,
    pub size: f64,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            snap: true,
            size: 20.0,
        }
    }
}

#[derive(Debug, PartialEq, Store)]
pub struct GraphState {
    pub nodes: HashMap<NodeId, Node>,
    pub wires: HashMap<WireId, Wire>,
}

impl GraphState {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::from([
                (
                    NodeId(0),
                    Node {
                        title: "Add".to_string(),
                        pos: Point2D::new(50.0, 50.0),
                        inputs: HashMap::from([
                            (
                                PinId("A".to_string()),
                                PinType::Value {
                                    subtype: "number".to_string(),
                                    color: "#4CAF50".to_string(),
                                },
                            ),
                            (
                                PinId("B".to_string()),
                                PinType::Value {
                                    subtype: "number".to_string(),
                                    color: "#4CAF50".to_string(),
                                },
                            ),
                        ]),
                        outputs: HashMap::from([(
                            PinId("Result".to_string()),
                            PinType::Value {
                                subtype: "number".to_string(),
                                color: "#4CAF50".to_string(),
                            },
                        )]),
                    },
                ),
                (
                    NodeId(1),
                    Node {
                        title: "Print".to_string(),
                        pos: Point2D::new(350.0, 100.0),
                        inputs: HashMap::from([
                            (PinId("".to_string()), PinType::Flow),
                            (
                                PinId("Message".to_string()),
                                PinType::Value {
                                    subtype: "string".to_string(),
                                    color: "#2196F3".to_string(),
                                },
                            ),
                        ]),
                        outputs: HashMap::from([(PinId("".to_string()), PinType::Flow)]),
                    },
                ),
                (
                    NodeId(2),
                    Node {
                        title: "Add".to_string(),
                        pos: Point2D::new(50.0, 200.0),
                        inputs: HashMap::from([
                            (
                                PinId("A".to_string()),
                                PinType::Value {
                                    subtype: "number".to_string(),
                                    color: "#4CAF50".to_string(),
                                },
                            ),
                            (
                                PinId("B".to_string()),
                                PinType::Value {
                                    subtype: "number".to_string(),
                                    color: "#4CAF50".to_string(),
                                },
                            ),
                        ]),
                        outputs: HashMap::from([(
                            PinId("Result".to_string()),
                            PinType::Value {
                                subtype: "number".to_string(),
                                color: "#4CAF50".to_string(),
                            },
                        )]),
                    },
                ),
                (
                    NodeId(3),
                    Node {
                        title: "Print".to_string(),
                        pos: Point2D::new(750.0, 100.0),
                        inputs: HashMap::from([
                            (PinId("".to_string()), PinType::Flow),
                            (
                                PinId("Message".to_string()),
                                PinType::Value {
                                    subtype: "string".to_string(),
                                    color: "#2196F3".to_string(),
                                },
                            ),
                        ]),
                        outputs: HashMap::from([(PinId("".to_string()), PinType::Flow)]),
                    },
                ),
            ]),
            wires: HashMap::from([
                (
                    WireId(0),
                    Wire {
                        from_node: NodeId(1),
                        from_pin: PinId("".to_string()),
                        to_node: NodeId(3),
                        to_pin: PinId("".to_string()),
                        wire_type: PinType::Flow,
                    },
                ),
                (
                    WireId(1),
                    Wire {
                        from_node: NodeId(0),
                        from_pin: PinId("Result".to_string()),
                        to_node: NodeId(2),
                        to_pin: PinId("A".to_string()),
                        wire_type: PinType::Value {
                            subtype: "number".to_string(),
                            color: "#4CAF50".to_string(),
                        },
                    },
                ),
            ]),
        }
    }
}
