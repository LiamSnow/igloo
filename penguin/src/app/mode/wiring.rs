use crate::app::{
    App,
    event::{Event, EventTarget, EventValue},
    mode::{MenuMode, Mode},
};
use igloo_interface::PenguinPinRef;
use wasm_bindgen::JsValue;

pub fn start_wiring(app: &mut App, start_pin: PenguinPinRef) -> Result<(), JsValue> {
    app.graph.start_wiring(&start_pin);
    app.set_mode(Mode::Wiring(start_pin))
}

impl App {
    pub fn handle_wiring_mode(&mut self, event: Event) -> Result<(), JsValue> {
        let Mode::Wiring(ref mut start_pin) = self.mode else {
            unreachable!();
        };

        match (event.target, event.value) {
            // move wire around
            (_, EventValue::MouseMove(_)) => {
                let world_pos = self.viewport.client_to_world(self.mouse_pos);
                self.graph.update_wiring(world_pos)?;
                Ok(())
            }

            // place wire
            (EventTarget::Pin(end_pin), EventValue::MouseUp(_)) => {
                if start_pin.can_connect_to(&end_pin) {
                    self.graph.add_wire(start_pin.clone(), end_pin)?;
                }

                self.set_mode(Mode::Idle)
            }

            // open context menu to add node onto wire
            (_, EventValue::MouseUp(_)) => {
                let wpos = self.viewport.client_to_world(self.mouse_pos);

                let ws = Some(start_pin.clone());

                self.menu.show_search(&self.mouse_pos, &ws)?;
                self.set_mode(Mode::Menu(MenuMode {
                    pos: wpos,
                    from_pin: ws,
                }))
            }

            _ => Ok(()),
        }
    }

    pub fn finish_wiring_mode(&mut self) -> Result<(), JsValue> {
        self.graph.stop_wiring();
        Ok(())
    }
}
