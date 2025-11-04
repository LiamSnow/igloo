use std::collections::HashMap;
use wasm_bindgen::JsValue;

use crate::{
    app::{
        App,
        event::{Event, EventTarget, EventValue},
        mode::Mode,
    },
    viewport::WorldPoint,
};
use igloo_interface::{PenguinPinRef, graph::PenguinNode};

#[derive(Clone, Debug, PartialEq)]
pub struct MenuMode {
    pub pos: WorldPoint,
    pub from_pin: Option<PenguinPinRef>,
}

impl App {
    pub fn handle_menu_mode(&mut self, event: Event) -> Result<(), JsValue> {
        let Mode::Menu(ref mut mm) = self.mode else {
            unreachable!();
        };

        match (event.target, event.value) {
            (EventTarget::MenuSearchItem(defn_ref), EventValue::MouseClick(e)) => {
                let defn = self
                    .graph
                    .registry
                    .get_defn(&defn_ref)
                    .ok_or(JsValue::from_str("Unknown node definition"))?
                    .clone();

                let node = PenguinNode {
                    defn_ref,
                    x: mm.pos.x,
                    y: mm.pos.y,
                    input_feature_values: HashMap::with_capacity(defn.input_features.len()),
                    input_pin_values: HashMap::with_capacity(defn.inputs.len()),
                };

                let node_id = self.graph.place_node(node)?;

                // auto wire
                if let Some(start_pin) = &mm.from_pin {
                    if let Some((pin_id, pin_defn)) = start_pin.find_compatible(&defn) {
                        let end_pin = PenguinPinRef {
                            node_id,
                            id: pin_id.clone(),
                            is_output: !start_pin.is_output,
                            r#type: pin_defn.r#type,
                        };

                        self.graph.add_wire(start_pin.clone(), end_pin)?;
                    }
                }

                // close menu
                if !e.shift_key() {
                    self.set_mode(Mode::Idle)?;
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn finish_menu_mode(&mut self) -> Result<(), JsValue> {
        self.menu.hide()
    }
}
