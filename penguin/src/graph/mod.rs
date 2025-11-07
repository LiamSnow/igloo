use crate::{
    dom::{self, Div, node::DomNode},
    graph::{
        cmds::{Command, Transaction},
        node::WebNode,
        wire::{WebTempWire, WebWire},
    },
    viewport::ClientToWorld,
};
use igloo_interface::penguin::{
    PenguinRegistry,
    graph::{PenguinGraph, PenguinNodeID, PenguinWireID},
};
use std::collections::HashMap;

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
    pub(self) nodes_el: DomNode<Div>,
    pub(self) wires_el: DomNode<Div>,
    pub(self) temp_wire: WebTempWire,
    pub(self) selection: Selection,
    pub(self) past: Vec<Transaction>,
    pub(self) future: Vec<Transaction>,
    pub ctw: ClientToWorld,
}

impl WebGraph {
    pub fn new<T>(registry: PenguinRegistry, parent: &DomNode<T>) -> Self {
        let wires_el = dom::div().id("penguin-wires").mount(parent);
        let nodes_el = dom::div().id("penguin-nodes").mount(parent);

        Self {
            registry,
            nodes: HashMap::with_capacity(100),
            wires: HashMap::with_capacity(100),
            temp_wire: WebTempWire::new(&wires_el),
            nodes_el,
            wires_el,
            selection: Selection::default(),
            past: Vec::new(),
            future: Vec::new(),
            ctw: ClientToWorld::default(),
        }
    }

    pub fn clear(&mut self) {
        self.selection.nodes.clear();
        self.selection.wires.clear();
        self.past.clear();
        self.future.clear();

        self.wires.clear();
        self.wires_el.set_html("");
        self.temp_wire = WebTempWire::new(&self.wires_el);

        self.nodes.clear();
        self.nodes_el.set_html("");
    }

    pub fn load(&mut self, graph: PenguinGraph) {
        self.clear();

        for (id, node) in graph.nodes {
            self.apply_command(&Command::AddNode { id, node }, false);
        }

        for (id, wire) in graph.wires {
            self.apply_command(
                &Command::AddWire {
                    id,
                    wire: wire.clone(),
                },
                false,
            )
        }
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
