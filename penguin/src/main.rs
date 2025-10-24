#![allow(non_snake_case)]

use dioxus::{logger::tracing, prelude::*};

mod comps;
mod ffi;
mod state;
mod types;

use comps::*;
use euclid::default::Point2D;
use state::*;
use web_sys::js_sys::Array;

use crate::{
    ffi::{LMB_DOWN, RMB_DOWN},
    types::{NodeId, WireId},
};

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    let mut graph = use_store(GraphState::new);
    let mut wiring_state: Signal<Option<WiringData>> = use_signal(|| None);
    let grid_settings: Signal<GridSettings> = use_signal(GridSettings::default);

    let onkeydown = move |e: Event<KeyboardData>| match e.key() {
        Key::Delete | Key::Backspace => {
            e.prevent_default();
            handle_delete(&mut graph);
        }
        _ => {}
    };

    use_effect(move || {
        if (!LMB_DOWN() || RMB_DOWN()) && wiring_state.write().take().is_some() {
            ffi::stopWiring();
        }
    });

    use_effect(move || {
        if !LMB_DOWN() {
            sync_js_rust(&graph);
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

        div {
            id: "penguin",
            tabindex: 0,
            onkeydown,
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
                        }
                    }
                }
            }
        }
    }
}

fn handle_delete(graph: &mut Store<GraphState>) {
    let selected_nodes = ffi::getSelectedNodeIds();
    let selected_wires = ffi::getSelectedWireIds();

    let mut g = graph.write();

    for wire_id in selected_wires {
        g.wires.remove(&WireId(wire_id));
    }

    for node_id in &selected_nodes {
        g.nodes.remove(&NodeId(*node_id));
    }

    g.wires.retain(|_, wire| {
        !selected_nodes.contains(&wire.from_node.0) && !selected_nodes.contains(&wire.to_node.0)
    });

    ffi::delayedRerender();
}

fn sync_js_rust(graph: &Store<GraphState>) -> Option<()> {
    let items = ffi::getAllNodePositions();

    let mut n = graph.nodes();
    let mut nodes = n.write();

    for item in items {
        let item: Array = item.into();
        let node_id = NodeId(item.get(0).as_f64()? as u16);
        let x = item.get(1).as_f64()?;
        let y = item.get(2).as_f64()?;

        let Some(node) = nodes.get_mut(&node_id) else {
            tracing::error!("JS requested update for {node_id:?}, which doesn't exist");
            continue;
        };

        node.pos = Point2D::new(x, y);
    }

    Some(())
}
