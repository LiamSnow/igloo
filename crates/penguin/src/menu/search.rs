use crate::{
    dom::{self, Button, Div, Input, events::EventTarget, node::DomNode},
    graph::node,
};
use igloo_interface::penguin::{
    PenguinNodeDefn, PenguinNodeDefnRef, PenguinPinRef, PenguinRegistry,
};

#[derive(Debug)]
pub struct MenuSearch {
    input: DomNode<Input>,
    results: DomNode<Div>,
    items: Vec<MenuSearchItem>,
}

impl MenuSearch {
    pub fn new<T>(registry: &PenguinRegistry, parent: &DomNode<T>) -> Self {
        let input = dom::input()
            .id("penguin-menu-search-input")
            .type_attr("text")
            .placeholder("Search nodes...")
            .event_target(EventTarget::MenuSearch)
            .listen_input_no_value()
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

        let results = dom::div()
            .id("penguin-menu-search-results")
            .hide()
            .mount(parent);

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
                ));
            }
        }

        Self {
            input,
            results,
            items,
        }
    }

    pub fn hide(&self) {
        self.input.hide();
        self.results.hide();
    }

    pub fn show(&mut self, from_pin: &Option<PenguinPinRef>) {
        for item in &mut self.items {
            item.compatible = if let Some(ws) = from_pin {
                ws.find_compatible(&item.defn).is_some()
            } else {
                true
            };

            if item.compatible {
                item.show();
            } else {
                item.hide();
            }
        }

        self.input.show();
        self.input.set_value("");
        self.input.focus();
        self.results.show();
    }

    pub fn handle_input(&mut self) {
        let value = self.input.value().to_lowercase();

        for item in &mut self.items {
            if !item.compatible {
                continue;
            }

            if item.node_name_lower.contains(&value) {
                item.show();
            } else {
                item.hide();
            }
        }
    }
}

#[derive(Debug)]
pub struct MenuSearchItem {
    el: DomNode<Button>,
    defn: PenguinNodeDefn,
    compatible: bool,
    node_name_lower: String,
    shown: bool,
}

impl MenuSearchItem {
    pub fn new<T>(
        parent: &DomNode<T>,
        lib_name: String,
        node_name: String,
        defn: PenguinNodeDefn,
    ) -> Self {
        let node_name_lower = node_name.to_lowercase();

        let mut el = dom::button()
            .class("penguin-menu-search-item")
            .mount(parent);

        dom::div()
            .class("penguin-menu-search-item-title")
            .text(&node_name)
            .mount(&el);

        let defn_ref = PenguinNodeDefnRef {
            lib_name,
            node_name,
            version: defn.version,
        };

        node::make_dummy(&el, &defn, &defn_ref);

        el.event_target(EventTarget::MenuSearchItem(defn_ref));
        el.listen_click();

        Self {
            el,
            defn,
            compatible: true,
            shown: true,
            node_name_lower,
        }
    }

    pub fn hide(&mut self) {
        if self.shown {
            self.shown = false;
            self.el.hide();
        }
    }

    pub fn show(&mut self) {
        if !self.shown {
            self.shown = true;
            self.el.show();
        }
    }
}
