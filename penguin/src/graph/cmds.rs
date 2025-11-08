use std::collections::HashSet;

use crate::{
    graph::{WebGraph, input::WebInputType, node::WebNode, wire::WebWire},
    viewport::WorldPoint,
};
use igloo_interface::penguin::graph::{PenguinNode, PenguinNodeID, PenguinWire, PenguinWireID};

const MAX_HISTORY: usize = 500;

#[derive(Debug, Clone)]
pub enum Command {
    AddNode {
        id: PenguinNodeID,
        node: PenguinNode,
    },
    DeleteNode {
        id: PenguinNodeID,
        node: PenguinNode,
    },
    AddWire {
        id: PenguinWireID,
        wire: PenguinWire,
    },
    DeleteWire {
        id: PenguinWireID,
        wire: PenguinWire,
    },
    MoveNodes {
        moves: Vec<(PenguinNodeID, WorldPoint, WorldPoint)>,
    },
    ChangeNodeInput {
        node_id: PenguinNodeID,
        r#type: WebInputType,
        old_value: String,
        new_value: String,
    },
    ResizeNodeInput {
        node_id: PenguinNodeID,
        r#type: WebInputType,
        old_size: (i32, i32),
        new_size: (i32, i32),
    },
    ResizeNodeSection {
        node_id: PenguinNodeID,
        old_size: (i32, i32),
        new_size: (i32, i32),
    },
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub commands: Vec<Command>,
}

impl Transaction {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            commands: Vec::with_capacity(capacity),
        }
    }

    pub fn single(cmd: Command) -> Self {
        Self {
            commands: vec![cmd],
        }
    }

    pub fn push(&mut self, cmd: Command) {
        self.commands.push(cmd);
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl WebGraph {
    pub fn execute(&mut self, tx: Transaction) {
        if tx.is_empty() {
            return;
        }

        // squash Command::*NodeInput
        if tx.commands.len() == 1 && self.try_squash_command(&tx.commands[0]) {
            return;
        }

        self.apply_transaction(&tx, false);
        self.past.push(tx);
        self.future.clear();

        if self.past.len() > MAX_HISTORY {
            self.past.remove(0);
        }
    }

    pub fn undo(&mut self) {
        let Some(tx) = self.past.pop() else {
            return;
        };

        let reversed_tx = Transaction {
            commands: tx.commands.iter().rev().map(|cmd| cmd.reverse()).collect(),
        };

        self.apply_transaction(&reversed_tx, true);
        self.future.push(tx);
    }

    pub fn redo(&mut self) {
        let Some(tx) = self.future.pop() else {
            return;
        };

        self.apply_transaction(&tx, true);
        self.past.push(tx);
    }

    fn apply_transaction(&mut self, tx: &Transaction, historical: bool) {
        let mut dirty_nodes = HashSet::new();
        let mut dirty_wires = HashSet::new();

        for cmd in &tx.commands {
            self.apply_command(cmd, historical, &mut dirty_nodes, &mut dirty_wires);
        }

        self.clean(dirty_nodes, dirty_wires);
    }

    fn clean(&mut self, dirty_nodes: HashSet<PenguinNodeID>, dirty_wires: HashSet<PenguinWireID>) {
        for node_id in &dirty_nodes {
            if let Some(node) = self.nodes.get_mut(node_id) {
                node.cache_pin_offsets(&self.ctw);
            }
        }

        for wire_id in &dirty_wires {
            self.redraw_wire(wire_id);
        }
    }

    pub(super) fn apply_command(
        &mut self,
        cmd: &Command,
        historical: bool,
        dirty_nodes: &mut HashSet<PenguinNodeID>,
        dirty_wires: &mut HashSet<PenguinWireID>,
    ) {
        match cmd {
            Command::AddNode { id, node } => {
                let web_node =
                    WebNode::new(&self.nodes_el, &self.registry, None, node.clone(), *id);
                self.nodes.insert(*id, web_node);
            }

            Command::DeleteNode { id, .. } => {
                self.nodes.remove(id);
            }

            Command::AddWire { id, wire } => {
                let web_wire = WebWire::new(&self.wires_el, *id, wire.clone());

                dirty_wires.insert(*id);

                self.wires.insert(*id, web_wire);

                if let Some(from_node) = self.nodes.get_mut(&wire.from_node)
                    && let Some(from_pin) = from_node.outputs.get_mut(&wire.from_pin)
                    && from_pin.add_connection(*id)
                {
                    dirty_nodes.insert(wire.from_node);
                    for wire_id in from_node.connections() {
                        dirty_wires.insert(wire_id);
                    }
                }
                if let Some(to_node) = self.nodes.get_mut(&wire.to_node)
                    && let Some(to_pin) = to_node.inputs.get_mut(&wire.to_pin)
                    && to_pin.add_connection(*id)
                {
                    dirty_nodes.insert(wire.to_node);
                    for wire_id in to_node.connections() {
                        dirty_wires.insert(wire_id);
                    }
                }
            }

            Command::DeleteWire { id, wire } => {
                if self.wires.remove(id).is_none() {
                    return;
                }

                if let Some(from_node) = self.nodes.get_mut(&wire.from_node)
                    && let Some(from_pin) = from_node.outputs.get_mut(&wire.from_pin)
                    && from_pin.remove_connection(*id)
                {
                    dirty_nodes.insert(wire.from_node);
                    for wire_id in from_node.connections() {
                        dirty_wires.insert(wire_id);
                    }
                }
                if let Some(to_node) = self.nodes.get_mut(&wire.to_node)
                    && let Some(to_pin) = to_node.inputs.get_mut(&wire.to_pin)
                    && to_pin.remove_connection(*id)
                {
                    dirty_nodes.insert(wire.to_node);
                    for wire_id in to_node.connections() {
                        dirty_wires.insert(wire_id);
                    }
                }
            }

            Command::MoveNodes { moves } => {
                for (id, _old_pos, new_pos) in moves {
                    if let Some(node) = self.nodes.get_mut(id) {
                        node.set_pos(*new_pos);
                        for wire_id in node.connections() {
                            dirty_wires.insert(wire_id);
                        }
                    }
                }
            }

            Command::ChangeNodeInput {
                node_id,
                r#type,
                new_value,
                ..
            } => {
                if let Some(node) = self.nodes.get_mut(node_id) {
                    let iv = match r#type {
                        WebInputType::Pin(pin_id) => node.inner.input_pin_values.get_mut(pin_id),
                        WebInputType::NodeFeature(feature_id) => {
                            node.inner.input_feature_values.get_mut(feature_id)
                        }
                    };

                    if let Some(iv) = iv {
                        iv.set_from_string(new_value.clone());
                    }

                    if historical {
                        node.update_input_value(r#type, new_value);
                    }
                }
            }

            Command::ResizeNodeInput {
                node_id,
                r#type,
                new_size,
                ..
            } => {
                log::info!("resize input");
                if let Some(node) = self.nodes.get_mut(node_id) {
                    let iv = match r#type {
                        WebInputType::Pin(pin_id) => node.inner.input_pin_values.get_mut(pin_id),
                        WebInputType::NodeFeature(feature_id) => {
                            node.inner.input_feature_values.get_mut(feature_id)
                        }
                    };

                    if let Some(iv) = iv {
                        iv.size = Some(*new_size);
                    }

                    if historical {
                        node.update_input_size(r#type, *new_size);
                    }

                    dirty_nodes.insert(*node_id);
                    for wire_id in node.connections() {
                        dirty_wires.insert(wire_id);
                    }
                }
            }

            Command::ResizeNodeSection {
                node_id, new_size, ..
            } => {
                log::info!("resize section");
                if let Some(node) = self.nodes.get_mut(node_id) {
                    node.inner.size = Some(*new_size);

                    if historical {
                        node.update_size(*new_size);
                    }

                    dirty_nodes.insert(*node_id);
                    for wire_id in node.connections() {
                        dirty_wires.insert(wire_id);
                    }
                }
            }
        }
    }

    pub fn redraw_wire(&mut self, wire_id: &PenguinWireID) {
        let Some(wire) = self.wires.get_mut(wire_id) else {
            return;
        };

        let from_node_pos = self.nodes.get(&wire.inner.from_node).unwrap().pos();
        let to_node_pos = self.nodes.get(&wire.inner.to_node).unwrap().pos();

        let from_pin = self
            .nodes
            .get(&wire.inner.from_node)
            .unwrap()
            .outputs
            .get(&wire.inner.from_pin)
            .unwrap();
        let to_pin = self
            .nodes
            .get(&wire.inner.to_node)
            .unwrap()
            .inputs
            .get(&wire.inner.to_pin)
            .unwrap();

        let from_pos = from_node_pos + from_pin.node_offset;
        let to_pos = to_node_pos + to_pin.node_offset;

        wire.from = from_pos;
        wire.to = to_pos;
        wire.redraw();
    }

    fn try_squash_command(&mut self, cmd: &Command) -> bool {
        let should_squash = if let Some(last_tx) = self.past.last() {
            if last_tx.commands.len() == 1 {
                commands_are_squashable(&last_tx.commands[0], cmd)
            } else {
                false
            }
        } else {
            false
        };

        if !should_squash {
            return false;
        }

        let mut dirty_nodes = HashSet::new();
        let mut dirty_wires = HashSet::new();

        self.apply_command(cmd, false, &mut dirty_nodes, &mut dirty_wires);

        if let Some(last_tx) = self.past.last_mut() {
            squash_commands(&mut last_tx.commands[0], cmd);
        }

        self.clean(dirty_nodes, dirty_wires);

        true
    }
}

fn commands_are_squashable(last: &Command, new: &Command) -> bool {
    match (last, new) {
        (
            Command::ChangeNodeInput {
                node_id: id1,
                r#type: type1,
                ..
            },
            Command::ChangeNodeInput {
                node_id: id2,
                r#type: type2,
                ..
            },
        ) => id1 == id2 && type1 == type2,
        (
            Command::ResizeNodeInput {
                node_id: id1,
                r#type: type1,
                ..
            },
            Command::ResizeNodeInput {
                node_id: id2,
                r#type: type2,
                ..
            },
        ) => id1 == id2 && type1 == type2,
        _ => false,
    }
}

fn squash_commands(last: &mut Command, new: &Command) {
    match (last, new) {
        (
            Command::ChangeNodeInput {
                new_value: last_new,
                ..
            },
            Command::ChangeNodeInput { new_value, .. },
        ) => {
            *last_new = new_value.clone();
        }
        (
            Command::ResizeNodeInput {
                new_size: last_new, ..
            },
            Command::ResizeNodeInput { new_size, .. },
        ) => {
            *last_new = *new_size;
        }
        _ => unreachable!(),
    }
}

impl Command {
    pub fn reverse(&self) -> Command {
        match self {
            Command::AddNode { id, node } => Command::DeleteNode {
                id: *id,
                node: node.clone(),
            },
            Command::DeleteNode { id, node } => Command::AddNode {
                id: *id,
                node: node.clone(),
            },
            Command::AddWire { id, wire } => Command::DeleteWire {
                id: *id,
                wire: wire.clone(),
            },
            Command::DeleteWire { id, wire } => Command::AddWire {
                id: *id,
                wire: wire.clone(),
            },
            Command::MoveNodes { moves } => Command::MoveNodes {
                moves: moves
                    .iter()
                    .map(|(id, old_pos, new_pos)| (*id, *new_pos, *old_pos))
                    .collect(),
            },
            Command::ChangeNodeInput {
                node_id,
                r#type,
                old_value,
                new_value,
            } => Command::ChangeNodeInput {
                node_id: *node_id,
                r#type: r#type.clone(),
                old_value: new_value.clone(),
                new_value: old_value.clone(),
            },
            Command::ResizeNodeInput {
                node_id,
                r#type,
                old_size,
                new_size,
            } => Command::ResizeNodeInput {
                node_id: *node_id,
                r#type: r#type.clone(),
                old_size: *new_size,
                new_size: *old_size,
            },
            Command::ResizeNodeSection {
                node_id,
                old_size,
                new_size,
            } => Command::ResizeNodeSection {
                node_id: *node_id,
                old_size: *new_size,
                new_size: *old_size,
            },
        }
    }
}
