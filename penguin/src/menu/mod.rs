use igloo_interface::{PenguinPinRef, PenguinRegistry};
use wasm_bindgen::JsValue;
use web_sys::Element;

use crate::{
    app::event::{EventTarget, ListenerBuilder, Listeners, document},
    menu::search::MenuSearch,
    viewport::ClientPoint,
};

mod search;

#[derive(Debug)]
pub struct Menu {
    backdrop: Element,
    menu: Element,
    search: MenuSearch,
    listeners: Listeners,
}

impl Drop for Menu {
    fn drop(&mut self) {
        self.backdrop.remove();
    }
}

impl Menu {
    pub fn new(registry: &PenguinRegistry, parent: &Element) -> Result<Self, JsValue> {
        let document = document();

        let backdrop = document.create_element("div")?;
        backdrop.set_id("penguin-menu-backdrop");
        backdrop.set_attribute("style", "display: none;")?;
        parent.append_child(&backdrop)?;

        let menu = document.create_element("div")?;
        menu.set_id("penguin-menu-menu");
        backdrop.append_child(&menu)?;
        menu.set_attribute("onmousedown", "event.stopPropagation();")?;
        menu.set_attribute("onmouseup", "event.stopPropagation();")?;
        menu.set_attribute("onwheel", "event.stopPropagation();")?;

        let listeners = ListenerBuilder::new(&backdrop, EventTarget::MenuBackdrop)
            .add_mousedown()?
            .add_mouseup()?
            .add_mousemove()?
            .add_contextmenu()?
            .add_wheel()?
            .add_keydown()?
            .add_copy()?
            .add_paste()?
            .add_cut()?
            .build();

        Ok(Self {
            backdrop,
            search: MenuSearch::new(registry, &menu)?,
            menu,
            listeners,
        })
    }

    pub fn show_options(&mut self, pos: ClientPoint) -> Result<(), JsValue> {
        self.show()?;

        todo!()
    }

    pub fn show_search(
        &mut self,
        cpos: ClientPoint,
        from_pin: &Option<PenguinPinRef>,
    ) -> Result<(), JsValue> {
        self.show()?;
        self.search.show(from_pin)?;
        self.set_pos(cpos)
    }

    pub fn hide(&mut self) -> Result<(), JsValue> {
        self.backdrop.set_attribute("style", "display: none;")?;
        self.search.hide()?;
        Ok(())
    }

    fn show(&self) -> Result<(), JsValue> {
        self.backdrop.remove_attribute("style")
    }

    fn set_pos(&self, cpos: ClientPoint) -> Result<(), JsValue> {
        let cpos = cpos.cast::<f64>();
        let (bwidth, bheight) = {
            let rect = self.backdrop.get_bounding_client_rect();
            (rect.width(), rect.height())
        };
        let (mwidth, mheight) = {
            let rect = self.menu.get_bounding_client_rect();
            (rect.width(), rect.height())
        };

        // try at bottom right
        let mut x = cpos.x;
        let mut y = cpos.y;
        if x + mwidth <= bwidth && y + mheight <= bheight {
            let style = format!("left: {}px; top: {}px;", x, y);
            return self.menu.set_attribute("style", &style);
        }

        // try top right
        x = cpos.x;
        y = cpos.y - mheight;
        if x + mwidth <= bwidth && y >= 0.0 {
            let style = format!("left: {}px; top: {}px;", x, y);
            return self.menu.set_attribute("style", &style);
        }

        // try bottom left
        x = cpos.x - mwidth;
        y = cpos.y;
        if x >= 0.0 && y + mheight <= bheight {
            let style = format!("left: {}px; top: {}px;", x, y);
            return self.menu.set_attribute("style", &style);
        }

        // try top left
        x = cpos.x - mwidth;
        y = cpos.y - mheight;
        if x >= 0.0 && y >= 0.0 {
            let style = format!("left: {}px; top: {}px;", x, y);
            return self.menu.set_attribute("style", &style);
        }

        // clamp
        const EDGE_PADDING: f64 = 10.0;
        x = cpos.x.min(bwidth - mwidth - EDGE_PADDING).max(EDGE_PADDING);
        y = cpos
            .y
            .min(bheight - mheight - EDGE_PADDING)
            .max(EDGE_PADDING);

        let style = format!("left: {}px; top: {}px;", x, y);
        self.menu.set_attribute("style", &style)
    }

    pub fn handle_search_input(&mut self) -> Result<(), JsValue> {
        self.search.handle_input()
    }
}
