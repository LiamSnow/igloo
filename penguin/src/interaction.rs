use igloo_interface::{
    PenguinNodeDefn, PenguinPinDefn, PenguinPinID, PenguinPinType, graph::PenguinNodeID,
};

use crate::viewport::{ClientPoint, WorldPoint};

#[derive(Debug, Clone, Default)]
pub enum Interaction {
    #[default]
    Idle,
    Panning {
        start_pos: ClientPoint,
        last_pos: ClientPoint,
    },
    Dragging {
        node_id: PenguinNodeID,
        start_client_pos: ClientPoint,
        start_node_pos: WorldPoint,
    },
    BoxSelecting {
        start_pos: ClientPoint,
        last_pos: ClientPoint,
        append: bool,
    },
    Wiring(WiringState),
    Context {
        wpos: WorldPoint,
        ws: Option<WiringState>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct WiringState {
    pub start_node: PenguinNodeID,
    pub start_pin: PenguinPinID,
    pub is_output: bool,
    pub wire_type: PenguinPinType,
}

impl WiringState {
    pub fn is_valid_end(
        &self,
        end_node: PenguinNodeID,
        end_type: PenguinPinType,
        end_is_out: bool,
    ) -> bool {
        let compatible = if self.is_output {
            self.wire_type.can_connect_to(end_type)
        } else {
            end_type.can_connect_to(self.wire_type)
        };

        self.is_output != end_is_out && compatible && self.start_node != end_node
    }

    pub fn cast_name(&self, end_type: PenguinPinType) -> Option<String> {
        if self.is_output {
            self.wire_type.cast_name(end_type)
        } else {
            end_type.cast_name(self.wire_type)
        }
    }

    pub fn find_compatible<'a>(
        &self,
        defn: &'a PenguinNodeDefn,
    ) -> Option<(&'a PenguinPinID, &'a PenguinPinDefn)> {
        let t = if self.is_output {
            &defn.inputs
        } else {
            &defn.outputs
        };

        for (id, defn) in t {
            if defn.r#type == self.wire_type {
                return Some((id, defn));
            }
        }

        None
    }
}
