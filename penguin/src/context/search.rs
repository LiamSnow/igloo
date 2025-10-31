use igloo_interface::{PenguinNodeDefn, PenguinNodeDefnRef, PenguinRegistry};
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{Element, HtmlInputElement, MouseEvent};

use crate::{app::APP, ffi, interaction::WiringState};

#[derive(Debug)]
pub struct ContextSearch {
    input: HtmlInputElement,
    results: Element,
    items: Vec<ContextSearchItem>,
}

impl Drop for ContextSearch {
    fn drop(&mut self) {
        self.input.remove();
        self.results.remove();
    }
}

impl ContextSearch {
    pub fn new(registry: &PenguinRegistry, parent: &Element) -> Result<Self, JsValue> {
        let document = ffi::document();

        let input = document
            .create_element("input")?
            .dyn_into::<HtmlInputElement>()?;
        input.set_id("penguin-context-search-input");
        input.set_attribute("type", "text")?;
        input.set_attribute("placeholder", "Search nodes...")?;
        input.set_attribute(
            "oninput", 
            r#"document.querySelectorAll('#penguin-context-search-results > .penguin-context-search-item[data-compatible]').forEach(item => {
                 item.style.display = item.textContent.toLowerCase().includes(this.value.toLowerCase()) ? '' : 'none';
            })"#
        )?;
        parent.append_child(&input)?;
        input.focus()?;

        let results = document.create_element("div")?;
        results.set_id("penguin-context-search-results");
        parent.append_child(&results)?;

        let mut items = Vec::with_capacity(1000);

        for (lib_path, lib) in &registry.libraries {
            for (node_path, defn) in &lib.nodes {
                if defn.hide_search {
                    continue;
                }

                items.push(ContextSearchItem::new(
                    &results,
                    lib_path.clone(),
                    node_path.clone(),
                    defn.clone(),
                )?);
            }
        }

        let me = Self {
            input,
            results,
            items,
        };

        me.hide()?;

        Ok(me)
    }

    pub fn hide(&self) -> Result<(), JsValue> {
        self.input.set_attribute("style", "display: none;")?;
        self.results.set_attribute("style", "display: none;")
    }

    pub fn show(&mut self, ws: &Option<WiringState>) -> Result<(), JsValue> {
        for item in &mut self.items {
            let comp = if let Some(ws) = ws {
                ws.find_compatible(&item.defn).is_some()
            } else {
                true
            };

            if comp {
                item.el.set_attribute("data-compatible", "")?;
                item.el.remove_attribute("style")?;
            } else {
                item.el.remove_attribute("data-compatible")?;
                item.el.set_attribute("style", "display: none;")?;
            }
        }

        self.input.remove_attribute("style")?;
        self.input.set_value("");
        self.results.remove_attribute("style")
    }
}

#[derive(Debug)]
pub struct ContextSearchItem {
    el: Element,
    closure: Closure<dyn FnMut(MouseEvent)>,
    defn: PenguinNodeDefn,
}

impl Drop for ContextSearchItem {
    fn drop(&mut self) {
        self.el.remove();
    }
}

impl ContextSearchItem {
    pub fn new(
        parent: &Element,
        lib_path: String,
        node_path: String,
        defn: PenguinNodeDefn,
    ) -> Result<Self, JsValue> {
        let document = ffi::document();

        let el = document.create_element("button")?;
        el.set_class_name("penguin-context-search-item");
        parent.append_child(&el)?;

        let title = document.create_element("div")?;
        title.set_class_name("penguin-context-search-item-title");
        title.set_inner_html(&defn.title);
        el.append_child(&title)?;

        let path = document.create_element("div")?;
        path.set_class_name("penguin-context-search-item-path");
        path.set_inner_html(&format!("{lib_path}.{node_path}"));
        el.append_child(&path)?;

        let defn_1 = defn.clone();
        let closure = Closure::wrap(Box::new(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();

            APP.with(|app| {
                let mut b = app.borrow_mut();
                let Some(app) = b.as_mut() else {
                    return;
                };

                app.context_add_node(
                    defn_1.clone(),
                    PenguinNodeDefnRef::new(&lib_path, &node_path, defn.version),
                    !e.shift_key(),
                );
            });
        }) as Box<dyn FnMut(_)>);
        el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;

        Ok(Self { el, closure, defn })
    }

    // pub fn hide(&self) -> Result<(), JsValue> {
    //     self.el.set_attribute("style", "display: none;")
    // }

    // pub fn show(&self) -> Result<(), JsValue> {
    //     self.el.remove_attribute("style")
    // }
}
