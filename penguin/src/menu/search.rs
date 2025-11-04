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
    listeners: Listeners,
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
        parent.append_child(&input)?;

        let listeners = ListenerBuilder::new(&input, EventTarget::MenuSearch)
            .add_input_no_value()?
            .build();

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
            listeners,
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
            item.compatible = if let Some(ws) = from_pin {
                ws.find_compatible(&item.defn).is_some()
            } else {
                true
            };

            if item.compatible {
                item.show()?;
            } else {
                item.hide()?;
            }
        }

        self.input.remove_attribute("style")?;
        self.input.set_value("");
        self.input.focus()?;
        self.results.remove_attribute("style")
    }

    pub fn handle_input(&mut self) -> Result<(), JsValue> {
        let value = self.input.value().to_lowercase();

        for item in &mut self.items {
            if !item.compatible {
                continue;
            }

            if item.node_name_lower.contains(&value) {
                item.show()?;
            } else {
                item.hide()?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct MenuSearchItem {
    el: Element,
    defn: PenguinNodeDefn,
    compatible: bool,
    node_name_lower: String,
    shown: bool,
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
        lib_name: String,
        node_name: String,
        defn: PenguinNodeDefn,
    ) -> Result<Self, JsValue> {
        let document = document();

        let el = document.create_element("button")?;
        el.set_class_name("penguin-menu-search-item");
        parent.append_child(&el)?;

        let title = document.create_element("div")?;
        title.set_class_name("penguin-menu-search-item-title");
        title.set_inner_html(&node_name);
        el.append_child(&title)?;

        node::make_dummy(&el, &defn)?;

        let node_name_lower = node_name.to_lowercase();
        let listeners = ListenerBuilder::new(
            &el,
            EventTarget::MenuSearchItem(PenguinNodeDefnRef {
                lib_name,
                node_name,
                version: defn.version,
            }),
        )
        .add_mouseclick()?
        .build();

        Ok(Self {
            el,
            defn,
            compatible: true,
            shown: true,
            node_name_lower,
            listeners,
        })
    }

    pub fn hide(&mut self) -> Result<(), JsValue> {
        if self.shown {
            self.shown = false;
            self.el.set_attribute("style", "display: none;")?;
        }
        Ok(())
    }

    pub fn show(&mut self) -> Result<(), JsValue> {
        if !self.shown {
            self.shown = true;
            self.el.remove_attribute("style")?;
        }
        Ok(())
    }
}
