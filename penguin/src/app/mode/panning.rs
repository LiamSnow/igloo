use crate::{
    app::{App, mode::Mode},
    dom::events::{Event, EventValue},
    viewport::{ClientPoint, PenguinVector},
};

#[derive(Clone, Debug, PartialEq)]
pub struct PanningMode {
    pub start_pos: ClientPoint,
    pub last_pos: ClientPoint,
    pub moved: bool,
}

impl App {
    pub fn start_panning_mode(&mut self, pm: PanningMode) {
        self.set_mode(Mode::Panning(pm));
    }

    pub fn handle_panning_mode(&mut self, event: Event) {
        let Mode::Panning(ref mut pm) = self.mode else {
            unreachable!();
        };

        match event.value {
            EventValue::MouseMove(_) => {
                let delta = (self.mouse_pos - pm.last_pos).cast::<f64>();
                self.viewport.pan_by(PenguinVector::new(delta.x, delta.y));
                self.graph.ctw = self.viewport.client_to_world_transform();

                pm.last_pos = self.mouse_pos;

                if !pm.moved {
                    let distance = pm
                        .start_pos
                        .cast::<f64>()
                        .distance_to(pm.last_pos.cast::<f64>());

                    if distance > 10.0 {
                        pm.moved = true;
                        self.el.set_class(
                            "disable-wire-events disable-node-events disable-pin-events",
                        );
                    }
                }
            }
            EventValue::MouseUp(_) => {
                // was just a click
                if !pm.moved {
                    self.graph.clear_selection();
                }

                self.start_idle_mode();
            }
            _ => {}
        }
    }

    pub fn finish_panning_mode(&mut self) {
        self.el.set_class("");
    }
}
