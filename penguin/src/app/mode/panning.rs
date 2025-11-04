use wasm_bindgen::JsValue;

use crate::{
    app::{
        App,
        event::{Event, EventValue},
        mode::Mode,
    },
    viewport::{ClientPoint, PenguinVector},
};

#[derive(Clone, Debug, PartialEq)]
pub struct PanningMode {
    pub start_pos: ClientPoint,
    pub last_pos: ClientPoint,
}

impl App {
    pub fn handle_panning_mode(&mut self, event: Event) -> Result<(), JsValue> {
        let Mode::Panning(ref mut pm) = self.mode else {
            unreachable!();
        };

        match event.value {
            EventValue::MouseMove(_) => {
                let delta = (self.mouse_pos - pm.last_pos).cast::<f64>();
                self.viewport.pan_by(PenguinVector::new(delta.x, delta.y))?;
                self.graph.ctw = self.viewport.client_to_world_transform();

                pm.last_pos = self.mouse_pos;

                Ok(())
            }
            EventValue::MouseUp(_) => {
                let distance = pm
                    .start_pos
                    .cast::<f64>()
                    .distance_to(pm.last_pos.cast::<f64>());

                // it was just a click
                if distance < 10.0 {
                    self.graph.clear_selection();
                }

                self.set_mode(Mode::Idle)
            }
            _ => Ok(()),
        }
    }
}
