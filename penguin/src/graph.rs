use crate::{ffi, types::*};
use dioxus::{logger::tracing, prelude::*};
use euclid::default::Point2D;
use std::collections::HashMap;
use web_sys::js_sys::Array;

#[derive(Debug, PartialEq, Store)]
pub struct Graph {
    pub nodes: HashMap<NodeID, Node>,
    pub wires: HashMap<WireID, Wire>,
}

impl Graph {
    pub fn delete_node(&mut self, id: NodeID) {
        self.nodes.remove(&id);
        self.wires
            .retain(|_, wire| wire.from_node != id && wire.to_node != id);
        ffi::delayedRerender();
    }

    pub fn delete_wire(&mut self, id: &WireID) {
        self.wires.remove(id);
        ffi::delayedRerender();
    }

    pub fn bulk_delete(&mut self, node_ids: Vec<u16>, wire_ids: Vec<u16>) {
        for wire_id in wire_ids {
            self.wires.remove(&WireID(wire_id));
        }

        for node_id in &node_ids {
            self.nodes.remove(&NodeID(*node_id));
        }

        self.wires.retain(|_, wire| {
            !node_ids.contains(&wire.from_node.0) && !node_ids.contains(&wire.to_node.0)
        });

        ffi::delayedRerender();
    }

    pub fn delete_selection(&mut self) {
        self.bulk_delete(ffi::getSelectedNodeIds(), ffi::getSelectedWireIds());
    }

    pub fn sync_from_js(&mut self) {
        let items = ffi::getAllNodePositions();

        for item in items {
            let item: Array = item.into();

            let Some(node_id) = item.get(0).as_f64() else {
                tracing::error!("Failed to parse node_id value ({:?}) as f64", item.get(0));
                continue;
            };
            let node_id = NodeID(node_id as u16);

            let Some(x) = item.get(1).as_f64() else {
                tracing::error!("Failed to parse x value ({:?}) as f64", item.get(1));
                continue;
            };

            let Some(y) = item.get(2).as_f64() else {
                tracing::error!("Failed to parse y value ({:?}) as f64", item.get(2));
                continue;
            };

            let Some(node) = self.nodes.get_mut(&node_id) else {
                tracing::error!("JS requested update for {node_id:?}, which doesn't exist");
                continue;
            };

            node.pos = Point2D::new(x, y);
        }
    }

    pub fn new() -> Self {
        Self {
            nodes: HashMap::from([
                (
                    NodeID(0),
                    Node {
                        title: "Add".to_string(),
                        pos: Point2D::new(50.0, 50.0),
                        inputs: HashMap::from([
                            (
                                PinID("A".to_string()),
                                PinType::Value {
                                    subtype: "number".to_string(),
                                    color: "#4CAF50".to_string(),
                                },
                            ),
                            (
                                PinID("B".to_string()),
                                PinType::Value {
                                    subtype: "number".to_string(),
                                    color: "#4CAF50".to_string(),
                                },
                            ),
                        ]),
                        outputs: HashMap::from([(
                            PinID("Result".to_string()),
                            PinType::Value {
                                subtype: "number".to_string(),
                                color: "#4CAF50".to_string(),
                            },
                        )]),
                    },
                ),
                (
                    NodeID(1),
                    Node {
                        title: "Print".to_string(),
                        pos: Point2D::new(350.0, 100.0),
                        inputs: HashMap::from([
                            (PinID("".to_string()), PinType::Flow),
                            (
                                PinID("Message".to_string()),
                                PinType::Value {
                                    subtype: "string".to_string(),
                                    color: "#2196F3".to_string(),
                                },
                            ),
                        ]),
                        outputs: HashMap::from([(PinID("".to_string()), PinType::Flow)]),
                    },
                ),
                (
                    NodeID(2),
                    Node {
                        title: "Add".to_string(),
                        pos: Point2D::new(50.0, 200.0),
                        inputs: HashMap::from([
                            (
                                PinID("A".to_string()),
                                PinType::Value {
                                    subtype: "number".to_string(),
                                    color: "#4CAF50".to_string(),
                                },
                            ),
                            (
                                PinID("B".to_string()),
                                PinType::Value {
                                    subtype: "number".to_string(),
                                    color: "#4CAF50".to_string(),
                                },
                            ),
                        ]),
                        outputs: HashMap::from([(
                            PinID("Result".to_string()),
                            PinType::Value {
                                subtype: "number".to_string(),
                                color: "#4CAF50".to_string(),
                            },
                        )]),
                    },
                ),
                (
                    NodeID(3),
                    Node {
                        title: "Print".to_string(),
                        pos: Point2D::new(750.0, 100.0),
                        inputs: HashMap::from([
                            (PinID("".to_string()), PinType::Flow),
                            (
                                PinID("Message".to_string()),
                                PinType::Value {
                                    subtype: "string".to_string(),
                                    color: "#2196F3".to_string(),
                                },
                            ),
                        ]),
                        outputs: HashMap::from([(PinID("".to_string()), PinType::Flow)]),
                    },
                ),
            ]),
            wires: HashMap::from([
                (
                    WireID(0),
                    Wire {
                        from_node: NodeID(1),
                        from_pin: PinID("".to_string()),
                        to_node: NodeID(3),
                        to_pin: PinID("".to_string()),
                        wire_type: PinType::Flow,
                    },
                ),
                (
                    WireID(1),
                    Wire {
                        from_node: NodeID(0),
                        from_pin: PinID("Result".to_string()),
                        to_node: NodeID(2),
                        to_pin: PinID("A".to_string()),
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
