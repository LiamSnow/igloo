use crate::{
    graph::{WebGraph, input::WebInputType, node::WebNode, wire::WebWire},
    viewport::WorldPoint,
};
use igloo_interface::graph::{PenguinNode, PenguinNodeID, PenguinWire, PenguinWireID};
use wasm_bindgen::JsValue;

const MAX_HISTORY: usize = 100;

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
    pub fn execute(&mut self, tx: Transaction) -> Result<(), JsValue> {
        if tx.is_empty() {
            return Ok(());
        }

        // squash Command::*NodeInput
        if tx.commands.len() == 1 && self.try_squash_command(&tx.commands[0])? {
            return Ok(());
        }

        self.apply_transaction(&tx, false)?;
        self.past.push(tx);
        self.future.clear();

        if self.past.len() > MAX_HISTORY {
            self.past.remove(0);
        }

        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), JsValue> {
        let Some(tx) = self.past.pop() else {
            return Ok(());
        };

        let reversed_tx = Transaction {
            commands: tx.commands.iter().rev().map(|cmd| cmd.reverse()).collect(),
        };

        self.apply_transaction(&reversed_tx, true)?;
        self.future.push(tx);
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), JsValue> {
        let Some(tx) = self.future.pop() else {
            return Ok(());
        };

        self.apply_transaction(&tx, true)?;
        self.past.push(tx);
        Ok(())
    }

    fn apply_transaction(&mut self, tx: &Transaction, historical: bool) -> Result<(), JsValue> {
        for cmd in &tx.commands {
            self.apply_command(cmd, historical)?;
        }
        Ok(())
    }

    pub(super) fn apply_command(&mut self, cmd: &Command, historical: bool) -> Result<(), JsValue> {
        match cmd {
            Command::AddNode { id, node } => {
                let web_node =
                    WebNode::new(&self.nodes_el, &self.registry, None, node.clone(), *id)?;
                self.nodes.insert(*id, web_node);
            }

            Command::DeleteNode { id, .. } => {
                self.nodes.remove(id);
            }

            Command::AddWire { id, wire } => {
                let (from_hitbox, from_pos) = {
                    let from_node = self
                        .nodes
                        .get_mut(&wire.from_node)
                        .ok_or(JsValue::from_str("Missing from_node"))?;
                    let from_pin = from_node
                        .outputs
                        .get_mut(&wire.from_pin)
                        .ok_or(JsValue::from_str("Missing from_pin"))?;
                    from_pin.add_connection(*id)?;
                    (from_pin.hitbox.clone(), from_node.pos())
                };

                let (to_hitbox, to_pos) = {
                    let to_node = self
                        .nodes
                        .get_mut(&wire.to_node)
                        .ok_or(JsValue::from_str("Missing to_node"))?;
                    let to_pin = to_node
                        .inputs
                        .get_mut(&wire.to_pin)
                        .ok_or(JsValue::from_str("Missing to_pin"))?;
                    to_pin.add_connection(*id)?;
                    (to_pin.hitbox.clone(), to_node.pos())
                };

                let mut web_wire =
                    WebWire::new(&self.wires_el, *id, wire.clone(), from_hitbox, to_hitbox)?;

                web_wire.redraw_from(from_pos)?;
                web_wire.redraw_to(to_pos)?;

                self.redraw_node_wires(&wire.to_node)?;
                self.redraw_node_wires(&wire.from_node)?;

                self.wires.insert(*id, web_wire);
            }

            Command::DeleteWire { id, wire } => {
                if self.wires.remove(id).is_some() {
                    if let Some(from_node) = self.nodes.get_mut(&wire.from_node)
                        && let Some(from_pin) = from_node.outputs.get_mut(&wire.from_pin)
                    {
                        from_pin.remove_connection(*id)?;
                    }
                    if let Some(to_node) = self.nodes.get_mut(&wire.to_node)
                        && let Some(to_pin) = to_node.inputs.get_mut(&wire.to_pin)
                    {
                        to_pin.remove_connection(*id)?;
                    }
                }
            }

            Command::MoveNodes { moves } => {
                for (id, _old_pos, new_pos) in moves {
                    if let Some(node) = self.nodes.get_mut(id) {
                        node.set_pos(*new_pos)?;
                        self.redraw_node_wires(id)?;
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
                        node.update_input_value(r#type, new_value)?;
                    }
                }
            }

            Command::ResizeNodeInput {
                node_id,
                r#type,
                new_size,
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
                        iv.size = Some(*new_size);
                    }

                    if historical {
                        node.update_input_size(r#type, *new_size)?;
                    }

                    self.redraw_node_wires(node_id)?;
                }
            }
        }

        Ok(())
    }

    pub fn redraw_node_wires(&mut self, node_id: &PenguinNodeID) -> Result<(), JsValue> {
        let Some(node) = self.nodes.get(node_id) else {
            return Ok(());
        };
        let pos = node.pos();

        let mut wire_ids = Vec::new();
        for pin in node.outputs.values() {
            wire_ids.extend(pin.connections().iter().copied());
        }
        for pin in node.inputs.values() {
            wire_ids.extend(pin.connections().iter().copied());
        }

        for wire_id in wire_ids {
            if let Some(wire) = self.wires.get_mut(&wire_id) {
                if wire.inner().from_node == *node_id {
                    wire.redraw_from(pos)?;
                }
                if wire.inner().to_node == *node_id {
                    wire.redraw_to(pos)?;
                }
            }
        }

        Ok(())
    }

    fn try_squash_command(&mut self, cmd: &Command) -> Result<bool, JsValue> {
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
            return Ok(false);
        }

        self.apply_command(cmd, false)?;

        if let Some(last_tx) = self.past.last_mut() {
            squash_commands(&mut last_tx.commands[0], cmd);
        }

        Ok(true)
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
        }
    }
}
