pub mod input;
pub mod node;
pub mod pin;
pub mod wire;

use igloo_interface::{
    PenguinPinRef, PenguinRegistry,
    graph::{PenguinGraph, PenguinNode, PenguinNodeID, PenguinWire, PenguinWireID},
};
pub use node::*;
pub use pin::*;
use wasm_bindgen::JsValue;
use web_sys::{ClipboardEvent, Element};
pub use wire::*;

use std::collections::{HashMap, HashSet};

use crate::{
    app::event::document,
    graph::input::WebInputType,
    viewport::{ClientBox, ClientToWorld, WorldPoint},
};

#[derive(Debug)]
pub struct WebGraph {
    nodes: HashMap<PenguinNodeID, WebNode>,
    wires: HashMap<PenguinWireID, WebWire>,
    nodes_el: Element,
    wires_el: Element,
    temp_wire: WebTempWire,
    selection: Selection,
}

#[derive(Debug, Default)]
pub struct Selection {
    nodes: HashSet<PenguinNodeID>,
    wires: HashSet<PenguinWireID>,
}

impl WebGraph {
    pub fn new(parent: &Element) -> Result<Self, JsValue> {
        let document = document();

        let wires_el = document.create_element("div")?;
        wires_el.set_id("penguin-wires");
        parent.append_child(&wires_el)?;

        let nodes_el = document.create_element("div")?;
        nodes_el.set_id("penguin-nodes");
        parent.append_child(&nodes_el)?;

        Ok(Self {
            nodes: HashMap::with_capacity(100),
            wires: HashMap::with_capacity(100),
            temp_wire: WebTempWire::new(&wires_el)?,
            nodes_el,
            wires_el,
            selection: Selection::default(),
        })
    }

    pub fn load(&mut self, registry: &PenguinRegistry, graph: PenguinGraph) -> Result<(), JsValue> {
        self.wires = HashMap::with_capacity(graph.wires.len());
        self.wires_el.set_inner_html("");

        self.temp_wire = WebTempWire::new(&self.wires_el)?;

        self.nodes = HashMap::with_capacity(graph.nodes.len());
        self.nodes_el.set_inner_html("");

        for (id, node) in graph.nodes {
            // TODO probably should load partial state on error? And try auto fix idk

            self.nodes.insert(
                id,
                WebNode::new(&self.nodes_el, registry, Some(&graph.wires), node, id)?,
            );
        }

        for (id, wire) in graph.wires {
            let (from_pin_hitbox, from_node_pos) = {
                let Some(from_node) = self.nodes.get_mut(&wire.from_node) else {
                    log::error!("Dangling wire. Missing from_node. id={id:?}, wire={wire:?}");
                    continue;
                };

                let Some(from_pin) = from_node.outputs.get_mut(&wire.from_pin) else {
                    log::error!("Dangling wire. Missing from_pin. id={id:?}, wire={wire:?}");
                    continue;
                };

                (from_pin.hitbox.clone(), from_node.pos())
            };

            let (to_pin_hitbox, to_node_pos) = {
                let Some(to_node) = self.nodes.get_mut(&wire.to_node) else {
                    log::error!("Dangling wire. Missing to_node. id={id:?}, wire={wire:?}");
                    continue;
                };

                let Some(to_pin) = to_node.inputs.get_mut(&wire.to_pin) else {
                    log::error!("Dangling wire. Missing to_pin. id={id:?}, wire={wire:?}.");
                    continue;
                };

                (to_pin.hitbox.clone(), to_node.pos())
            };

            // TODO assert types match (from type, wire type, to type)

            let mut wire = WebWire::new(&self.wires_el, id, wire, from_pin_hitbox, to_pin_hitbox)?;

            wire.redraw_from(from_node_pos)?;
            wire.redraw_to(to_node_pos)?;

            self.wires.insert(id, wire);
        }

        Ok(())
    }

    pub fn penguin(&self) -> PenguinGraph {
        let mut res = PenguinGraph {
            nodes: HashMap::with_capacity(self.nodes.len()),
            wires: HashMap::with_capacity(self.wires.len()),
        };

        for (id, node) in &self.nodes {
            res.nodes.insert(*id, node.inner().clone());
        }

        for (id, wire) in &self.wires {
            res.wires.insert(*id, wire.inner().clone());
        }

        res
    }

    pub fn place_node(
        &mut self,
        registry: &PenguinRegistry,
        inner: PenguinNode,
    ) -> Result<PenguinNodeID, JsValue> {
        let node_id = PenguinNodeID(self.nodes.keys().map(|id| id.0).max().unwrap_or(0) + 1);

        self.nodes.insert(
            node_id,
            WebNode::new(&self.nodes_el, registry, None, inner, node_id)?,
        );

        Ok(node_id)
    }

    pub fn delete_wire(&mut self, wire_id: PenguinWireID) -> Result<(), JsValue> {
        let Some(wire) = self.wires.remove(&wire_id) else {
            return Ok(());
        };

        if let Some(from_node) = self.nodes.get_mut(&wire.inner.from_node)
            && let Some(from_pin) = from_node.outputs.get_mut(&wire.inner.from_pin)
        {
            from_pin.remove_connection(wire_id)?;
        }

        if let Some(to_node) = self.nodes.get_mut(&wire.inner.to_node)
            && let Some(to_pin) = to_node.inputs.get_mut(&wire.inner.to_pin)
        {
            to_pin.remove_connection(wire_id)?;
        }

        self.redraw_node_wires(&wire.inner.from_node)?;
        self.redraw_node_wires(&wire.inner.to_node)?;

        Ok(())
    }

    pub fn delete_pin_wires(&mut self, pin: &PenguinPinRef) {
        if let Some(node) = self.nodes.get(&pin.node_id) {
            let pin = if pin.is_output {
                node.outputs.get(&pin.id)
            } else {
                node.inputs.get(&pin.id)
            };

            let Some(pin) = pin else {
                return;
            };

            for wire_id in pin.connections().to_vec() {
                self.delete_wire(wire_id);
            }
        }
    }

    pub fn add_wire(&mut self, pin_a: PenguinPinRef, pin_b: PenguinPinRef) -> Result<(), JsValue> {
        let (from, to) = if pin_a.is_output {
            (pin_a, pin_b)
        } else {
            (pin_b, pin_a)
        };

        // remove existing wires
        let to_node = self.nodes.get_mut(&to.node_id).unwrap();
        let to_pin = to_node.inputs.get_mut(&to.id).unwrap();
        let connections = to_pin.take_connections()?;

        for wire_id in connections {
            self.delete_wire(wire_id)?;
        }

        // normal connections
        if from.r#type == to.r#type {
            let wire_id = PenguinWireID(self.wires.keys().map(|id| id.0).max().unwrap_or(0) + 1);

            let to_node = self.nodes.get_mut(&to.node_id).unwrap();
            let to_pin = to_node.inputs.get_mut(&to.id).unwrap();
            let to_hitbox = to_pin.hitbox.clone();
            to_pin.add_connection(wire_id)?;

            let from_node = self.nodes.get_mut(&from.node_id).unwrap();
            let from_pin = from_node.outputs.get_mut(&from.id).unwrap();
            let from_hitbox = from_pin.hitbox.clone();
            from_pin.add_connection(wire_id)?;

            let inner = PenguinWire {
                from_node: from.node_id,
                from_pin: from.id,
                to_node: to.node_id,
                to_pin: to.id,
                r#type: from.r#type,
            };

            let wire = WebWire::new(&self.wires_el, wire_id, inner, from_hitbox, to_hitbox)?;

            self.wires.insert(wire_id, wire);
        }
        // cast connections
        else {
            // TODO
        }

        self.redraw_node_wires(&from.node_id)?;
        self.redraw_node_wires(&to.node_id)?;

        Ok(())
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

    pub fn get_node_pos(&self, node_id: &PenguinNodeID) -> Result<WorldPoint, JsValue> {
        let Some(node) = self.nodes.get(node_id) else {
            return Err(JsValue::from_str("Unknown Node"));
        };

        Ok(node.pos())
    }

    fn redraw_node_wires(&mut self, node_id: &PenguinNodeID) -> Result<(), JsValue> {
        let Some(node) = self.nodes.get_mut(node_id) else {
            return Err(JsValue::from_str("Unknown Node"));
        };

        let pos = node.pos();

        for pin in node.outputs.values() {
            let wire_ids = pin.connections();
            for wire_id in wire_ids {
                let Some(wire) = self.wires.get_mut(wire_id) else {
                    // TODO log error
                    continue;
                };

                wire.redraw_from(pos)?;
            }
        }

        for pin in node.inputs.values() {
            let wire_ids = pin.connections();
            for wire_id in wire_ids {
                let Some(wire) = self.wires.get_mut(wire_id) else {
                    // TODO log error
                    continue;
                };

                wire.redraw_to(pos)?;
            }
        }

        Ok(())
    }

    fn node_mut(&mut self, node_id: &PenguinNodeID) -> Result<&mut WebNode, JsValue> {
        let Some(node) = self.nodes.get_mut(node_id) else {
            return Err(JsValue::from_str("Unknown Node"));
        };
        Ok(node)
    }

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

        if let Some(wire) = self.wires.get(&wire_id) {
            wire.select(true);
        }
    }

    pub fn select_pin_wires(&mut self, pin: &PenguinPinRef) {
        if let Some(node) = self.nodes.get(&pin.node_id) {
            let pin = if pin.is_output {
                node.outputs.get(&pin.id)
            } else {
                node.inputs.get(&pin.id)
            };

            let Some(pin) = pin else {
                return;
            };

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

    pub fn delete_selection(&mut self) {
        let mut wires = self.selection.wires.clone();

        for node_id in &self.selection.nodes {
            if let Some(node) = self.nodes.remove(node_id) {
                for pin in node.inputs.values() {
                    let conns = pin.connections();
                    for conn in conns {
                        wires.insert(*conn);
                    }
                }

                for pin in node.outputs.values() {
                    let conns = pin.connections();
                    for conn in conns {
                        wires.insert(*conn);
                    }
                }
            }
        }

        for wire in wires {
            self.delete_wire(wire);
        }

        self.selection.nodes.clear();
        self.selection.wires.clear();
    }

    pub fn box_select(&mut self, cbox: ClientBox, ctw: ClientToWorld, append: bool) {
        if !append {
            self.clear_selection();
        }

        let m: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, node)| cbox.intersects(&node.client_box()))
            .map(|(id, _)| *id)
            .collect();

        for id in m {
            self.select_node(id, true);
        }

        let m: Vec<_> = self
            .wires
            .iter()
            .filter(|(_, wire)| wire.intersects(&cbox, &ctw))
            .map(|(id, _)| *id)
            .collect();

        for id in m {
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

    pub fn handle_input_change(
        &mut self,
        node_id: PenguinNodeID,
        r#type: input::WebInputType,
        new_value: String,
    ) -> Result<(), JsValue> {
        let node = self.node_mut(&node_id)?;

        let iv = match r#type {
            WebInputType::Pin(pin_id) => node.inner.input_pin_values.get_mut(&pin_id),
            WebInputType::NodeFeature(feature_id) => {
                node.inner.input_feature_values.get_mut(&feature_id)
            }
        };

        if let Some(iv) = iv {
            iv.set_from_string(new_value);
        }

        Ok(())
    }

    pub fn handle_input_resize(
        &mut self,
        node_id: PenguinNodeID,
        r#type: input::WebInputType,
        size: (i32, i32),
    ) -> Result<(), JsValue> {
        let node = self.node_mut(&node_id)?;

        let iv = match r#type {
            WebInputType::Pin(pin_id) => node.inner.input_pin_values.get_mut(&pin_id),
            WebInputType::NodeFeature(feature_id) => {
                node.inner.input_feature_values.get_mut(&feature_id)
            }
        };

        if let Some(iv) = iv {
            iv.size = Some(size);
        }

        self.redraw_node_wires(&node_id)?;

        Ok(())
    }

    pub fn handle_copy(&self, e: &ClipboardEvent) -> Result<(), JsValue> {
        log::info!("COPY");

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
        registry: &PenguinRegistry,
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

        // give new IDs
        for (i, old_id) in graph.nodes.keys().enumerate() {
            node_id_map.insert(*old_id, PenguinNodeID(next_node_id + i as u16));
        }

        for (i, old_id) in graph.wires.keys().enumerate() {
            wire_id_map.insert(*old_id, PenguinWireID(next_wire_id + i as u16));
        }

        // update wires with new IDs
        let mut transformed_wires = HashMap::new();
        for (old_id, mut wire) in graph.wires {
            let new_id = *wire_id_map.get(&old_id).unwrap();
            wire.from_node = *node_id_map.get(&wire.from_node).unwrap();
            wire.to_node = *node_id_map.get(&wire.to_node).unwrap();
            transformed_wires.insert(new_id, wire);
        }

        for (old_id, mut node) in graph.nodes {
            let new_id = *node_id_map.get(&old_id).unwrap();
            node.x += mouse_pos.x;
            node.y += mouse_pos.y;
            self.nodes.insert(
                new_id,
                WebNode::new(
                    &self.nodes_el,
                    registry,
                    Some(&transformed_wires),
                    node,
                    new_id,
                )?,
            );
        }

        for (new_id, wire) in transformed_wires {
            let (from_pin_hitbox, from_node_pos) = {
                let Some(from_node) = self.nodes.get_mut(&wire.from_node) else {
                    log::error!("Missing from_node during paste. wire={wire:?}");
                    continue;
                };
                let Some(from_pin) = from_node.outputs.get_mut(&wire.from_pin) else {
                    log::error!("Missing from_pin during paste. wire={wire:?}");
                    continue;
                };
                (from_pin.hitbox.clone(), from_node.pos())
            };

            let (to_pin_hitbox, to_node_pos) = {
                let Some(to_node) = self.nodes.get_mut(&wire.to_node) else {
                    log::error!("Missing to_node during paste. wire={wire:?}");
                    continue;
                };
                let Some(to_pin) = to_node.inputs.get_mut(&wire.to_pin) else {
                    log::error!("Missing to_pin during paste. wire={wire:?}");
                    continue;
                };
                (to_pin.hitbox.clone(), to_node.pos())
            };

            let mut web_wire =
                WebWire::new(&self.wires_el, new_id, wire, from_pin_hitbox, to_pin_hitbox)?;
            web_wire.redraw_from(from_node_pos)?;
            web_wire.redraw_to(to_node_pos)?;

            self.wires.insert(new_id, web_wire);
        }

        e.prevent_default();
        Ok(())
    }

    pub fn handle_cut(&mut self, e: &ClipboardEvent) -> Result<(), JsValue> {
        self.handle_copy(e)?;
        self.delete_selection();
        Ok(())
    }
}
