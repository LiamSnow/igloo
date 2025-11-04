use igloo_interface::{PenguinPinRef, PenguinRegistry};
use wasm_bindgen::JsValue;
use web_sys::Element;

use crate::{
    app::event::{EventTarget, ListenerBuilder, Listeners, document},
    context::search::ContextSearch,
    viewport::ClientPoint,
};

mod search;

#[derive(Debug)]
pub struct ContextMenu {
    backdrop: Element,
    menu: Element,
    search: ContextSearch,
    listeners: Listeners,
}

impl Drop for ContextMenu {
    fn drop(&mut self) {
        self.backdrop.remove();
    }
}

impl ContextMenu {
    pub fn new(registry: &PenguinRegistry, parent: &Element) -> Result<Self, JsValue> {
        let document = document();

        let backdrop = document.create_element("div")?;
        backdrop.set_id("penguin-context-backdrop");
        backdrop.set_attribute("style", "display: none;")?;
        parent.append_child(&backdrop)?;

        let menu = document.create_element("div")?;
        menu.set_id("penguin-context-menu");
        backdrop.append_child(&menu)?;
        menu.set_attribute("onmousedown", "event.stopPropagation();")?;
        menu.set_attribute("onmouseup", "event.stopPropagation();")?;
        menu.set_attribute("onwheel", "event.stopPropagation();")?;

        let listeners = ListenerBuilder::new(&backdrop, EventTarget::ContextBackdrop)
            .add_mousedown(false)?
            .add_mouseup(true)?
            .add_mousemove(false)?
            .add_contextmenu(true)?
            .add_wheel(false)?
            .add_keydown(false)?
            .add_copy(false)?
            .add_paste(false)?
            .add_cut(false)?
            .build();

        Ok(Self {
            backdrop,
            search: ContextSearch::new(registry, &menu)?,
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
        registry: &PenguinRegistry,
        cpos: &ClientPoint,
        from_pin: &Option<PenguinPinRef>,
    ) -> Result<(), JsValue> {
        self.show(cpos)?;
        self.search.show(from_pin)
    }
}
