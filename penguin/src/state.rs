// state.rs

use crate::types::*;
use dioxus::prelude::*;
use euclid::default::Point2D;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct ViewportState {
    pub pan: Point2D<f64>,
    pub zoom: f64,
}

impl ViewportState {
    pub fn new() -> Self {
        Self {
            pan: Point2D::new(0., 0.),
            zoom: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum InteractionMode {
    Idle,
    Panning {
        start: Point2D<f64>,
        has_moved: bool,
    },
    Dragging,
    BoxSelect {
        start: Point2D<f64>,
        has_moved: bool,
        append: bool,
    },
    Wiring {
        start: PinRef,
        is_output: bool,
        typ: PinType,
    },
}

#[derive(Clone, Debug, PartialEq, Store, Default)]
pub struct Selected {
    pub nodes: HashSet<NodeId>,
    pub wires: HashSet<WireId>,
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
                        position: Point2D::new(50.0, 50.0),
                        el: None,
                        inputs: HashMap::from([
                            (
                                PinId("A".to_string()),
                                Pin {
                                    el: None,
                                    typ: PinType::Value {
                                        subtype: "number".to_string(),
                                        color: "#4CAF50".to_string(),
                                    },
                                },
                            ),
                            (
                                PinId("B".to_string()),
                                Pin {
                                    el: None,
                                    typ: PinType::Value {
                                        subtype: "number".to_string(),
                                        color: "#4CAF50".to_string(),
                                    },
                                },
                            ),
                        ]),
                        outputs: HashMap::from([(
                            PinId("Result".to_string()),
                            Pin {
                                el: None,
                                typ: PinType::Value {
                                    subtype: "number".to_string(),
                                    color: "#4CAF50".to_string(),
                                },
                            },
                        )]),
                    },
                ),
                (
                    NodeId(1),
                    Node {
                        title: "Print".to_string(),
                        position: Point2D::new(350.0, 100.0),
                        el: None,
                        inputs: HashMap::from([
                            (
                                PinId("".to_string()),
                                Pin {
                                    el: None,
                                    typ: PinType::Flow,
                                },
                            ),
                            (
                                PinId("Message".to_string()),
                                Pin {
                                    el: None,
                                    typ: PinType::Value {
                                        subtype: "string".to_string(),
                                        color: "#2196F3".to_string(),
                                    },
                                },
                            ),
                        ]),
                        outputs: HashMap::from([(
                            PinId("".to_string()),
                            Pin {
                                el: None,
                                typ: PinType::Flow,
                            },
                        )]),
                    },
                ),
                (
                    NodeId(2),
                    Node {
                        title: "Add".to_string(),
                        position: Point2D::new(50.0, 200.0),
                        el: None,
                        inputs: HashMap::from([
                            (
                                PinId("A".to_string()),
                                Pin {
                                    el: None,
                                    typ: PinType::Value {
                                        subtype: "number".to_string(),
                                        color: "#4CAF50".to_string(),
                                    },
                                },
                            ),
                            (
                                PinId("B".to_string()),
                                Pin {
                                    el: None,
                                    typ: PinType::Value {
                                        subtype: "number".to_string(),
                                        color: "#4CAF50".to_string(),
                                    },
                                },
                            ),
                        ]),
                        outputs: HashMap::from([(
                            PinId("Result".to_string()),
                            Pin {
                                el: None,
                                typ: PinType::Value {
                                    subtype: "number".to_string(),
                                    color: "#4CAF50".to_string(),
                                },
                            },
                        )]),
                    },
                ),
                (
                    NodeId(3),
                    Node {
                        title: "Print".to_string(),
                        position: Point2D::new(750.0, 100.0),
                        el: None,
                        inputs: HashMap::from([
                            (
                                PinId("".to_string()),
                                Pin {
                                    el: None,
                                    typ: PinType::Flow,
                                },
                            ),
                            (
                                PinId("Message".to_string()),
                                Pin {
                                    el: None,
                                    typ: PinType::Value {
                                        subtype: "string".to_string(),
                                        color: "#2196F3".to_string(),
                                    },
                                },
                            ),
                        ]),
                        outputs: HashMap::from([(
                            PinId("".to_string()),
                            Pin {
                                el: None,
                                typ: PinType::Flow,
                            },
                        )]),
                    },
                ),
            ]),
            wires: HashMap::from([
                (
                    WireId(0),
                    Wire {
                        el: None,
                        from: PinRef {
                            node_id: NodeId(1),
                            pin_id: PinId("".to_string()),
                        },
                        to: PinRef {
                            node_id: NodeId(3),
                            pin_id: PinId("".to_string()),
                        },
                    },
                ),
                (
                    WireId(1),
                    Wire {
                        el: None,
                        from: PinRef {
                            node_id: NodeId(0),
                            pin_id: PinId("Result".to_string()),
                        },
                        to: PinRef {
                            node_id: NodeId(2),
                            pin_id: PinId("A".to_string()),
                        },
                    },
                ),
            ]),
        }
    }
}
