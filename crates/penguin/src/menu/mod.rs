use crate::{
    dom::{self, Div, events::EventTarget, node::DomNode},
    menu::search::MenuSearch,
    viewport::ClientPoint,
};
use igloo_interface::penguin::{PenguinPinRef, PenguinRegistry};

mod search;

#[derive(Debug)]
pub struct Menu {
    backdrop: DomNode<Div>,
    menu: DomNode<Div>,
    search: MenuSearch,
}

impl Menu {
    pub fn new<T>(registry: &PenguinRegistry, parent: &DomNode<T>) -> Self {
        let backdrop = dom::div()
            .id("penguin-menu-backdrop")
            .hide()
            .event_target(EventTarget::MenuBackdrop)
            .listen_mousedown()
            .listen_contextmenu()
            .attr("onmouseup", "event.stopPropagation();")
            .attr("onmousemove", "event.stopPropagation();")
            .attr("onwheel", "event.stopPropagation();")
            .attr("onkeydown", "event.stopPropagation();")
            .attr("oncopy", "event.stopPropagation();")
            .attr("onpaste", "event.stopPropagation();")
            .attr("oncut", "event.stopPropagation();")
            .mount(parent);

        let menu = dom::div()
            .id("penguin-menu-menu")
            .attr("onmousedown", "event.stopPropagation();")
            .attr("onmouseup", "event.stopPropagation();")
            .attr("onmousemove", "event.stopPropagation();")
            .attr("oncontextmenu", "event.stopPropagation();")
            .attr("onwheel", "event.stopPropagation();")
            .attr("onkeydown", "event.stopPropagation();")
            .attr("oncopy", "event.stopPropagation();")
            .attr("onpaste", "event.stopPropagation();")
            .attr("oncut", "event.stopPropagation();")
            .hide()
            .mount(parent);

        Self {
            backdrop,
            search: MenuSearch::new(registry, &menu),
            menu,
        }
    }

    // pub fn show_options(&mut self, _pos: ClientPoint) {
    //     self.show();

    //     todo!()
    // }

    pub fn show_search(&mut self, cpos: ClientPoint, from_pin: &Option<PenguinPinRef>) {
        self.show();
        self.search.show(from_pin);
        self.set_pos(cpos);
    }

    pub fn hide(&mut self) {
        self.backdrop.hide();
        self.menu.hide();
        self.search.hide();
    }

    fn show(&self) {
        self.backdrop.show();
        self.menu.show();
    }

    fn set_pos(&self, cpos: ClientPoint) {
        let cpos = cpos.cast::<f64>();
        let (bwidth, bheight) = {
            let rect = self.backdrop.client_box();
            (rect.width(), rect.height())
        };
        let (mwidth, mheight) = {
            let rect = self.menu.client_box();
            (rect.width(), rect.height())
        };

        // try at bottom right
        let mut x = cpos.x;
        let mut y = cpos.y;
        if x + mwidth <= bwidth && y + mheight <= bheight {
            self.menu.set_left(x);
            self.menu.set_top(y);
        }

        // try top right
        x = cpos.x;
        y = cpos.y - mheight;
        if x + mwidth <= bwidth && y >= 0.0 {
            self.menu.set_left(x);
            self.menu.set_top(y);
        }

        // try bottom left
        x = cpos.x - mwidth;
        y = cpos.y;
        if x >= 0.0 && y + mheight <= bheight {
            self.menu.set_left(x);
            self.menu.set_top(y);
        }

        // try top left
        x = cpos.x - mwidth;
        y = cpos.y - mheight;
        if x >= 0.0 && y >= 0.0 {
            self.menu.set_left(x);
            self.menu.set_top(y);
        }

        // clamp
        const EDGE_PADDING: f64 = 10.0;
        x = cpos.x.min(bwidth - mwidth - EDGE_PADDING).max(EDGE_PADDING);
        y = cpos
            .y
            .min(bheight - mheight - EDGE_PADDING)
            .max(EDGE_PADDING);

        self.menu.set_left(x);
        self.menu.set_top(y);
    }

    pub fn handle_search_input(&mut self) {
        self.search.handle_input();
    }
}
