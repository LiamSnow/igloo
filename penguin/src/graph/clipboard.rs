use super::cmds::*;
use crate::{graph::WebGraph, viewport::WorldPoint};
use igloo_interface::graph::{PenguinGraph, PenguinNodeID, PenguinWireID};
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use web_sys::ClipboardEvent;

impl WebGraph {
    pub fn handle_copy(&self, e: &ClipboardEvent) -> Result<(), JsValue> {
        if self.selection.nodes.is_empty() {
            return Ok(());
        }

        let mut graph = PenguinGraph {
            nodes: HashMap::with_capacity(self.selection.nodes.len()),
            wires: HashMap::with_capacity(self.selection.wires.len()),
        };

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut count = 0;

        for id in &self.selection.nodes {
            if let Some(node) = self.nodes.get(id) {
                let inner = node.inner();
                sum_x += inner.x;
                sum_y += inner.y;
                count += 1;
            }
        }

        let center = WorldPoint::new(sum_x / count as f64, sum_y / count as f64);

        for id in &self.selection.nodes {
            if let Some(node) = self.nodes.get(id) {
                let mut node = node.inner().clone();
                node.x -= center.x;
                node.y -= center.y;
                graph.nodes.insert(*id, node);
            }
        }

        for id in &self.selection.wires {
            if let Some(wire) = self.wires.get(id) {
                let inner = wire.inner();
                if self.selection.nodes.contains(&inner.from_node)
                    && self.selection.nodes.contains(&inner.to_node)
                {
                    graph.wires.insert(*id, inner.clone());
                }
            }
        }

        let json = serde_json::to_string(&graph)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;

        if let Some(clipboard) = e.clipboard_data() {
            clipboard.set_data("application/x-penguin", &json)?;
            clipboard.set_data("text/plain", &json)?;
            e.prevent_default();
        }

        Ok(())
    }

    pub fn handle_paste(
        &mut self,
        e: &ClipboardEvent,
        mouse_pos: WorldPoint,
    ) -> Result<(), JsValue> {
        let Some(clipboard) = e.clipboard_data() else {
            return Ok(());
        };

        let Ok(json) = clipboard.get_data("application/x-penguin") else {
            return Ok(());
        };

        let graph: PenguinGraph = serde_json::from_str(&json)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;

        let next_node_id = self.nodes.keys().map(|id| id.0).max().unwrap_or(0) + 1;
        let next_wire_id = self.wires.keys().map(|id| id.0).max().unwrap_or(0) + 1;

        let mut node_id_map = HashMap::new();
        let mut wire_id_map = HashMap::new();

        for (i, old_id) in graph.nodes.keys().enumerate() {
            node_id_map.insert(*old_id, PenguinNodeID(next_node_id + i as u16));
        }

        for (i, old_id) in graph.wires.keys().enumerate() {
            wire_id_map.insert(*old_id, PenguinWireID(next_wire_id + i as u16));
        }

        let mut tx = Transaction::with_capacity(graph.nodes.len() + graph.wires.len());

        for (old_id, mut node) in graph.nodes {
            let new_id = *node_id_map.get(&old_id).unwrap();
            node.x += mouse_pos.x;
            node.y += mouse_pos.y;

            tx.push(Command::AddNode { id: new_id, node });
        }

        for (old_id, mut wire) in graph.wires {
            let new_id = *wire_id_map.get(&old_id).unwrap();
            wire.from_node = *node_id_map.get(&wire.from_node).unwrap();
            wire.to_node = *node_id_map.get(&wire.to_node).unwrap();

            tx.push(Command::AddWire { id: new_id, wire });
        }

        e.prevent_default();
        self.execute(tx)
    }

    pub fn handle_cut(&mut self, e: &ClipboardEvent) -> Result<(), JsValue> {
        self.handle_copy(e)?;
        self.delete_selection()
    }
}
