use igloo_interface::graph::PenguinNodeID;
use wasm_bindgen::JsValue;

use crate::{
    app::{
        App,
        event::{Event, EventValue},
        mode::Mode,
    },
    viewport::WorldPoint,
};

#[derive(Clone, Debug, PartialEq)]
pub struct DraggingMode {
    pub primary_node: PenguinNodeID,
    pub primary_node_pos: WorldPoint,
    pub start_pos: WorldPoint,
    pub node_poses: Vec<(PenguinNodeID, WorldPoint)>,
}

impl App {
    pub fn handle_dragging_mode(&mut self, event: Event) -> Result<(), JsValue> {
        let Mode::Dragging(ref mut dm) = self.mode else {
            unreachable!();
        };

        match event.value {
            EventValue::MouseMove(_) => {
                let wpos = self.viewport.client_to_world(self.mouse_pos);
                let delta = wpos - dm.start_pos;
                let primary_new_pos = self.viewport.snap(dm.primary_node_pos + delta);
                let delta = primary_new_pos - dm.primary_node_pos;

                for (node_id, initial_pos) in &dm.node_poses {
                    let new_pos = *initial_pos + delta;
                    self.graph.move_node(node_id, new_pos)?;
                }

                Ok(())
            }
            EventValue::MouseUp(_) => self.set_mode(Mode::Idle),
            _ => Ok(()),
        }
    }

    pub fn finish_dragging_mode(&mut self) -> Result<(), JsValue> {
        if let Mode::Dragging(dm) = &self.mode {
            let mut moves = Vec::with_capacity(dm.node_poses.len());

            for (node_id, initial_pos) in &dm.node_poses {
                let final_pos = self.graph.get_node_pos(node_id)?;

                if initial_pos != &final_pos {
                    moves.push((*node_id, *initial_pos, final_pos));
                }
            }

            self.graph.finish_moves(moves)?;
        }

        Ok(())
    }
}
