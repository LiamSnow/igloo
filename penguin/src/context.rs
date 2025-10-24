use dioxus::{html::geometry::ClientPoint, signals::WritableExt};
use dioxus_stores::Store;

use crate::{ffi, graph::Graph, types::*};

#[derive(Clone, Debug, Default)]
pub struct ContextMenuState {
    pub visible: bool,
    pub x: f64,
    pub y: f64,
    pub items: Vec<ContextMenuItem>,
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
    /// node ids, wire ids
    DeleteSelection(Vec<u16>, Vec<u16>),
    DuplicateNode(NodeID),
    DuplicateWire(WireID),
    /// node ids, wire ids
    DuplicateSelection(Vec<u16>, Vec<u16>),
    CopyNode(NodeID),
    CopyWire(WireID),
    /// node ids, wire ids
    CopySelection(Vec<u16>, Vec<u16>),
    Paste,
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
            ContextMenuAction::DeleteSelection(node_ids, wire_ids) => {
                graph
                    .write()
                    .bulk_delete(node_ids.clone(), wire_ids.clone());
            }
            ContextMenuAction::DuplicateNode(node_id) => todo!(),
            ContextMenuAction::DuplicateWire(wire_id) => todo!(),
            ContextMenuAction::DuplicateSelection(items, items1) => todo!(),
            ContextMenuAction::CopyNode(node_id) => todo!(),
            ContextMenuAction::CopyWire(wire_id) => todo!(),
            ContextMenuAction::CopySelection(items, items1) => todo!(),
            ContextMenuAction::Paste => todo!(),
        }
    }
}

impl ContextMenuState {
    pub fn open_base(&mut self, pos: ClientPoint, mut items: Vec<ContextMenuItem>) {
        let sel_nodes = ffi::getSelectedNodeIds();
        let sel_wires = ffi::getSelectedWireIds();
        let has_selection = !sel_nodes.is_empty() || !sel_wires.is_empty();

        items.push(ContextMenuItem {
            label: "Paste".to_string(),
            action: ContextMenuAction::Paste,
        });

        if has_selection {
            items.push(ContextMenuItem {
                label: "Copy Selection".to_string(),
                action: ContextMenuAction::CopySelection(sel_nodes.clone(), sel_wires.clone()),
            });
            items.push(ContextMenuItem {
                label: "Duplicate Selection".to_string(),
                action: ContextMenuAction::DuplicateSelection(sel_nodes.clone(), sel_wires.clone()),
            });
            items.push(ContextMenuItem {
                label: "Delete Selection".to_string(),
                action: ContextMenuAction::DeleteSelection(sel_nodes, sel_wires),
            });
        }

        self.visible = true;
        self.x = pos.x;
        self.y = pos.y;
        self.items = items;
    }

    pub fn open_workspace(&mut self, pos: ClientPoint) {
        self.open_base(pos, vec![]);
    }

    pub fn open_node(&mut self, pos: ClientPoint, node_id: NodeID) {
        self.open_base(
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
        self.open_base(
            pos,
            vec![
                ContextMenuItem {
                    label: "Copy Wire".to_string(),
                    action: ContextMenuAction::CopyWire(wire_id),
                },
                ContextMenuItem {
                    label: "Duplicate Wire".to_string(),
                    action: ContextMenuAction::DuplicateWire(wire_id),
                },
                ContextMenuItem {
                    label: "Delete Wire".to_string(),
                    action: ContextMenuAction::DeleteWire(wire_id),
                },
                ContextMenuItem {
                    label: "Paste".to_string(),
                    action: ContextMenuAction::Paste,
                },
            ],
        );
    }
}
