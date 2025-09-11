use std::collections::HashMap;

use crate::penguin::{coordinates::WorldPoint, model::registry::NodeRegistry};

/// everything in a workspace, savable to file, runnable on the backend
pub struct Scene {
    /// id -> Node
    pub nodes: HashMap<u32, Node>,
    /// id -> Wire
    pub wires: HashMap<u32, Wire>,
    next_node_id: u32,
    next_wire_id: u32,
}

#[derive(Clone, PartialEq)]
pub struct Node {
    pub defn: u32,
    pub pos: WorldPoint,
}

pub struct Wire {
    kind: WireKind,
    from: NodePinRef,
    to: NodePinRef,
}

pub enum WireKind {
    Flow,
    Value,
}

pub struct NodePinRef {
    pub node: u32,
    pub pin: u32,
}

impl Scene {
    pub fn empty() -> Self {
        Self {
            nodes: HashMap::new(),
            wires: HashMap::new(),
            next_node_id: 0,
            next_wire_id: 0,
        }
    }

    pub fn add_node(&mut self, defn: u32, pos: WorldPoint) -> u32 {
        // TODO verify no id conflict
        let id = self.next_node_id;
        self.nodes.insert(id, Node { defn, pos });
        self.next_node_id += 1;
        id
    }

    pub fn connect(&mut self, kind: WireKind, from: NodePinRef, to: NodePinRef) -> u32 {
        let id = self.next_wire_id;
        self.wires.insert(id, Wire { kind, from, to });
        self.next_wire_id += 1;
        id
    }
}
