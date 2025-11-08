use crate::{
    app::{
        App,
        mode::{MenuMode, Mode},
    },
    dom::events::{Event, EventTarget, EventValue},
};
use igloo_interface::penguin::PenguinPinRef;

impl App {
    pub fn start_wiring_mode(&mut self, start_pin: PenguinPinRef) {
        let ctw = self.viewport.client_to_world_transform();
        self.graph.start_wiring(&start_pin, &ctw);
        self.el.set_class("disable-wire-events");
        self.set_mode(Mode::Wiring(start_pin));
    }

    pub fn handle_wiring_mode(&mut self, event: Event) {
        let Mode::Wiring(ref mut start_pin) = self.mode else {
            unreachable!();
        };

        match (&*event.target, event.value) {
            // move wire around
            (_, EventValue::MouseMove(_)) => {
                let world_pos = self.viewport.client_to_world(self.mouse_pos);
                self.graph.update_wiring(world_pos);
            }

            // place wire
            (EventTarget::Pin(end_pin), EventValue::MouseUp(_)) => {
                if start_pin.can_connect_to(end_pin) {
                    self.graph.add_wire(start_pin.clone(), end_pin.clone());
                }

                self.start_idle_mode();
            }

            // open context menu to add node onto wire
            (_, EventValue::MouseUp(_)) => {
                let wpos = self.viewport.client_to_world(self.mouse_pos);

                let ws = Some(start_pin.clone());

                self.menu.show_search(self.mouse_pos, &ws);
                self.start_menu_mode(MenuMode {
                    pos: wpos,
                    from_pin: ws,
                });
            }

            _ => {}
        }
    }

    pub fn finish_wiring_mode(&mut self) {
        self.graph.stop_wiring();
    }
}
