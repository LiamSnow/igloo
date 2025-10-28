use crate::{
    context::{ContextMenuMode, ContextMenuState},
    ffi,
    graph::Graph,
    state::WiringData,
};
use dioxus::prelude::*;
use euclid::default::Point2D;
use igloo_interface::{NodeDefnRef, PenguinRegistry};

#[component]
pub fn ContextMenu(
    graph: Store<Graph>,
    state: Signal<ContextMenuState>,
    wiring_state: Signal<Option<WiringData>>,
) -> Element {
    if !state().visible {
        return rsx! {};
    }

    use_effect(move || {
        if !state().visible {
            ffi::changeInteractionMode("idle".to_string());
            wiring_state.set(None);
        }
    });

    rsx! {
        div {
            id: "context-menu-backdrop",
            onclick: move |_| {
                state.write().visible = false;
            },
            oncontextmenu: move |e| {
                e.prevent_default();
                state.write().visible = false;
            },
            onkeydown: move |e| {
                if e.key() == Key::Escape {
                    state.write().visible = false;
                }
            },
        }

        div {
            id: "context-menu",
            style: "left: {state().pos.x}px; top: {state().pos.y}px;",
            onclick: move |e| {
                e.stop_propagation();
            },

            match state().mode {
                ContextMenuMode::Items(items) => rsx! {
                    for (idx, item) in items.into_iter().enumerate() {
                        button {
                            key: "{idx}",
                            class: "context-menu-item",
                            onclick: move |_| {
                                item.action.exec(&mut graph);
                                state.write().visible = false;
                            },
                            "{item.label}"
                        }
                    }
                },
                ContextMenuMode::PlaceNode { filter, pending_wire } => rsx! {
                    NodePlaceMenu { graph, state, filter, pending_wire }
                }
            }
        }
    }
}

#[component]
fn NodePlaceMenu(
    graph: Store<Graph>,
    state: Signal<ContextMenuState>,
    filter: Option<igloo_interface::PinType>,
    pending_wire: Option<crate::state::WiringData>,
) -> Element {
    let registry = use_context::<PenguinRegistry>();
    let registry_1 = registry.clone();
    let mut search_query = use_signal(String::new);

    let filtered_nodes = use_memo(move || registry_1.filter_nodes(&search_query(), filter));

    rsx! {
        input {
            class: "node-search-input",
            r#type: "text",
            placeholder: "Search nodes...",
            value: "{search_query}",
            oninput: move |evt| {
                search_query.set(evt.value());
            },
            onmounted: move |e| {
                spawn(async move {
                    let _ = e.set_focus(true).await;
                });
            }
        }

        div {
            class: "node-search-results",
            for (lib_name, node_name, defn) in filtered_nodes() {
                {
                    let defn_ref = NodeDefnRef::new(&lib_name, &node_name);
                    let pending_wire = pending_wire.clone();
                    let reg = registry.clone();

                    rsx! {
                        button {
                            class: "node-search-item",
                            onclick: move |_| {
                                let pos = state().pos;
                                let world_coords = ffi::clientToWorld(pos.x, pos.y);
                                let world_pos = Point2D::new(world_coords[0], world_coords[1]);

                                graph.write().place_and_connect_node(
                                    defn_ref.clone(),
                                    world_pos,
                                    pending_wire.clone(),
                                    &reg,
                                );

                                state.write().visible = false;
                            },
                            div {
                                class: "node-search-item-title",
                                "{defn.title}"
                            }
                            div {
                                class: "node-search-item-path",
                                "{lib_name}.{node_name}"
                            }
                        }
                    }
                }
            }
        }
    }
}
