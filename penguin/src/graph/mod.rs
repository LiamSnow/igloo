use crate::{
    app::event::document,
    graph::{
        cmds::{Command, Transaction},
        node::WebNode,
        wire::{WebTempWire, WebWire},
    },
};
use igloo_interface::{
    PenguinRegistry,
    graph::{PenguinGraph, PenguinNodeID, PenguinWireID},
};
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use web_sys::Element;

mod cmds;
pub mod view;
pub use view::*;
pub mod clipboard;
pub mod ops;
pub mod select;
pub use select::*;

#[derive(Debug)]
pub struct WebGraph {
    pub registry: PenguinRegistry,
    pub(self) nodes: HashMap<PenguinNodeID, WebNode>,
    pub(self) wires: HashMap<PenguinWireID, WebWire>,
    pub(self) nodes_el: Element,
    pub(self) wires_el: Element,
    pub(self) temp_wire: WebTempWire,
    pub(self) selection: Selection,
    pub(self) past: Vec<Transaction>,
    pub(self) future: Vec<Transaction>,
}

impl WebGraph {
    pub fn new(registry: PenguinRegistry, parent: &Element) -> Result<Self, JsValue> {
        let document = document();

        let wires_el = document.create_element("div")?;
        wires_el.set_id("penguin-wires");
        parent.append_child(&wires_el)?;

        let nodes_el = document.create_element("div")?;
        nodes_el.set_id("penguin-nodes");
        parent.append_child(&nodes_el)?;

        Ok(Self {
            registry,
            nodes: HashMap::with_capacity(100),
            wires: HashMap::with_capacity(100),
            temp_wire: WebTempWire::new(&wires_el)?,
            nodes_el,
            wires_el,
            selection: Selection::default(),
            past: Vec::new(),
            future: Vec::new(),
        })
    }

    pub fn clear(&mut self) -> Result<(), JsValue> {
        self.selection.nodes.clear();
        self.selection.wires.clear();
        self.past.clear();
        self.future.clear();

        self.wires.clear();
        self.wires_el.set_inner_html("");
        self.temp_wire = WebTempWire::new(&self.wires_el)?;

        self.nodes.clear();
        self.nodes_el.set_inner_html("");

        Ok(())
    }

    pub fn load(&mut self, graph: PenguinGraph) -> Result<(), JsValue> {
        self.clear()?;

        for (id, node) in graph.nodes {
            self.apply_command(&Command::AddNode { id, node }, false)?;
        }

        for (id, wire) in graph.wires {
            if let Err(e) = self.apply_command(
                &Command::AddWire {
                    id,
                    wire: wire.clone(),
                },
                false,
            ) {
                log::error!("Failed to load wire. id={id:?}, wire={wire:?}, error={e:?}");
            }
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
}
