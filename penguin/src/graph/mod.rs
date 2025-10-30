pub mod node;
pub mod pin;
pub mod wire;

use igloo_interface::{
    PenguinPinID, PenguinPinType, PenguinRegistry,
    graph::{PenguinGraph, PenguinNodeID, PenguinWire, PenguinWireID},
};
pub use node::*;
pub use pin::*;
use wasm_bindgen::JsValue;
use web_sys::{Element, HtmlElement};
pub use wire::*;

use std::collections::HashMap;

use crate::{ffi, interaction::WiringState, viewport::WorldPoint};

#[derive(Debug)]
pub struct WebGraph {
    nodes: HashMap<PenguinNodeID, WebNode>,
    wires: HashMap<PenguinWireID, WebWire>,
    nodes_el: Element,
    wires_el: Element,
    pub temp_wire: WebTempWire,
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
        })
    }

    pub fn load(&mut self, registry: &PenguinRegistry, graph: PenguinGraph) -> Result<(), JsValue> {
        self.wires.clear();
        self.wires_el.set_inner_html("");

        self.temp_wire = WebTempWire::new(&self.wires_el)?;

        self.nodes.clear();
        self.nodes_el.set_inner_html("");

        for (id, node) in graph.nodes {
            // TODO probably should load partial state? Or auto fix idk
            self.nodes
                .insert(id, WebNode::new(&self.nodes_el, registry, node, id)?);
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

                from_pin.add_connection(id)?;

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

                to_pin.add_connection(id)?;

                (to_pin.hitbox.clone(), to_node.pos())
            };

            // TODO assert types match (from type, wire type, to type)

            let mut wire = WebWire::new(&self.wires_el, wire, from_pin_hitbox, to_pin_hitbox)?;

            wire.redraw_from(from_node_pos)?;
            wire.redraw_to(to_node_pos)?;

            self.wires.insert(id, wire);
        }

        Ok(())
    }

    pub fn penguin_graph(&self) -> PenguinGraph {
        todo!()
    }

    pub fn remove_wire(&mut self, wire_id: PenguinWireID) -> Result<(), JsValue> {
        let Some(wire) = self.wires.remove(&wire_id) else {
            return Ok(());
        };

        // FIXME unwraps
        let from_node = self.nodes.get_mut(&wire.inner.from_node).unwrap();
        let from_pin = from_node.outputs.get_mut(&wire.inner.from_pin).unwrap();
        from_pin.remove_connection(wire_id)?;

        let to_node = self.nodes.get_mut(&wire.inner.to_node).unwrap();
        let to_pin = to_node.inputs.get_mut(&wire.inner.to_pin).unwrap();
        to_pin.remove_connection(wire_id)?;

        Ok(())
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
            self.remove_wire(wire_id)?;
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

            let mut wire = WebWire::new(&self.wires_el, inner, from_hitbox, to_hitbox)?;

            wire.redraw_from(from_node_pos)?;
            wire.redraw_to(to_node_pos)?;

            self.wires.insert(wire_id, wire);
        }
        // cast connections
        else {
            // TODO
        }

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

        self.redraw_node_wires(node_id);

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
}
