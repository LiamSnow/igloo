use super::cmds::*;
use crate::{
    graph::WebGraph,
    viewport::{ClientBox, ClientToWorld, WorldPoint},
};
use igloo_interface::penguin::{
    PenguinPinRef,
    graph::{PenguinNodeID, PenguinWireID},
};
use std::collections::HashSet;
use wasm_bindgen::JsValue;

#[derive(Debug, Default)]
pub struct Selection {
    pub nodes: HashSet<PenguinNodeID>,
    pub wires: HashSet<PenguinWireID>,
}

impl WebGraph {
    pub fn select_node(&mut self, node_id: PenguinNodeID, append: bool) {
        if !append {
            self.clear_selection();
        }

        self.selection.nodes.insert(node_id);

        if let Some(node) = self.nodes.get(&node_id) {
            node.select(true);
        }
    }

    pub fn select_wire(&mut self, wire_id: PenguinWireID, append: bool) {
        if !append {
            self.clear_selection();
        }

        self.selection.wires.insert(wire_id);

        if let Some(wire) = self.wires.get(&wire_id)
            && let Err(e) = wire.select(true)
        {
            log::error!("Failed to select {wire_id:?}: {e:?}");
        }
    }

    pub fn select_pin_wires(&mut self, pin: &PenguinPinRef) {
        if let Some(node) = self.nodes.get(&pin.node_id)
            && let Some(pin) = node.pin(pin)
        {
            for wire_id in pin.connections().to_vec() {
                self.select_wire(wire_id, true);
            }
        }
    }

    pub fn clear_selection(&mut self) {
        for node_id in &self.selection.nodes {
            if let Some(node) = self.nodes.get(node_id) {
                node.select(false);
            }
        }

        for wire_id in &self.selection.wires {
            if let Some(wire) = self.wires.get(wire_id) {
                wire.select(false);
            }
        }

        self.selection.nodes.clear();
        self.selection.wires.clear();
    }

    pub fn delete_selection(&mut self) -> Result<(), JsValue> {
        let mut tx =
            Transaction::with_capacity(self.selection.nodes.len() + self.selection.wires.len());

        let mut wires_to_delete = self.selection.wires.clone();

        for node_id in &self.selection.nodes {
            if let Some(node) = self.nodes.get(node_id) {
                for pin in node.inputs.values() {
                    for wire_id in pin.connections() {
                        wires_to_delete.insert(*wire_id);
                    }
                }
                for pin in node.outputs.values() {
                    for wire_id in pin.connections() {
                        wires_to_delete.insert(*wire_id);
                    }
                }
            }
        }

        for wire_id in &wires_to_delete {
            if let Some(wire) = self.wires.get(wire_id) {
                tx.push(Command::DeleteWire {
                    id: *wire_id,
                    wire: wire.inner().clone(),
                });
            }
        }

        for node_id in &self.selection.nodes {
            if let Some(node) = self.nodes.get(node_id) {
                tx.push(Command::DeleteNode {
                    id: *node_id,
                    node: node.inner().clone(),
                });
            }
        }

        self.selection.nodes.clear();
        self.selection.wires.clear();

        self.execute(tx)
    }

    pub fn box_select(&mut self, cbox: ClientBox, ctw: ClientToWorld, append: bool) {
        if !append {
            self.clear_selection();
        }

        let nodes: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, node)| cbox.intersects(&node.client_box()))
            .map(|(id, _)| *id)
            .collect();

        for id in nodes {
            self.select_node(id, true);
        }

        let wires: Vec<_> = self
            .wires
            .iter()
            .filter(|(_, wire)| wire.intersects(&cbox, &ctw))
            .map(|(id, _)| *id)
            .collect();

        for id in wires {
            self.select_wire(id, true);
        }
    }

    pub fn selection_poses(&self) -> Result<Vec<(PenguinNodeID, WorldPoint)>, JsValue> {
        let mut res = Vec::with_capacity(self.selection.nodes.len());
        for node_id in &self.selection.nodes {
            res.push((*node_id, self.get_node_pos(node_id)?));
        }
        Ok(res)
    }
}
