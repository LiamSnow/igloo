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

#[derive(Debug, Clone, PartialEq)]
pub enum GraphCommand {
    AddNode {
        id: NodeID,
        node: Node,
    },
    DeleteNode {
        id: NodeID,
        node: Node,
        connected_wires: Vec<(WireID, Wire)>,
    },
    MoveNodes {
        /// (id, old_pos, new_pos)
        moves: Vec<(NodeID, Point2D<f64>, Point2D<f64>)>,
    },
    AddWire {
        id: WireID,
        wire: Wire,
    },
    DeleteWire {
        id: WireID,
        wire: Wire,
    },
    Batch {
        commands: Vec<GraphCommand>,
    },
}

#[derive(Debug, PartialEq, Store, Default)]
pub struct Graph {
    pub nodes: HashMap<NodeID, Node>,
    pub wires: HashMap<WireID, Wire>,
    undo_stack: Vec<GraphCommand>,
    redo_stack: Vec<GraphCommand>,
}

impl GraphCommand {
    pub fn execute(&self, graph: &mut Graph) {
        match self {
            GraphCommand::AddNode { id, node } => {
                web_sys::console::time_with_label("insert_node");
                graph.nodes.insert(*id, node.clone());
                web_sys::console::time_with_label("insert_node");
            }
            GraphCommand::DeleteNode { id, .. } => {
                graph.nodes.remove(id);
                graph
                    .wires
                    .retain(|_, wire| wire.from_node != *id && wire.to_node != *id);
            }
            GraphCommand::MoveNodes { moves } => {
                for (id, _, new_pos) in moves {
                    if let Some(node) = graph.nodes.get_mut(id) {
                        node.x = new_pos.x;
                        node.y = new_pos.y;
                    }
                }
            }
            GraphCommand::AddWire { id, wire } => {
                graph.wires.insert(*id, wire.clone());
            }
            GraphCommand::DeleteWire { id, .. } => {
                graph.wires.remove(id);
            }
            GraphCommand::Batch { commands } => {
                for cmd in commands {
                    cmd.execute(graph);
                }
            }
        }
        ffi::delayedRerender();
    }

    pub fn undo(&self, graph: &mut Graph) {
        match self {
            GraphCommand::AddNode { id, .. } => {
                graph.nodes.remove(id);
                graph
                    .wires
                    .retain(|_, wire| wire.from_node != *id && wire.to_node != *id);
            }
            GraphCommand::DeleteNode {
                id,
                node,
                connected_wires,
            } => {
                graph.nodes.insert(*id, node.clone());
                for (wire_id, wire) in connected_wires {
                    graph.wires.insert(*wire_id, wire.clone());
                }
            }
            GraphCommand::MoveNodes { moves } => {
                for (id, old_pos, _) in moves {
                    if let Some(node) = graph.nodes.get_mut(id) {
                        node.x = old_pos.x;
                        node.y = old_pos.y;
                    }
                }
            }
            GraphCommand::AddWire { id, .. } => {
                graph.wires.remove(id);
            }
            GraphCommand::DeleteWire { id, wire } => {
                graph.wires.insert(*id, wire.clone());
            }
            GraphCommand::Batch { commands } => {
                for cmd in commands.iter().rev() {
                    cmd.undo(graph);
                }
            }
        }
        ffi::delayedRerender();
    }
}

impl Graph {
    pub fn undo(&mut self) -> bool {
        if let Some(cmd) = self.undo_stack.pop() {
            cmd.undo(self);
            self.redo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(cmd) = self.redo_stack.pop() {
            cmd.execute(self);
            self.undo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    fn push_command(&mut self, cmd: GraphCommand) {
        cmd.execute(self);
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
    }

    pub fn delete(&mut self, selection: Selection) {
        let mut batch = Vec::new();

        for wire_id in selection.wire_ids {
            let wire_id = WireID(wire_id);
            if let Some(wire) = self.wires.get(&wire_id) {
                batch.push(GraphCommand::DeleteWire {
                    id: wire_id,
                    wire: wire.clone(),
                });
            }
        }

        for node_id in &selection.node_ids {
            let node_id = NodeID(*node_id);
            if let Some(node) = self.nodes.get(&node_id) {
                let connected_wires: Vec<_> = self
                    .wires
                    .iter()
                    .filter(|(_, wire)| wire.from_node == node_id || wire.to_node == node_id)
                    .map(|(id, wire)| (*id, wire.clone()))
                    .collect();

                batch.push(GraphCommand::DeleteNode {
                    id: node_id,
                    node: node.clone(),
                    connected_wires,
                });
            }
        }

        if !batch.is_empty() {
            self.push_command(GraphCommand::Batch { commands: batch });
        }
    }

    pub fn delete_node(&mut self, id: NodeID) {
        if let Some(node) = self.nodes.get(&id) {
            let connected_wires: Vec<_> = self
                .wires
                .iter()
                .filter(|(_, wire)| wire.from_node == id || wire.to_node == id)
                .map(|(wire_id, wire)| (*wire_id, wire.clone()))
                .collect();

            self.push_command(GraphCommand::DeleteNode {
                id,
                node: node.clone(),
                connected_wires,
            });
        }
    }

    pub fn delete_wire(&mut self, id: &WireID) {
        if let Some(wire) = self.wires.get(id) {
            self.push_command(GraphCommand::DeleteWire {
                id: *id,
                wire: wire.clone(),
            });
        }
    }

    pub fn commit_node_moves(&mut self) {
        let items = ffi::getAllNodePositions();
        let mut moves = Vec::new();

        for item in items {
            let item: Array = item.into();

            let Some(node_id) = item.get(0).as_f64() else {
                tracing::error!("Failed to parse node_id value ({:?}) as f64", item.get(0));
                continue;
            };
            let node_id = NodeID(node_id as u16);

            let Some(new_x) = item.get(1).as_f64() else {
                tracing::error!("Failed to parse x value ({:?}) as f64", item.get(1));
                continue;
            };

            let Some(new_y) = item.get(2).as_f64() else {
                tracing::error!("Failed to parse y value ({:?}) as f64", item.get(2));
                continue;
            };

            let Some(node) = self.nodes.get(&node_id) else {
                tracing::error!("JS requested update for {node_id:?}, which doesn't exist");
                continue;
            };
            let old_pos = Point2D::new(node.x, node.y);
            let new_pos = Point2D::new(new_x, new_y);

            if old_pos != new_pos {
                moves.push((node_id, old_pos, new_pos));
            }
        }

        if !moves.is_empty() {
            self.push_command(GraphCommand::MoveNodes { moves });
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

        let mut batch = Vec::new();

        // remove existing wire on the input
        if let Some((existing_id, existing_wire)) = self
            .wires
            .iter()
            .find(|(_, wire)| wire.to_node == to_node && wire.to_pin == to_pin)
        {
            batch.push(GraphCommand::DeleteWire {
                id: *existing_id,
                wire: existing_wire.clone(),
            });
        }

        // normal connection
        if ws.wire_type == end_type {
            let next_id = WireID(self.wires.keys().map(|id| id.0).max().unwrap_or(0) + 1);
            batch.push(GraphCommand::AddWire {
                id: next_id,
                wire: Wire {
                    from_node,
                    from_pin,
                    to_node,
                    to_pin,
                    r#type: ws.wire_type,
                },
            });
        }
        // cast connection
        else {
            let cast_name = ws.cast_name(end_type).unwrap();

            let from_pos = self
                .nodes
                .get(&from_node)
                .map(|n| n.pos())
                .unwrap_or_default();
            let to_pos = self
                .nodes
                .get(&to_node)
                .map(|n| n.pos())
                .unwrap_or_default();
            let mid_pos =
                Point2D::new((from_pos.x + to_pos.x) / 2.0, (from_pos.y + to_pos.y) / 2.0);

            let cast_node_id = NodeID(self.nodes.keys().map(|id| id.0).max().unwrap_or(0) + 1);
            batch.push(GraphCommand::AddNode {
                id: cast_node_id,
                node: Node {
                    defn_ref: NodeDefnRef::new("std", &cast_name),
                    x: mid_pos.x,
                    y: mid_pos.y,
                    phantom_state: HashMap::new(),
                    dynvalue_state: HashMap::new(),
                    value_inputs: HashMap::new(),
                    pin_values: HashMap::new(),
                },
            });

            let next_wire_id = self.wires.keys().map(|id| id.0).max().unwrap_or(0) + 1;

            batch.push(GraphCommand::AddWire {
                id: WireID(next_wire_id),
                wire: Wire {
                    from_node,
                    from_pin,
                    to_node: cast_node_id,
                    to_pin: PinRef::new(0),
                    r#type: ws.wire_type,
                },
            });

            batch.push(GraphCommand::AddWire {
                id: WireID(next_wire_id + 1),
                wire: Wire {
                    from_node: cast_node_id,
                    from_pin: PinRef::new(0),
                    to_node,
                    to_pin,
                    r#type: end_type,
                },
            });
        }

        self.push_command(GraphCommand::Batch { commands: batch });
        true
    }

    pub fn place_node(&mut self, defn_ref: NodeDefnRef, world_pos: Point2D<f64>) -> NodeID {
        let next_id = NodeID(self.nodes.keys().map(|id| id.0).max().unwrap_or(0) + 1);

        self.push_command(GraphCommand::AddNode {
            id: next_id,
            node: Node {
                defn_ref,
                x: world_pos.x,
                y: world_pos.y,
                phantom_state: HashMap::new(),
                dynvalue_state: HashMap::new(),
                value_inputs: HashMap::new(),
                pin_values: HashMap::new(),
            },
        });

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
                    let pin_ref = PinRef::new(*idx as u32);
                    self.complete_wire(ws, new_node_id, pin_ref, *pin_type, false);
                }
            }
        }

        new_node_id
    }

    pub fn copy(&self, selection: &Selection, cursor_pos: Point2D<f64>) -> Result<String, String> {
        let mut nodes = HashMap::new();
        for node_id in &selection.node_ids {
            let node_id = NodeID(*node_id);
            if let Some(node) = self.nodes.get(&node_id) {
                let mut cloned = node.clone();
                cloned.x -= cursor_pos.x;
                cloned.y -= cursor_pos.y;
                nodes.insert(node_id, cloned);
            }
        }

        let wires: Vec<Wire> = self
            .wires
            .values()
            .filter(|wire| nodes.contains_key(&wire.from_node) && nodes.contains_key(&wire.to_node))
            .cloned()
            .collect();

        let clip = PenguinClipboard { nodes, wires };

        let bytes = borsh::to_vec(&clip)
            .map_err(|e| format!("Borsh serialization of clipboard failed: {e}"))?;

        Ok(base64::encode(&bytes))
    }

    pub fn paste(&mut self, clipboard_data: &str, cursor_pos: Point2D<f64>) -> Result<(), String> {
        let bytes =
            base64::decode(clipboard_data).map_err(|e| format!("Base64 decode failed: {}", e))?;

        let clip: PenguinClipboard = borsh::from_slice(&bytes)
            .map_err(|e| format!("Borsh deserialization failed: {}", e))?;

        let next_node_id = self.nodes.keys().map(|id| id.0).max().unwrap_or(0) + 1;
        let mut id_map = HashMap::new();

        for (i, old_id) in clip.nodes.keys().enumerate() {
            id_map.insert(*old_id, NodeID(next_node_id + i as u16));
        }

        let mut batch = Vec::new();

        for (old_id, node) in clip.nodes {
            let new_id = id_map[&old_id];
            let mut new_node = node;
            new_node.x += cursor_pos.x;
            new_node.y += cursor_pos.y;
            batch.push(GraphCommand::AddNode {
                id: new_id,
                node: new_node,
            });
        }

        let next_wire_id = self.wires.keys().map(|id| id.0).max().unwrap_or(0) + 1;

        for (i, wire) in clip.wires.into_iter().enumerate() {
            let new_wire = Wire {
                from_node: id_map[&wire.from_node],
                from_pin: wire.from_pin,
                to_node: id_map[&wire.to_node],
                to_pin: wire.to_pin,
                r#type: wire.r#type,
            };
            batch.push(GraphCommand::AddWire {
                id: WireID(next_wire_id + i as u16),
                wire: new_wire,
            });
        }

        self.push_command(GraphCommand::Batch { commands: batch });

        Ok(())
    }

    pub fn new() -> Self {
        let mut me = Self {
            // nodes: HashMap::from([
            //     (
            //         NodeID(0),
            //         Node {
            //             defn_ref: NodeDefnRef::new("std", "on_start"),
            //             x: 50.0,
            //             y: 50.0,
            //             ..Default::default()
            //         },
            //     ),
            //     (
            //         NodeID(1),
            //         Node {
            //             defn_ref: NodeDefnRef::new("std", "print"),
            //             x: 350.0,
            //             y: 100.0,
            //             ..Default::default()
            //         },
            //     ),
            //     (
            //         NodeID(2),
            //         Node {
            //             defn_ref: NodeDefnRef::new("std", "const_text"),
            //             x: 50.0,
            //             y: 200.0,
            //             ..Default::default()
            //         },
            //     ),
            //     (
            //         NodeID(3),
            //         Node {
            //             defn_ref: NodeDefnRef::new("std", "int_add"),
            //             x: 200.0,
            //             y: 300.0,
            //             ..Default::default()
            //         },
            //     ),
            //     (
            //         NodeID(4),
            //         Node {
            //             defn_ref: NodeDefnRef::new("std", "const_bool"),
            //             x: 200.0,
            //             y: 500.0,
            //             ..Default::default()
            //         },
            //     ),
            //     (
            //         NodeID(5),
            //         Node {
            //             defn_ref: NodeDefnRef::new("std", "const_int"),
            //             x: 0.0,
            //             y: 500.0,
            //             ..Default::default()
            //         },
            //     ),
            //     (
            //         NodeID(6),
            //         Node {
            //             defn_ref: NodeDefnRef::new("std", "const_real"),
            //             x: 0.0,
            //             y: 600.0,
            //             ..Default::default()
            //         },
            //     ),
            //     (
            //         NodeID(7),
            //         Node {
            //             defn_ref: NodeDefnRef::new("std", "branch"),
            //             x: 500.0,
            //             y: 500.0,
            //             ..Default::default()
            //         },
            //     ),
            //     (
            //         NodeID(8),
            //         Node {
            //             defn_ref: NodeDefnRef::new("std", "merge"),
            //             x: 700.0,
            //             y: 500.0,
            //             ..Default::default()
            //         },
            //     ),
            //     (
            //         NodeID(9),
            //         Node {
            //             defn_ref: NodeDefnRef::new("std", "comment"),
            //             x: 560.0,
            //             y: 380.0,
            //             ..Default::default()
            //         },
            //     ),
            // ]),
            // wires: HashMap::from([
            //     (
            //         WireID(0),
            //         Wire {
            //             from_node: NodeID(0),
            //             from_pin: PinRef::new(0),
            //             to_node: NodeID(1),
            //             to_pin: PinRef::new(0),
            //             r#type: PinType::Flow,
            //         },
            //     ),
            //     (
            //         WireID(1),
            //         Wire {
            //             from_node: NodeID(2),
            //             from_pin: PinRef::new(0),
            //             to_node: NodeID(1),
            //             to_pin: PinRef::new(1),
            //             r#type: PinType::Value(ValueType::Text),
            //         },
            //     ),
            // ]),
            ..Default::default()
        };

        let mut x = 0.;
        let mut y = 0.;

        for i in 0..500 {
            if i % 10 == 0 {
                x = 0.;
                y += 200.;
            } else {
                x += 250.;
            }

            me.nodes.insert(
                NodeID(i),
                Node {
                    defn_ref: NodeDefnRef::new("std", "int_add"),
                    x,
                    y,
                    ..Default::default()
                },
            );
        }

        me
    }
}
