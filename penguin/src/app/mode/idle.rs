use crate::{
    app::{
        App,
        mode::{BoxSelectingMode, DraggingMode, Mode, PanningMode},
    },
    dom::events::{Clientable, Event, EventTarget, EventValue},
};

impl App {
    pub fn handle_idle_mode(&mut self, event: Event) {
        if !matches!(self.mode, Mode::Idle) {
            unreachable!();
        }

        match (event.target, event.value) {
            (EventTarget::Global, EventValue::MouseDown(e)) if e.button() == 0 => {
                self.set_mode(Mode::Panning(PanningMode {
                    start_pos: e.client_pos(),
                    last_pos: e.client_pos(),
                }));
            }

            (EventTarget::Global, EventValue::ContextMenu(e)) => {
                e.prevent_default();

                self.set_mode(Mode::BoxSelecting(BoxSelectingMode {
                    start_pos: e.client_pos(),
                    append: e.ctrl_key() || e.shift_key(),
                }));
            }

            (EventTarget::Node(node_id), EventValue::MouseDown(e)) if e.button() == 0 => {
                let selfend = e.ctrl_key() || e.shift_key();
                self.graph.select_node(node_id, selfend);

                let node_pos = self.graph.get_node_pos(&node_id);
                let selection_poses = self.graph.selection_poses();

                self.set_mode(Mode::Dragging(DraggingMode {
                    primary_node: node_id,
                    primary_node_pos: node_pos,
                    start_pos: self.viewport.client_to_world(e.client_pos()),
                    node_poses: selection_poses,
                }));
            }

            (EventTarget::Wire(wire_id), EventValue::MouseClick(e)) if e.button() == 0 => {
                if e.alt_key() {
                    self.graph.delete_wire(wire_id);
                } else {
                    let selfend = e.ctrl_key() || e.shift_key();
                    self.graph.select_wire(wire_id, selfend);
                }
            }

            (EventTarget::Wire(wire_id), EventValue::MouseDoubleClick(e)) if e.button() == 0 => {
                let cpos = e.client_pos();
                let wpos = self.viewport.client_to_world(cpos);
                self.graph.split_wire_with_reroute(wire_id, wpos);
            }

            (EventTarget::Pin(pin), EventValue::MouseDown(e)) if e.button() == 0 => {
                if e.shift_key() {
                    self.graph.select_pin_wires(&pin);
                } else if e.alt_key() {
                    self.graph.delete_pin_wires(&pin);
                } else {
                    self.start_wiring_mode(pin);
                }
            }

            _ => {}
        }
    }
}
