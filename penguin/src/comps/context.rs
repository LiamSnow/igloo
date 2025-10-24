use crate::{context::ContextMenuState, graph::Graph};
use dioxus::prelude::*;

#[component]
pub fn ContextMenu(graph: Store<Graph>, context_menu_state: Signal<ContextMenuState>) -> Element {
    let state = context_menu_state();

    if !state.visible {
        return rsx! {};
    }

    rsx! {
        div {
            id: "context-menu-backdrop",
            onclick: move |_| {
                context_menu_state.write().visible = false;
            },
            oncontextmenu: move |e| {
                e.prevent_default();
                context_menu_state.write().visible = false;
            },
        }

        div {
            id: "context-menu",
            style: "left: {state.x}px; top: {state.y}px;",
            onclick: move |e: Event<MouseData>| {
                e.stop_propagation();
            },

            for (idx, item) in state.items.into_iter().enumerate() {
                button {
                    key: "{idx}",
                    class: "context-menu-item",
                    onclick: move |_| {
                        item.action.exec(&mut graph);
                        context_menu_state.write().visible = false;
                    },
                    "{item.label}"
                }
            }
        }
    }
}
