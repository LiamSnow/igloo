use super::cmds::*;
use crate::{
    graph::{WebGraph, input::WebInputType},
    viewport::{ClientToWorld, WorldPoint},
};
use igloo_interface::{
    PenguinNodeDefnRef, PenguinPinID, PenguinPinRef,
    graph::{PenguinNode, PenguinNodeID, PenguinWire, PenguinWireID},
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

        let mut tx = Transaction::with_capacity(4);

        // remove existing wires
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

        // direct connect
        if from.r#type == to.r#type {
            let wire_id = PenguinWireID(self.wires.keys().map(|id| id.0).max().unwrap_or(0) + 1);
            let wire = PenguinWire {
                from_node: from.node_id,
                from_pin: from.id,
                to_node: to.node_id,
                to_pin: to.id,
                r#type: from.r#type,
            };

            tx.push(Command::AddWire { id: wire_id, wire });
        }
        // cast connect
        else if let Some(cast_node_name) = from.cast_name(to.r#type) {
            let defn_ref = PenguinNodeDefnRef::new("std", &cast_node_name, 1);

            // TODO place between pins NOT nodes
            let from_node_pos = self
                .nodes
                .get(&from.node_id)
                .ok_or(JsValue::from_str("From node not found"))?
                .pos();
            let to_node_pos = self
                .nodes
                .get(&to.node_id)
                .ok_or(JsValue::from_str("To node not found"))?
                .pos();

            let cast_x = (from_node_pos.x + to_node_pos.x) / 2.0;
            let cast_y = (from_node_pos.y + to_node_pos.y) / 2.0;

            let cast_node_id =
                PenguinNodeID(self.nodes.keys().map(|id| id.0).max().unwrap_or(0) + 1);
            let wire_id_1 = PenguinWireID(self.wires.keys().map(|id| id.0).max().unwrap_or(0) + 1);
            let wire_id_2 = PenguinWireID(wire_id_1.0 + 1);

            tx.push(Command::AddNode {
                id: cast_node_id,
                node: PenguinNode {
                    defn_ref,
                    x: cast_x,
                    y: cast_y,
                    ..Default::default()
                },
            });

            tx.push(Command::AddWire {
                id: wire_id_1,
                wire: PenguinWire {
                    from_node: from.node_id,
                    from_pin: from.id,
                    to_node: cast_node_id,
                    to_pin: PenguinPinID::from_str("Input"),
                    r#type: from.r#type,
                },
            });

            tx.push(Command::AddWire {
                id: wire_id_2,
                wire: PenguinWire {
                    from_node: cast_node_id,
                    from_pin: PenguinPinID::from_str("Output"),
                    to_node: to.node_id,
                    to_pin: to.id,
                    r#type: to.r#type,
                },
            });
        } else {
            return Ok(());
        }

        self.execute(tx)
    }
    pub fn start_wiring(
        &mut self,
        start: &PenguinPinRef,
        ctw: &ClientToWorld,
    ) -> Result<(), JsValue> {
        let Some(node) = self.nodes.get(&start.node_id) else {
            return Err(JsValue::from_str("Unknown Node"));
        };

        let Some(pin) = node.pin(start) else {
            return Err(JsValue::from_str("Unknown Pin"));
        };

        self.temp_wire
            .show(&pin.hitbox, start.r#type, start.is_output, ctw)?;

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

    pub fn swap_node_variant(
        &mut self,
        node_id: PenguinNodeID,
        new_node_name: String,
    ) -> Result<(), JsValue> {
        let Some(cur_web_node) = self.nodes.get(&node_id) else {
            return Err(JsValue::from_str("Node not found"));
        };

        let mut new_node_inner = cur_web_node.inner().clone();
        new_node_inner.defn_ref.node_name = new_node_name;

        let new_defn = self
            .registry
            .get_defn(&new_node_inner.defn_ref)
            .ok_or(JsValue::from_str("New node definition not found"))?;

        // collect wires
        let mut wires_to_remove = Vec::new();
        let mut wires_to_readd = Vec::new();
        for (pin_id, pin) in &cur_web_node.inputs {
            for wire_id in pin.connections() {
                if let Some(wire) = self.wires.get(wire_id) {
                    let wire_inner = wire.inner().clone();
                    wires_to_remove.push((*wire_id, wire_inner.clone()));

                    // only readd if still exists
                    if new_defn.inputs.contains_key(pin_id) {
                        wires_to_readd.push(wire_inner);
                    }
                }
            }
        }
        for (pin_id, pin) in &cur_web_node.outputs {
            for wire_id in pin.connections() {
                if let Some(wire) = self.wires.get(wire_id) {
                    let wire_inner = wire.inner().clone();
                    wires_to_remove.push((*wire_id, wire_inner.clone()));

                    // only readd if still exists
                    if new_defn.outputs.contains_key(pin_id) {
                        wires_to_readd.push(wire_inner);
                    }
                }
            }
        }

        let mut tx = Transaction::with_capacity(wires_to_remove.len() + 2 + wires_to_readd.len());

        for (wire_id, wire) in wires_to_remove {
            tx.push(Command::DeleteWire { id: wire_id, wire });
        }

        tx.push(Command::DeleteNode {
            id: node_id,
            node: cur_web_node.inner().clone(),
        });

        tx.push(Command::AddNode {
            id: node_id,
            node: new_node_inner,
        });

        // readd wires
        let next_wire_id_base = self.wires.keys().map(|id| id.0).max().unwrap_or(0) + 1;
        for (i, wire) in wires_to_readd.into_iter().enumerate() {
            let new_wire_id = PenguinWireID(next_wire_id_base + i as u16);
            tx.push(Command::AddWire {
                id: new_wire_id,
                wire,
            });
        }

        self.execute(tx)
    }

    pub fn split_wire_with_reroute(
        &mut self,
        wire_id: PenguinWireID,
        wpos: WorldPoint,
    ) -> Result<(), JsValue> {
        let Some(wire) = self.wires.get(&wire_id) else {
            return Ok(());
        };
        let original_wire = wire.inner().clone();

        let node_name = format!("Reroute {}", original_wire.r#type);
        let defn_Ref = PenguinNodeDefnRef::new("Standard Library", &node_name, 1);
        if self.registry.get_defn(&defn_Ref).is_none() {
            return Err(JsValue::from_str("Reroute node definition not found"));
        }

        let node_id = PenguinNodeID(self.nodes.keys().map(|id| id.0).max().unwrap_or(0) + 1);
        let wire_id_1 = PenguinWireID(self.wires.keys().map(|id| id.0).max().unwrap_or(0) + 1);
        let wire_id_2 = PenguinWireID(wire_id_1.0 + 1);

        let mut tx = Transaction::with_capacity(4);

        tx.push(Command::DeleteWire {
            id: wire_id,
            wire: original_wire.clone(),
        });

        tx.push(Command::AddNode {
            id: node_id,
            node: PenguinNode {
                defn_ref: defn_Ref,
                x: wpos.x,
                y: wpos.y,
                ..Default::default()
            },
        });

        tx.push(Command::AddWire {
            id: wire_id_1,
            wire: PenguinWire {
                from_node: original_wire.from_node,
                from_pin: original_wire.from_pin.clone(),
                to_node: node_id,
                to_pin: PenguinPinID::from_str("Input"),
                r#type: original_wire.r#type,
            },
        });

        tx.push(Command::AddWire {
            id: wire_id_2,
            wire: PenguinWire {
                from_node: node_id,
                from_pin: PenguinPinID::from_str("Output"),
                to_node: original_wire.to_node,
                to_pin: original_wire.to_pin,
                r#type: original_wire.r#type,
            },
        });

        self.execute(tx)
    }
}
