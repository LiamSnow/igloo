#![allow(non_snake_case)]

use crate::{context::ContextMenuState, ffi::*, graph::*};
use comps::*;
use dioxus::{logger::tracing, prelude::*};
use igloo_interface::PenguinRegistry;
use state::*;
use std::collections::HashSet;
use wasm_bindgen::JsCast;

mod comps;
mod context;
mod ffi;
mod graph;
mod state;
mod types;

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    use_context_provider(PenguinRegistry::new);

    let mut graph = use_store(Graph::new);
    let mut wiring_state: Signal<Option<WiringData>> = use_signal(|| None);
    let grid_settings: Signal<GridSettings> = use_signal(GridSettings::default);
    let mut context_menu_state = use_signal(ContextMenuState::default);
    let mut rmb_start = use_signal(|| None);

    let connectivity = use_memo(move || {
        let mut map = HashSet::new();
        for wire in graph.wires().read().values() {
            map.insert((wire.from_node, wire.from_pin, true));
            map.insert((wire.to_node, wire.to_pin, false));
        }
        map
    });

    let onkeydown = move |e: Event<KeyboardData>| {
        if ffi::isInputFocused() {
            return;
        }

        let mods = e.modifiers();
        let ctrl = mods.ctrl();
        let shift = mods.shift();

        match e.key() {
            Key::Delete | Key::Backspace => {
                e.prevent_default();
                graph.write().delete(ffi::get_selection());
                ffi::clearSelection();
            }
            Key::Character(c) => {
                if c == "z" && ctrl && !shift {
                    e.prevent_default();
                    graph.write().undo();
                } else if (c == "y" && ctrl) || (c == "z" && ctrl && shift) {
                    e.prevent_default();
                    graph.write().redo();
                }
            }
            _ => {}
        }
    };

    let oncopy = move |e: Event<ClipboardData>| {
        if ffi::isInputFocused() {
            return;
        }

        e.prevent_default();

        if let Some(event) = e.data.downcast::<web_sys::Event>() {
            if let Some(clipboard_event) = event.dyn_ref::<web_sys::ClipboardEvent>() {
                if let Some(dt) = clipboard_event.clipboard_data() {
                    let selection = ffi::get_selection();
                    let cursor_pos = ffi::get_mouse_world_pos();

                    if let Ok(data) = graph.read().copy(&selection, cursor_pos) {
                        let _ = dt.set_data("text/plain", &data);
                    }
                }
            }
        }
    };

    let oncut = move |e: Event<ClipboardData>| {
        if ffi::isInputFocused() {
            return;
        }

        e.prevent_default();

        if let Some(event) = e.data.downcast::<web_sys::Event>() {
            if let Some(clipboard_event) = event.dyn_ref::<web_sys::ClipboardEvent>() {
                if let Some(dt) = clipboard_event.clipboard_data() {
                    let selection = ffi::get_selection();
                    let cursor_pos = ffi::get_mouse_world_pos();

                    let Ok(data) = graph.read().copy(&selection, cursor_pos) else {
                        return;
                    };
                    let _ = dt.set_data("text/plain", &data);
                    graph.write().delete(selection);
                    ffi::clearSelection();
                }
            }
        }
    };

    let onpaste = move |e: Event<ClipboardData>| {
        if ffi::isInputFocused() {
            return;
        }

        e.prevent_default();

        if let Some(event) = e.data.downcast::<web_sys::Event>() {
            if let Some(clipboard_event) = event.dyn_ref::<web_sys::ClipboardEvent>() {
                if let Some(dt) = clipboard_event.clipboard_data() {
                    if let Ok(text) = dt.get_data("text/plain") {
                        let cursor_pos = ffi::get_mouse_world_pos();
                        let _ = graph.write().paste(&text, cursor_pos);
                    }
                }
            }
        }
    };

    let oncontextmenu = move |e: Event<MouseData>| {
        rmb_start.set(Some(e.client_coordinates()));
    };

    use_effect(move || {
        if !LMB_DOWN() || RMB_DOWN() {
            let Some(ws) = wiring_state.write().take() else {
                return;
            };

            ffi::stopWiring();

            if !ws.is_output {
                return;
            }

            let pos = *MOUSE_POS.peek();
            context_menu_state
                .write()
                .open_workspace(pos, Some(ws.wire_type), Some(ws));
        }
    });

    use_effect(move || {
        if !LMB_DOWN() {
            graph.write().commit_node_moves();
        }
    });

    use_effect(move || {
        if !RMB_DOWN() {
            let Some(start) = rmb_start.write().take() else {
                return;
            };

            let pos = *MOUSE_POS.peek();
            if start.distance_to(pos) < 10. {
                context_menu_state.write().open_workspace(pos, None, None);
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

        ContextMenu { graph, state: context_menu_state }

        div {
            id: "penguin",
            tabindex: 0,
            onkeydown,
            oncopy,
            onpaste,
            oncut,
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
                            connectivity,
                            wiring_state,
                            context_menu_state,
                        }
                    }
                }
            }
        }
    }
}
