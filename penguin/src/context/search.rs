use std::collections::HashMap;

use igloo_interface::{PenguinNodeDefn, PenguinNodeDefnRef, PenguinRegistry, graph::PenguinNode};
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{Element, HtmlElement, MouseEvent};

use crate::{app::APP, ffi, viewport::WorldPoint};

#[derive(Debug)]
pub struct ContextSearch {
    input: HtmlElement,
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
    pub fn new(
        registry: &PenguinRegistry,
        parent: &Element,
        wpos: WorldPoint,
    ) -> Result<Self, JsValue> {
        let document = ffi::document();

        let input = document
            .create_element("input")?
            .dyn_into::<HtmlElement>()?;
        input.set_id("penguin-context-search-input");
        input.set_attribute("type", "text")?;
        input.set_attribute("placeholder", "Search nodes...")?;
        input.set_attribute(
            "oninput", 
            r#"document.querySelectorAll('.penguin-context-search-item').forEach(item => {
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
                    wpos,
                )?);
            }
        }

        Ok(Self {
            input,
            results,
            items,
        })
    }
}

#[derive(Debug)]
pub struct ContextSearchItem {
    el: Element,
    closure: Closure<dyn FnMut(MouseEvent)>,
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
        wpos: WorldPoint,
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

        let closure = Closure::wrap(Box::new(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();

            APP.with(|app| {
                let mut b = app.borrow_mut();
                let Some(app) = b.as_mut() else {
                    return;
                };

                app.graph.place_node(
                    &app.registry,
                    PenguinNode {
                        defn_ref: PenguinNodeDefnRef::new(&lib_path, &node_path, defn.version),
                        x: wpos.x,
                        y: wpos.y,
                        inputs: HashMap::default(),
                        values: HashMap::default(),
                    },
                );

                if !e.shift_key() {
                    app.context.hide();
                }
            });
        }) as Box<dyn FnMut(_)>);
        el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;

        Ok(Self { el, closure })
    }
}
