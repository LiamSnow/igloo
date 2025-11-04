use super::cmds::*;
use crate::{
    graph::{WebGraph, input::WebInputType},
    viewport::WorldPoint,
};
use igloo_interface::{
    PenguinPinRef,
    graph::{PenguinNode, PenguinNodeID, PenguinWireID},
};
use wasm_bindgen::JsValue;

impl WebGraph {
    pub fn place_node(&mut self, inner: PenguinNode) -> Result<PenguinNodeID, JsValue> {
        let node_id = PenguinNodeID(self.nodes.keys().map(|id| id.0).max().unwrap_or(0) + 1);

        let tx = Transaction::single(Command::AddNode {
            id: node_id,
            node: inner,
        });

        self.execute(tx)?;
        Ok(node_id)
    }

    pub fn delete_wire(&mut self, wire_id: PenguinWireID) -> Result<(), JsValue> {
        let Some(wire) = self.wires.get(&wire_id) else {
            return Ok(());
        };

        let tx = Transaction::single(Command::DeleteWire {
            id: wire_id,
            wire: wire.inner().clone(),
        });

        self.execute(tx)
    }

    pub fn delete_pin_wires(&mut self, pin: &PenguinPinRef) -> Result<(), JsValue> {
        let Some(node) = self.nodes.get(&pin.node_id) else {
            return Ok(());
        };

        let pin_obj = if pin.is_output {
            node.outputs.get(&pin.id)
        } else {
            node.inputs.get(&pin.id)
        };

        let Some(pin_obj) = pin_obj else {
            return Ok(());
        };

        let wire_ids: Vec<_> = pin_obj.connections().to_vec();
        let mut tx = Transaction::with_capacity(wire_ids.len());

        for wire_id in wire_ids {
            if let Some(wire) = self.wires.get(&wire_id) {
                tx.push(Command::DeleteWire {
                    id: wire_id,
                    wire: wire.inner().clone(),
                });
            }
        }

        self.execute(tx)
    }

    pub fn add_wire(&mut self, pin_a: PenguinPinRef, pin_b: PenguinPinRef) -> Result<(), JsValue> {
        let (from, to) = if pin_a.is_output {
            (pin_a, pin_b)
        } else {
            (pin_b, pin_a)
        };

        let mut tx = Transaction::with_capacity(2);

        if let Some(to_node) = self.nodes.get(&to.node_id)
            && let Some(to_pin) = to_node.inputs.get(&to.id)
        {
            for wire_id in to_pin.connections() {
                if let Some(wire) = self.wires.get(wire_id) {
                    tx.push(Command::DeleteWire {
                        id: *wire_id,
                        wire: wire.inner().clone(),
                    });
                }
            }
        }

        if from.r#type == to.r#type {
            let wire_id = PenguinWireID(self.wires.keys().map(|id| id.0).max().unwrap_or(0) + 1);
            let wire = igloo_interface::graph::PenguinWire {
                from_node: from.node_id,
                from_pin: from.id,
                to_node: to.node_id,
                to_pin: to.id,
                r#type: from.r#type,
            };

            tx.push(Command::AddWire { id: wire_id, wire });
        }

        self.execute(tx)
    }

    pub fn start_wiring(&mut self, start: &PenguinPinRef) -> Result<(), JsValue> {
        let Some(node) = self.nodes.get(&start.node_id) else {
            return Err(JsValue::from_str("Unknown Node"));
        };

        let Some(pin) = node.pin(start) else {
            return Err(JsValue::from_str("Unknown Pin"));
        };

        self.temp_wire
            .show(&pin.hitbox, node.pos(), start.r#type, start.is_output)?;

        for node in self.nodes.values_mut() {
            for pin in node.inputs.values_mut() {
                pin.show_wiring(start);
            }
        }

        Ok(())
    }

    pub fn update_wiring(&self, wpos: WorldPoint) -> Result<(), JsValue> {
        self.temp_wire.update(wpos)
    }

    pub fn stop_wiring(&mut self) -> Result<(), JsValue> {
        self.temp_wire.hide()?;

        for node in self.nodes.values_mut() {
            for pin in node.inputs.values_mut() {
                pin.hide_wiring();
            }
        }

        Ok(())
    }

    /// moves node, without appending to history
    /// WARN: make sure to call finish_move
    pub fn move_node(
        &mut self,
        node_id: &PenguinNodeID,
        new_pos: WorldPoint,
    ) -> Result<(), JsValue> {
        let Some(node) = self.nodes.get_mut(node_id) else {
            return Err(JsValue::from_str("Unknown Node"));
        };

        node.set_pos(new_pos)?;
        self.redraw_node_wires(node_id)?;

        Ok(())
    }

    pub fn finish_moves(
        &mut self,
        moves: Vec<(PenguinNodeID, WorldPoint, WorldPoint)>,
    ) -> Result<(), JsValue> {
        if moves.is_empty() {
            return Ok(());
        }

        let tx = Transaction::single(Command::MoveNodes { moves });
        self.execute(tx)
    }

    pub fn get_node_pos(&self, node_id: &PenguinNodeID) -> Result<WorldPoint, JsValue> {
        let Some(node) = self.nodes.get(node_id) else {
            return Err(JsValue::from_str("Unknown Node"));
        };

        Ok(node.pos())
    }

    pub fn handle_input_change(
        &mut self,
        node_id: PenguinNodeID,
        r#type: WebInputType,
        new_value: String,
    ) -> Result<(), JsValue> {
        let old_value = if let Some(node) = self.nodes.get(&node_id) {
            match &r#type {
                WebInputType::Pin(pin_id) => node
                    .inner
                    .input_pin_values
                    .get(pin_id)
                    .map(|v| v.value.to_string()),
                WebInputType::NodeFeature(feature_id) => node
                    .inner
                    .input_feature_values
                    .get(feature_id)
                    .map(|v| v.value.to_string()),
            }
        } else {
            None
        };

        let Some(old_value) = old_value else {
            return Ok(());
        };

        let tx = Transaction::single(Command::ChangeNodeInput {
            node_id,
            r#type,
            old_value,
            new_value,
        });

        self.execute(tx)
    }

    pub fn handle_input_resize(
        &mut self,
        node_id: PenguinNodeID,
        r#type: WebInputType,
        new_size: (i32, i32),
    ) -> Result<(), JsValue> {
        let old_size = if let Some(node) = self.nodes.get(&node_id) {
            match &r#type {
                WebInputType::Pin(pin_id) => {
                    node.inner.input_pin_values.get(pin_id).and_then(|v| v.size)
                }
                WebInputType::NodeFeature(feature_id) => node
                    .inner
                    .input_feature_values
                    .get(feature_id)
                    .and_then(|v| v.size),
            }
        } else {
            None
        };

        let Some(old_size) = old_size else {
            return Ok(());
        };

        if old_size == new_size {
            return Ok(());
        }

        let tx = Transaction::single(Command::ResizeNodeInput {
            node_id,
            r#type,
            old_size,
            new_size,
        });

        self.execute(tx)
    }
}
