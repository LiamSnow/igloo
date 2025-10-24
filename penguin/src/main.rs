#![allow(non_snake_case)]

use dioxus::prelude::*;

mod comps;
mod context;
mod ffi;
mod graph;
mod state;
mod types;

use comps::*;
use state::*;

use crate::{context::ContextMenuState, ffi::*, graph::*};

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    let mut graph = use_store(Graph::new);
    let mut wiring_state: Signal<Option<WiringData>> = use_signal(|| None);
    let grid_settings: Signal<GridSettings> = use_signal(GridSettings::default);
    let mut context_menu_state = use_signal(ContextMenuState::default);
    let mut rmb_start = use_signal(|| None);

    let onkeydown = move |e: Event<KeyboardData>| match e.key() {
        Key::Delete | Key::Backspace => {
            e.prevent_default();
            graph.write().delete_selection();
        }
        _ => {}
    };

    let oncontextmenu = move |e: Event<MouseData>| {
        rmb_start.set(Some(e.client_coordinates()));
    };

    use_effect(move || {
        if (!LMB_DOWN() || RMB_DOWN()) && wiring_state.write().take().is_some() {
            ffi::stopWiring();
        }
    });

    use_effect(move || {
        if !LMB_DOWN() {
            graph.write().sync_from_js();
        }
    });

    use_effect(move || {
        if !RMB_DOWN() {
            let Some(start) = rmb_start.write().take() else {
                return;
            };

            let pos = *MOUSE_POS.peek();
            if start.distance_to(pos) < 10. {
                context_menu_state.write().open_workspace(pos);
            }
        }
    });

    use_effect(move || {
        let set = grid_settings();
        ffi::setGridSettings(set.enabled, set.snap, set.size);
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/penguin.css") }

        div {
            id: "penguin-selection-box",
            display: "none",
        }

        ContextMenu { graph, context_menu_state }

        div {
            id: "penguin",
            tabindex: 0,
            onkeydown,
            oncontextmenu,
            onmount: move |_| {
                ffi::init();
                ffi::register_listeners();
                let set = grid_settings();
                ffi::setGridSettings(set.enabled, set.snap, set.size);
            },

            GridSettingsComponent { grid_settings }

            svg {
                id: "penguin-wires",

                defs {
                    pattern {
                        id: "penguin-dot-grid",
                        x: 0,
                        y: 0,
                        width: grid_settings().size,
                        height: grid_settings().size,
                        pattern_units: "userSpaceOnUse",

                        circle {
                            cx: 0,
                            cy: 0,
                            r: 1.5,
                            fill: "rgba(255, 255, 255, 0.15)",
                        }
                    }
                }

                if grid_settings().enabled {
                    rect {
                        id: "grid-background",
                        x: -10000,
                        y: -10000,
                        width: 20000,
                        height: 20000,
                        fill: "url(#penguin-dot-grid)",
                    }
                }

                for (id, wire) in graph.wires()() {
                    WireComponent {
                        id,
                        wire,
                        context_menu_state,
                    }
                }

                if let Some(ws) = wiring_state() {
                    path {
                        id: "penguin-temp-wire",
                        fill: "none",
                        stroke: ws.wire_type.stroke(),
                        stroke_width: ws.wire_type.stroke_width(),
                        stroke_dasharray: "5 5",
                    }
                }
            }

            div {
                id: "penguin-viewport",

                div {
                    id: "penguin-nodes",
                    for (id, node) in graph.nodes().iter() {
                        NodeComponent {
                            graph,
                            id,
                            node,
                            wiring_state,
                            context_menu_state,
                        }
                    }
                }
            }
        }
    }
}
