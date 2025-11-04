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

    pub fn hide(&mut self) -> Result<(), JsValue> {
        self.backdrop.set_attribute("style", "display: none;")?;
        self.search.hide()?;
        Ok(())
    }

    fn show(&self, cpos: &ClientPoint) -> Result<(), JsValue> {
        self.backdrop.remove_attribute("style")?;
        let style = format!("left: {}px; top: {}px;", cpos.x, cpos.y);
        self.menu.set_attribute("style", &style)
    }

    pub fn show_options(&mut self, pos: ClientPoint) -> Result<(), JsValue> {
        todo!()
    }

    pub fn show_search(
        &mut self,
        cpos: &ClientPoint,
        from_pin: &Option<PenguinPinRef>,
    ) -> Result<(), JsValue> {
        self.show(cpos)?;
        self.search.show(from_pin)
    }
}
