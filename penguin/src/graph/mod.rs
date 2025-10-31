pub mod node;
pub mod pin;
pub mod wire;

use igloo_interface::{
    PenguinPinID, PenguinPinType, PenguinRegistry,
    graph::{PenguinGraph, PenguinNode, PenguinNodeID, PenguinWire, PenguinWireID},
};
pub use node::*;
pub use pin::*;
use wasm_bindgen::JsValue;
use web_sys::{Element, HtmlElement, MouseEvent};
pub use wire::*;

use std::collections::{HashMap, HashSet};

use crate::{
    ffi,
    interaction::WiringState,
    viewport::{ClientBox, ClientToWorld, WorldPoint},
};

#[derive(Debug)]
pub struct WebGraph {
    nodes: HashMap<PenguinNodeID, WebNode>,
    wires: HashMap<PenguinWireID, WebWire>,
    nodes_el: Element,
    wires_el: Element,
    pub temp_wire: WebTempWire,
    pub selection: Selection,
}

#[derive(Debug, Default)]
pub struct Selection {
    pub nodes: HashSet<PenguinNodeID>,
    pub wires: HashSet<PenguinWireID>,
}

impl WebGraph {
    pub fn new(viewport_el: &Element) -> Result<Self, JsValue> {
        let document = ffi::document();

        let wires_el = document.create_element("div")?;
        wires_el.set_id("penguin-wires");
        viewport_el.append_child(&wires_el)?;

        let nodes_el = document.create_element("div")?;
        nodes_el.set_id("penguin-nodes");
        viewport_el.append_child(&nodes_el)?;

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

    pub fn penguin_graph(&self) -> PenguinGraph {
        todo!()
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

    pub fn delete_pin_wires(
        &mut self,
        node_id: &PenguinNodeID,
        pin_id: &PenguinPinID,
        is_out: bool,
    ) {
        if let Some(node) = self.nodes.get(node_id) {
            let pin = if is_out {
                node.outputs.get(pin_id)
            } else {
                node.inputs.get(pin_id)
            };

            let Some(pin) = pin else {
                return;
            };

            for wire_id in pin.connections().to_vec() {
                self.delete_wire(wire_id);
            }
        }
    }

    pub fn add_wire(
        &mut self,
        from_node_id: PenguinNodeID,
        from_pin_id: PenguinPinID,
        from_type: PenguinPinType,
        to_node_id: PenguinNodeID,
        to_pin_id: PenguinPinID,
        to_type: PenguinPinType,
    ) -> Result<(), JsValue> {
        // remove existing wires
        let to_node = self.nodes.get_mut(&to_node_id).unwrap();
        let to_pin = to_node.inputs.get_mut(&to_pin_id).unwrap();
        let connections = to_pin.take_connections()?;

        for wire_id in connections {
            self.delete_wire(wire_id)?;
        }

        // normal connections
        if from_type == to_type {
            let wire_id = PenguinWireID(self.wires.keys().map(|id| id.0).max().unwrap_or(0) + 1);

            let to_node = self.nodes.get_mut(&to_node_id).unwrap();
            let to_node_pos = to_node.pos();
            let to_pin = to_node.inputs.get_mut(&to_pin_id).unwrap();
            let to_hitbox = to_pin.hitbox.clone();
            to_pin.add_connection(wire_id)?;

            let from_node = self.nodes.get_mut(&from_node_id).unwrap();
            let from_node_pos = from_node.pos();
            let from_pin = from_node.outputs.get_mut(&from_pin_id).unwrap();
            let from_hitbox = from_pin.hitbox.clone();
            from_pin.add_connection(wire_id)?;

            let inner = PenguinWire {
                from_node: from_node_id,
                from_pin: from_pin_id,
                to_node: to_node_id,
                to_pin: to_pin_id,
                r#type: from_type,
            };

            let mut wire = WebWire::new(&self.wires_el, wire_id, inner, from_hitbox, to_hitbox)?;

            self.wires.insert(wire_id, wire);
        }
        // cast connections
        else {
            // TODO
        }

        self.redraw_node_wires(&from_node_id)?;
        self.redraw_node_wires(&to_node_id)?;

        Ok(())
    }

    pub fn show_wiring(&mut self, ws: &WiringState) {
        for node in self.nodes.values_mut() {
            for pin in node.inputs.values_mut() {
                pin.show_wiring(ws);
            }
        }
    }

    pub fn hide_wiring(&mut self) {
        for node in self.nodes.values_mut() {
            for pin in node.inputs.values_mut() {
                pin.hide_wiring();
            }
        }
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

    pub fn redraw_node_wires(&mut self, node_id: &PenguinNodeID) -> Result<(), JsValue> {
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

    pub fn select_pin_wires(
        &mut self,
        node_id: &PenguinNodeID,
        pin_id: &PenguinPinID,
        is_out: bool,
    ) {
        if let Some(node) = self.nodes.get(node_id) {
            let pin = if is_out {
                node.outputs.get(pin_id)
            } else {
                node.inputs.get(pin_id)
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
}
