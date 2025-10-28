use dioxus::{html::geometry::ClientPoint, signals::WritableExt};
use dioxus_stores::Store;
use igloo_interface::PinType;

use crate::{graph::Graph, state::WiringData, types::*};

#[derive(Clone, Debug, Default)]
pub struct ContextMenuState {
    pub mode: ContextMenuMode,
    pub visible: bool,
    pub pos: ClientPoint,
}

#[derive(Clone, Debug)]
pub enum ContextMenuMode {
    Items(Vec<ContextMenuItem>),
    PlaceNode {
        filter: Option<PinType>,
        pending_wire: Option<WiringData>,
    },
}

#[derive(Clone, Debug)]
pub struct ContextMenuItem {
    pub label: String,
    pub action: ContextMenuAction,
    // TODO icons
}

#[derive(Clone, Debug)]
pub enum ContextMenuAction {
    DeleteNode(NodeID),
    DeleteWire(WireID),
    DuplicateNode(NodeID),
    CopyNode(NodeID),
}

impl Default for ContextMenuMode {
    fn default() -> Self {
        Self::PlaceNode {
            filter: None,
            pending_wire: None,
        }
    }
}

impl ContextMenuAction {
    pub fn exec(&self, graph: &mut Store<Graph>) {
        match self {
            ContextMenuAction::DeleteNode(id) => {
                graph.write().delete_node(*id);
            }
            ContextMenuAction::DeleteWire(id) => {
                graph.write().delete_wire(id);
            }
            ContextMenuAction::DuplicateNode(node_id) => todo!(),
            ContextMenuAction::CopyNode(node_id) => todo!(),
        }
    }
}

impl ContextMenuState {
    pub fn open_items(&mut self, pos: ClientPoint, mut items: Vec<ContextMenuItem>) {
        self.visible = true;
        self.pos = pos;
        self.mode = ContextMenuMode::Items(items);
    }

    pub fn open_workspace(
        &mut self,
        pos: ClientPoint,
        filter: Option<PinType>,
        pending_wire: Option<WiringData>,
    ) {
        self.visible = true;
        self.pos = pos;
        self.mode = ContextMenuMode::PlaceNode {
            filter,
            pending_wire,
        };
    }

    pub fn open_node(&mut self, pos: ClientPoint, node_id: NodeID) {
        self.open_items(
            pos,
            vec![
                ContextMenuItem {
                    label: "Copy Node".to_string(),
                    action: ContextMenuAction::CopyNode(node_id),
                },
                ContextMenuItem {
                    label: "Duplicate Node".to_string(),
                    action: ContextMenuAction::DuplicateNode(node_id),
                },
                ContextMenuItem {
                    label: "Delete Node".to_string(),
                    action: ContextMenuAction::DeleteNode(node_id),
                },
            ],
        );
    }

    pub fn open_wire(&mut self, pos: ClientPoint, wire_id: WireID) {
        self.open_items(
            pos,
            vec![ContextMenuItem {
                label: "Delete Wire".to_string(),
                action: ContextMenuAction::DeleteWire(wire_id),
            }],
        );
    }
}
