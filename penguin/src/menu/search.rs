use crate::{
    app::event::{EventTarget, ListenerBuilder, Listeners, document},
    graph::node,
};
use igloo_interface::{PenguinNodeDefn, PenguinNodeDefnRef, PenguinPinRef, PenguinRegistry};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlInputElement};

#[derive(Debug)]
pub struct MenuSearch {
    input: HtmlInputElement,
    results: Element,
    items: Vec<MenuSearchItem>,
}

impl Drop for MenuSearch {
    fn drop(&mut self) {
        self.input.remove();
        self.results.remove();
    }
}

impl MenuSearch {
    pub fn new(registry: &PenguinRegistry, parent: &Element) -> Result<Self, JsValue> {
        let document = document();

        let input = document
            .create_element("input")?
            .dyn_into::<HtmlInputElement>()?;
        input.set_id("penguin-menu-search-input");
        input.set_attribute("type", "text")?;
        input.set_attribute("placeholder", "Search nodes...")?;
        // TODO replace with Rust listener
        input.set_attribute(
            "oninput",
            r#"document.querySelectorAll('#penguin-menu-search-results > .penguin-menu-search-item[data-compatible]').forEach(item => {
                 item.style.display = item.textContent.toLowerCase().includes(this.value.toLowerCase()) ? '' : 'none';
            })"#
        )?;
        parent.append_child(&input)?;
        input.focus()?;

        let results = document.create_element("div")?;
        results.set_id("penguin-menu-search-results");
        parent.append_child(&results)?;

        let mut items = Vec::with_capacity(1000);

        for (lib_path, lib) in &registry.libraries {
            for (node_path, defn) in &lib.nodes {
                if defn.hide_search {
                    continue;
                }

                items.push(MenuSearchItem::new(
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

    pub fn show(&mut self, from_pin: &Option<PenguinPinRef>) -> Result<(), JsValue> {
        for item in &mut self.items {
            let comp = if let Some(ws) = from_pin {
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
        self.input.focus()?;
        self.results.remove_attribute("style")
    }
}

#[derive(Debug)]
pub struct MenuSearchItem {
    el: Element,
    defn: PenguinNodeDefn,
    listeners: Listeners,
}

impl Drop for MenuSearchItem {
    fn drop(&mut self) {
        self.el.remove();
    }
}

impl MenuSearchItem {
    pub fn new(
        parent: &Element,
        lib_path: String,
        node_path: String,
        defn: PenguinNodeDefn,
    ) -> Result<Self, JsValue> {
        let document = document();

        let el = document.create_element("button")?;
        el.set_class_name("penguin-menu-search-item");
        parent.append_child(&el)?;

        let listeners = ListenerBuilder::new(
            &el,
            EventTarget::MenuSearchItem(PenguinNodeDefnRef::new(
                &lib_path,
                &node_path,
                defn.version,
            )),
        )
        .add_mouseclick()?
        .build();

        let title = document.create_element("div")?;
        title.set_class_name("penguin-menu-search-item-title");
        title.set_inner_html(&node_path);
        el.append_child(&title)?;

        // let content = document.create_element("div")?;
        // content.set_class_name("penguin-menu-search-item-content");
        node::make_dummy(&el, &defn)?;
        // el.append_child(&content)?;

        // let path = document.create_element("div")?;
        // path.set_class_name("penguin-menu-search-item-path");
        // path.set_inner_html(&format!("{lib_path}.{node_path}"));
        // el.append_child(&path)?;

        Ok(Self {
            el,
            defn,
            listeners,
        })
    }

    // pub fn hide(&self) -> Result<(), JsValue> {
    //     self.el.set_attribute("style", "display: none;")
    // }

    // pub fn show(&self) -> Result<(), JsValue> {
    //     self.el.remove_attribute("style")
    // }
}
