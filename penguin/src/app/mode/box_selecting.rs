use crate::{
    app::{
        App,
        mode::{MenuMode, Mode},
    },
    dom::events::{Event, EventValue},
    viewport::{ClientBox, ClientPoint},
};

#[derive(Clone, Debug, PartialEq)]
pub struct BoxSelectingMode {
    pub start_pos: ClientPoint,
    pub append: bool,
}

impl App {
    pub fn handle_box_selecting_mode(&mut self, event: Event) {
        let Mode::BoxSelecting(ref mut bs) = self.mode else {
            unreachable!();
        };

        match &event.value {
            EventValue::MouseMove(_) => {
                let current = self.mouse_pos;

                let left = i32::min(bs.start_pos.x, current.x);
                let top = i32::min(bs.start_pos.y, current.y);
                let width = i32::abs(bs.start_pos.x - current.x);
                let height = i32::abs(bs.start_pos.y - current.y);

                self.box_el.set_style("display", "block");
                self.box_el.set_left(left as f64);
                self.box_el.set_top(top as f64);
                self.box_el.set_width(width as f64);
                self.box_el.set_height(height as f64);
            }
            EventValue::MouseUp(_) => {
                let end_pos = self.mouse_pos;
                let distance = bs
                    .start_pos
                    .cast::<f64>()
                    .distance_to(end_pos.cast::<f64>());

                if distance < 10.0 {
                    // just a click -> open context menu
                    let wpos = self.viewport.client_to_world(end_pos);
                    self.menu.show_search(end_pos, &None);
                    self.set_mode(Mode::Menu(MenuMode {
                        pos: wpos,
                        from_pin: None,
                    }));
                } else {
                    // complete box selection
                    let cbox = ClientBox::new(
                        ClientPoint::new(
                            i32::min(bs.start_pos.x, end_pos.x),
                            i32::min(bs.start_pos.y, end_pos.y),
                        ),
                        ClientPoint::new(
                            i32::max(bs.start_pos.x, end_pos.x),
                            i32::max(bs.start_pos.y, end_pos.y),
                        ),
                    );

                    self.graph.box_select(
                        cbox,
                        self.viewport.client_to_world_transform(),
                        bs.append,
                    );
                    self.set_mode(Mode::Idle);
                }
            }
            _ => {}
        }
    }

    pub fn finish_box_selecting_mode(&mut self) {
        self.box_el.hide();
    }
}
