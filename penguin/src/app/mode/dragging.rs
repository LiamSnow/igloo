use crate::{
    app::{App, mode::Mode},
    dom::events::{Event, EventValue},
    viewport::WorldPoint,
};
use igloo_interface::penguin::graph::PenguinNodeID;

#[derive(Clone, Debug, PartialEq)]
pub struct DraggingMode {
    pub primary_node: PenguinNodeID,
    pub primary_node_pos: WorldPoint,
    pub start_pos: WorldPoint,
    pub node_poses: Vec<(PenguinNodeID, WorldPoint)>,
}

impl App {
    pub fn start_dragging_mode(&mut self, dm: DraggingMode) {
        self.el
            .set_class("disable-wire-events disable-node-events disable-pin-events");
        self.set_mode(Mode::Dragging(dm))
    }

    pub fn handle_dragging_mode(&mut self, event: Event) {
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
                    self.graph.move_node(node_id, new_pos);
                }
            }
            EventValue::MouseUp(_) => {
                self.start_idle_mode();
            }
            _ => {}
        }
    }

    pub fn finish_dragging_mode(&mut self) {
        self.el.set_class("");

        if let Mode::Dragging(dm) = &self.mode {
            let mut moves = Vec::with_capacity(dm.node_poses.len());

            for (node_id, initial_pos) in &dm.node_poses {
                let final_pos = self.graph.get_node_pos(node_id);

                if initial_pos != &final_pos {
                    moves.push((*node_id, *initial_pos, final_pos));
                }
            }

            self.graph.finish_moves(moves);
        }
    }
}
