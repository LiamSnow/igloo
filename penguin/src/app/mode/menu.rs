use crate::{
    app::{App, mode::Mode},
    dom::events::{Event, EventTarget, EventValue},
    viewport::WorldPoint,
};
use igloo_interface::penguin::{PenguinPinRef, graph::PenguinNode};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct MenuMode {
    pub pos: WorldPoint,
    pub from_pin: Option<PenguinPinRef>,
}

impl App {
    pub fn handle_menu_mode(&mut self, event: Event) {
        let Mode::Menu(ref mut mm) = self.mode else {
            unreachable!();
        };

        match (event.target, event.value) {
            (EventTarget::MenuSearchItem(defn_ref), EventValue::MouseClick(e)) => {
                let defn = self
                    .graph
                    .registry
                    .get_defn(&defn_ref)
                    .expect("Unknown node definition")
                    .clone();

                let node = PenguinNode {
                    defn_ref,
                    x: mm.pos.x,
                    y: mm.pos.y,
                    input_feature_values: HashMap::with_capacity(defn.input_features.len()),
                    input_pin_values: HashMap::with_capacity(defn.inputs.len()),
                    ..Default::default()
                };

                let node_id = self.graph.place_node(node);

                // auto wire
                if let Some(start_pin) = &mm.from_pin
                    && let Some((pin_id, pin_defn)) = start_pin.find_compatible(&defn)
                {
                    let end_pin = PenguinPinRef {
                        node_id,
                        id: pin_id.clone(),
                        is_output: !start_pin.is_output,
                        r#type: pin_defn.r#type,
                    };

                    self.graph.add_wire(start_pin.clone(), end_pin);
                }

                // close menu
                if !e.shift_key() {
                    self.set_mode(Mode::Idle);
                }
            }
            _ => {}
        }
    }

    pub fn finish_menu_mode(&mut self) {
        self.menu.hide();
    }
}
