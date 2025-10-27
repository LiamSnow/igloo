use crate::{
    ffi::{self, Selection},
    state::WiringData,
    types::*,
};
use dioxus::{logger::tracing, prelude::*};
use euclid::default::Point2D;
use igloo_interface::{NodeDefnRef, PenguinRegistry, PinType, ValueType};
use std::collections::HashMap;
use web_sys::js_sys::Array;

#[derive(Debug, PartialEq, Store)]
pub struct Graph {
    pub nodes: HashMap<NodeID, Node>,
    pub wires: HashMap<WireID, Wire>,
}

impl Graph {
    pub fn delete(&mut self, selection: Selection) {
        for wire_id in selection.wire_ids {
            self.wires.remove(&WireID(wire_id));
        }

        for node_id in &selection.node_ids {
            self.nodes.remove(&NodeID(*node_id));
        }

        self.wires.retain(|_, wire| {
            !selection.node_ids.contains(&wire.from_node.0)
                && !selection.node_ids.contains(&wire.to_node.0)
        });

        ffi::delayedRerender();
    }

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

    pub fn complete_wire(
        &mut self,
        ws: WiringData,
        end_node: NodeID,
        end_pin: PinRef,
        end_type: PinType,
        end_is_out: bool,
    ) -> bool {
        if !ws.is_valid_end(end_node, end_type, end_is_out) {
            return false;
        }

        let (from_node, from_pin, to_node, to_pin) = ws.resolve_connection(end_node, end_pin);

        // remove existing wire
        let existing_wire_id = self
            .wires
            .iter()
            .find(|(_, wire)| wire.to_node == to_node && wire.to_pin == to_pin)
            .map(|(id, _)| *id);
        if let Some(id) = existing_wire_id {
            self.wires.remove(&id);
        }

        // normal connection
        if ws.wire_type == end_type {
            let next_id = self.wires.keys().map(|id| id.0).max().unwrap_or(0) + 1;
            self.wires.insert(
                WireID(next_id),
                Wire {
                    from_node,
                    from_pin,
                    to_node,
                    to_pin,
                    r#type: ws.wire_type,
                },
            );
        }
        // cast connection
        else {
            let cast_name = ws.cast_name(end_type).unwrap();

            let from_pos = self
                .nodes
                .get(&from_node)
                .map(|n| n.pos)
                .unwrap_or_default();
            let to_pos = self.nodes.get(&to_node).map(|n| n.pos).unwrap_or_default();
            let mid_pos =
                Point2D::new((from_pos.x + to_pos.x) / 2.0, (from_pos.y + to_pos.y) / 2.0);

            // make cast node
            let cast_node_id = NodeID(self.nodes.keys().map(|id| id.0).max().unwrap_or(0) + 1);
            self.nodes.insert(
                cast_node_id,
                Node {
                    defn_ref: NodeDefnRef::new("std", &cast_name),
                    pos: mid_pos,
                    phantom_state: HashMap::new(),
                    dynvalue_state: HashMap::new(),
                    value_inputs: HashMap::new(),
                    pin_values: HashMap::new(),
                },
            );

            let next_wire_id = self.wires.keys().map(|id| id.0).max().unwrap_or(0) + 1;

            // start node -> cast node
            self.wires.insert(
                WireID(next_wire_id),
                Wire {
                    from_node,
                    from_pin,
                    to_node: cast_node_id,
                    to_pin: PinRef::new(0),
                    r#type: ws.wire_type,
                },
            );

            // cast node -> end node
            self.wires.insert(
                WireID(next_wire_id + 1),
                Wire {
                    from_node: cast_node_id,
                    from_pin: PinRef::new(0),
                    to_node,
                    to_pin,
                    r#type: end_type,
                },
            );
        }

        ffi::delayedRerender();
        true
    }

    pub fn place_node(&mut self, defn_ref: NodeDefnRef, world_pos: Point2D<f64>) -> NodeID {
        let next_id = NodeID(self.nodes.keys().map(|id| id.0).max().unwrap_or(0) + 1);

        self.nodes.insert(
            next_id,
            Node {
                defn_ref,
                pos: world_pos,
                phantom_state: HashMap::new(),
                dynvalue_state: HashMap::new(),
                value_inputs: HashMap::new(),
                pin_values: HashMap::new(),
            },
        );

        ffi::delayedRerender();

        next_id
    }

    pub fn place_and_connect_node(
        &mut self,
        defn_ref: NodeDefnRef,
        world_pos: Point2D<f64>,
        pending_wire: Option<WiringData>,
        registry: &PenguinRegistry,
    ) -> NodeID {
        let new_node_id = self.place_node(defn_ref.clone(), world_pos);

        if let Some(ws) = pending_wire {
            if let Some(defn) = registry.get_defn(&defn_ref.library, &defn_ref.name) {
                if let Some((idx, pin_type)) = defn.find_compatible_inputs(ws.wire_type).first() {
                    let pin_ref = PinRef::new(*idx);
                    self.complete_wire(ws, new_node_id, pin_ref, *pin_type, false);
                }
            }
        }

        new_node_id
    }

    pub fn new() -> Self {
        Self {
            nodes: HashMap::from([
                (
                    NodeID(0),
                    Node {
                        defn_ref: NodeDefnRef::new("std", "on_start"),
                        pos: Point2D::new(50.0, 50.0),
                        ..Default::default()
                    },
                ),
                (
                    NodeID(1),
                    Node {
                        defn_ref: NodeDefnRef::new("std", "print"),
                        pos: Point2D::new(350.0, 100.0),
                        ..Default::default()
                    },
                ),
                (
                    NodeID(2),
                    Node {
                        defn_ref: NodeDefnRef::new("std", "const_text"),
                        pos: Point2D::new(50.0, 200.0),
                        ..Default::default()
                    },
                ),
                (
                    NodeID(3),
                    Node {
                        defn_ref: NodeDefnRef::new("std", "int_add"),
                        pos: Point2D::new(200.0, 300.0),
                        ..Default::default()
                    },
                ),
                (
                    NodeID(4),
                    Node {
                        defn_ref: NodeDefnRef::new("std", "const_bool"),
                        pos: Point2D::new(200.0, 500.0),
                        ..Default::default()
                    },
                ),
                (
                    NodeID(5),
                    Node {
                        defn_ref: NodeDefnRef::new("std", "const_int"),
                        pos: Point2D::new(0.0, 500.0),
                        ..Default::default()
                    },
                ),
                (
                    NodeID(6),
                    Node {
                        defn_ref: NodeDefnRef::new("std", "const_real"),
                        pos: Point2D::new(0.0, 600.0),
                        ..Default::default()
                    },
                ),
                (
                    NodeID(7),
                    Node {
                        defn_ref: NodeDefnRef::new("std", "branch"),
                        pos: Point2D::new(500.0, 500.0),
                        ..Default::default()
                    },
                ),
                (
                    NodeID(8),
                    Node {
                        defn_ref: NodeDefnRef::new("std", "merge"),
                        pos: Point2D::new(700.0, 500.0),
                        ..Default::default()
                    },
                ),
                (
                    NodeID(9),
                    Node {
                        defn_ref: NodeDefnRef::new("std", "comment"),
                        pos: Point2D::new(560.0, 380.0),
                        ..Default::default()
                    },
                ),
            ]),
            wires: HashMap::from([
                (
                    WireID(0),
                    Wire {
                        from_node: NodeID(0),
                        from_pin: PinRef::new(0),
                        to_node: NodeID(1),
                        to_pin: PinRef::new(0),
                        r#type: PinType::Flow,
                    },
                ),
                (
                    WireID(1),
                    Wire {
                        from_node: NodeID(2),
                        from_pin: PinRef::new(0),
                        to_node: NodeID(1),
                        to_pin: PinRef::new(1),
                        r#type: PinType::Value(ValueType::Text),
                    },
                ),
            ]),
        }
    }
}
