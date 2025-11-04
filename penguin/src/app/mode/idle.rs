use wasm_bindgen::JsValue;

use crate::app::{
    App,
    event::{Event, EventTarget, EventValue},
    mode::{BoxSelectingMode, DraggingMode, Mode, PanningMode},
};

impl App {
    pub fn handle_idle_mode(&mut self, event: Event) -> Result<(), JsValue> {
        if !matches!(self.mode, Mode::Idle) {
            unreachable!();
        }

        match (event.target, event.value) {
            (EventTarget::Global, EventValue::MouseDown(e)) if e.button == 0 => {
                self.set_mode(Mode::Panning(PanningMode {
                    start_pos: e.pos,
                    last_pos: e.pos,
                }))
            }

            (EventTarget::Global, EventValue::ContextMenu(e)) => {
                self.set_mode(Mode::BoxSelecting(BoxSelectingMode {
                    start_pos: e.pos,
                    append: e.ctrl_key || e.shift_key,
                }))
            }

            (EventTarget::Node(node_id), EventValue::MouseDown(e)) if e.button == 0 => {
                let selfend = e.ctrl_key || e.shift_key;
                self.graph.select_node(node_id, selfend);

                let node_pos = self.graph.get_node_pos(&node_id)?;
                let selection_poses = self.graph.selection_poses()?;

                self.set_mode(Mode::Dragging(DraggingMode {
                    primary_node: node_id,
                    primary_node_pos: node_pos,
                    start_pos: self.viewport.client_to_world(e.pos),
                    node_poses: selection_poses,
                }))
            }

            (EventTarget::Wire(wire_id), EventValue::MouseDown(e)) if e.button == 0 => {
                if e.alt_key {
                    self.graph.delete_wire(wire_id)?;
                } else {
                    let selfend = e.ctrl_key || e.shift_key;
                    self.graph.select_wire(wire_id, selfend);
                }
                Ok(())
            }

            (EventTarget::Pin(pin), EventValue::MouseDown(e)) if e.button == 0 => {
                if e.shift_key {
                    self.graph.select_pin_wires(&pin);
                } else if e.alt_key {
                    self.graph.delete_pin_wires(&pin);
                } else {
                    super::start_wiring(self, pin)?;
                }
                Ok(())
            }

            _ => Ok(()),
        }
    }
}
