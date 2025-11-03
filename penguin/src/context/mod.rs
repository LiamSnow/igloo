use std::any::Any;

use igloo_interface::PenguinRegistry;
use wasm_bindgen::JsValue;
use web_sys::{Element, MouseEvent};

use crate::{
    context::search::ContextSearch,
    ffi::{self, add_app_event_listener},
    interaction::{Interaction, WiringState},
    viewport::ClientPoint,
};

mod search;

#[derive(Debug)]
pub struct ContextMenu {
    backdrop: Element,
    menu: Element,
    closures: Vec<Box<dyn Any>>,
    search: ContextSearch,
}

impl Drop for ContextMenu {
    fn drop(&mut self) {
        self.backdrop.remove();
    }
}

impl ContextMenu {
    pub fn new(registry: &PenguinRegistry, penguin_el: &Element) -> Result<Self, JsValue> {
        let document = ffi::document();

        let backdrop = document.create_element("div")?;
        backdrop.set_id("penguin-context-backdrop");
        backdrop.set_attribute("style", "display: none;")?;
        penguin_el.append_child(&backdrop)?;

        let menu = document.create_element("div")?;
        menu.set_id("penguin-context-menu");
        backdrop.append_child(&menu)?;
        menu.set_attribute("onmousedown", "event.stopPropagation();")?;
        menu.set_attribute("onmouseup", "event.stopPropagation();")?;
        menu.set_attribute("onwheel", "event.stopPropagation();")?;

        let mut closures = Vec::with_capacity(2);

        add_app_event_listener(
            &backdrop,
            "mousedown",
            &mut closures,
            |app, e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();
                app.set_interaction(Interaction::Idle);
            },
        )?;

        add_app_event_listener(
            &backdrop,
            "contextmenu",
            &mut closures,
            |app, e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();
                app.set_interaction(Interaction::Idle);
            },
        )?;

        Ok(Self {
            backdrop,
            search: ContextSearch::new(registry, &menu)?,
            menu,
            closures,
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
        ws: &Option<WiringState>,
    ) -> Result<(), JsValue> {
        self.show(cpos)?;
        self.search.show(ws)
    }
}
