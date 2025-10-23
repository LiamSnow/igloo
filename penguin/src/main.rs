#![allow(non_snake_case)]

use std::collections::HashSet;
use std::rc::Rc;

use dioxus::html::geometry::PixelsRect;
use dioxus::html::input_data::MouseButton;
use dioxus::logger::tracing;
use dioxus::prelude::*;

mod comps;
mod state;
mod sys;
mod types;

use comps::*;
use dioxus::web::WebEventExt;
use euclid::default::{Point2D, Rect};
use euclid::Size2D;
use state::*;
use sys::*;
use wasm_bindgen::JsCast;

fn main() {
    sys::setup();
    dioxus::launch(App);
}

fn App() -> Element {
    let mut graph = use_store(GraphState::new);
    let mut selected = use_store(Selected::default);
    let mut viewport = use_signal(ViewportState::new);
    let mut interaction = use_signal(|| InteractionMode::Idle);

    let onmousedown = move |e: Event<MouseData>| {
        if e.trigger_button() != Some(MouseButton::Primary) {
            return;
        }
        let vp = viewport.read();
        interaction.set(InteractionMode::Panning {
            start: vp.pan,
            has_moved: false,
        });
    };

    let oncontextmenu = move |e: Event<MouseData>| {
        e.prevent_default();

        let cc = e.data().client_coordinates();
        let mods = e.modifiers();
        interaction.set(InteractionMode::BoxSelect {
            start: Point2D::new(cc.x, cc.y),
            has_moved: false,
            append: mods.shift() || mods.ctrl(),
        });
    };

    use_effect(move || {
        let delta = MOUSE_DELTA();
        let m = interaction.peek().clone();
        match m {
            InteractionMode::Panning { start, has_moved } => {
                let mut vp = viewport.write();
                vp.pan += delta;

                if !has_moved && vp.pan.distance_to(start) > DRAG_THRESHOLD {
                    interaction.set(InteractionMode::Panning {
                        start,
                        has_moved: true,
                    });
                }
            }
            InteractionMode::Dragging => {
                let zoom = viewport.peek().zoom;
                let delta = delta / zoom;
                let ids = selected.nodes();
                for id in ids().iter() {
                    let mut g = graph.nodes();
                    let mut n = g.write();
                    if let Some(node) = n.get_mut(id) {
                        node.position += delta;
                    }
                }
            }
            _ => {}
        }
    });

    use_effect(move || {
        let mpos = MOUSE_POSITION();
        let m = interaction.peek().clone();
        if let InteractionMode::BoxSelect {
            start,
            has_moved,
            append,
        } = m
        {
            if !has_moved && start.distance_to(mpos) > DRAG_THRESHOLD {
                interaction.set(InteractionMode::BoxSelect {
                    start,
                    has_moved: true,
                    append,
                });
            }
        }
    });

    use_effect(move || {
        if !LMB_DOWN() {
            let m = interaction.peek().clone();
            if let InteractionMode::Panning { has_moved, .. } = m {
                if !has_moved {
                    selected.nodes().set(HashSet::default());
                    selected.wires().set(HashSet::default());
                }
            }
            interaction.set(InteractionMode::Idle);
        }
    });

    use_effect(move || {
        if !RMB_DOWN() {
            let m = interaction.peek().clone();

            let InteractionMode::BoxSelect {
                start,
                has_moved,
                append,
            } = m
            else {
                return;
            };

            interaction.set(InteractionMode::Idle);

            // TODO open context menu to place node
            if !has_moved {
            }
            // complete box sel
            else {
                spawn(async move {
                    let mut selected = selected.write();

                    if !append {
                        selected.nodes.clear();
                        selected.wires.clear();
                    }

                    let select_rect = points_to_rect(start, *MOUSE_POSITION.peek());
                    for (id, node) in graph.nodes().read().iter() {
                        let Some(el) = &node.el else {
                            continue;
                        };

                        let node_rect = match el.get_client_rect().await {
                            Ok(r) => r,
                            Err(e) => {
                                tracing::error!("Node mount error: {e}");
                                return;
                            }
                        };

                        if node_rect.intersection(&select_rect).is_some() {
                            selected.nodes.insert(*id);
                        }
                    }

                    let vp = viewport.peek();
                    let world_select_rect = {
                        Rect::new(
                            Point2D::new(
                                (select_rect.origin.x - vp.pan.x) / vp.zoom,
                                (select_rect.origin.y - vp.pan.y) / vp.zoom,
                            ),
                            Size2D::new(
                                select_rect.size.width / vp.zoom,
                                select_rect.size.height / vp.zoom,
                            ),
                        )
                    };

                    let w = graph.wires();
                    let wires = w.read();
                    for (wire_id, wire) in wires.iter() {
                        let Some(el) = &wire.el else {
                            continue;
                        };

                        let elw = el.as_web_event();
                        let Some(path_el) = elw.dyn_ref::<web_sys::SvgPathElement>() else {
                            continue;
                        };

                        let total_length = path_el.get_total_length();

                        let sample_interval = 5.0;
                        let num_samples = (total_length / sample_interval).ceil() as i32;

                        let mut intersects = false;
                        for i in 0..=num_samples {
                            let distance = (i as f32) * sample_interval;
                            let Ok(point) = path_el.get_point_at_length(distance) else {
                                continue;
                            };

                            let point = Point2D::new(point.x() as f64, point.y() as f64);

                            if world_select_rect.contains(point) {
                                intersects = true;
                                break;
                            }
                        }

                        if intersects {
                            selected.wires.insert(*wire_id);
                        }
                    }
                });
            }
        }
    });

    let mut penguin_el = use_signal(|| None);

    let onwheel = move |e: Event<WheelData>| {
        e.prevent_default();

        let Some(penguin_el): Option<Rc<MountedData>> = penguin_el() else {
            tracing::error!("Penguin never set mounted element");
            return;
        };

        spawn(async move {
            let rect = match penguin_el.get_client_rect().await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Penguin mount error: {e}");
                    return;
                }
            };

            let cc = e.client_coordinates();
            let mouse_x = cc.x - rect.origin.x;
            let mouse_y = cc.y - rect.origin.y;

            let mut vp = viewport.write();
            // FIXME remove strip units
            let zoom_delta = if e.delta().strip_units().y > 0.0 {
                0.9
            } else {
                1.1
            };
            let new_zoom = (vp.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);

            let zoom_ratio = new_zoom / vp.zoom;
            vp.pan.x = mouse_x - (mouse_x - vp.pan.x) * zoom_ratio;
            vp.pan.y = mouse_y - (mouse_y - vp.pan.y) * zoom_ratio;
            vp.zoom = new_zoom;
        });
    };

    let onkeydown = move |e: Event<KeyboardData>| match e.data().key() {
        Key::Escape => {
            let mut s = selected.write();
            s.wires = HashSet::default();
            s.nodes = HashSet::default();
        }
        Key::Delete | Key::Backspace => {
            let mut g = graph.write();
            let s = selected.read();

            for wire_id in &s.wires {
                g.wires.remove(wire_id);
            }

            for node_id in &s.nodes {
                g.nodes.remove(node_id);
            }

            g.wires.retain(|_, wire| {
                !s.nodes.contains(&wire.from.node_id) && !s.nodes.contains(&wire.to.node_id)
            });
        }
        _ => {}
    };

    let mut view_box = use_signal(|| None);
    use_effect(move || {
        let viewport = viewport();

        spawn(async move {
            let Some(penguin_el): Option<Rc<MountedData>> = penguin_el() else {
                return;
            };
            let Ok(rect) = penguin_el.get_client_rect().await else {
                return;
            };
            let x = -viewport.pan.x / viewport.zoom;
            let y = -viewport.pan.y / viewport.zoom;
            let width = rect.width() / viewport.zoom;
            let height = rect.height() / viewport.zoom;
            view_box.set(Some(format!("{x} {y} {width} {height}")))
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/penguin.css") }

        if let InteractionMode::BoxSelect { start, .. } = interaction() {
            div {
                id: "penguin-selection-box",
                style: selection_box_style(start, MOUSE_POSITION())
            }
        }

        div {
            id: "penguin",
            onmousedown,
            onwheel,
            oncontextmenu,
            onkeydown,
            tabindex: 0,
            onmounted: move |e| {
                penguin_el.set(Some(e.data()));
            },

            svg {
                id: "penguin-wires",
                view_box,

                for (id, wire) in graph.wires().iter() {
                    WireComponent {
                        graph,
                        id,
                        wire,
                        selected,
                        viewport,
                    }
                }

                if let InteractionMode::Wiring { start, is_output, typ } = interaction() {
                    TempWireComponent { graph, start, is_output, typ, viewport }
                }
            }

            div {
                id: "penguin-viewport",
                transform: "translate({viewport.read().pan.x}px, {viewport.read().pan.y}px) scale({viewport.read().zoom})",

                div {
                    id: "penguin-nodes",
                    for (id, node) in graph.nodes().iter() {
                        NodeComponent {
                            graph,
                            id,
                            node,
                            selected,
                            interaction,
                        }
                    }
                }
            }
        }
    }
}

fn points_to_rect(a: Point2D<f64>, b: Point2D<f64>) -> PixelsRect {
    PixelsRect::new(
        euclid::Point2D::new(f64::min(a.x, b.x), f64::min(a.y, b.y)),
        Size2D::new(f64::abs(a.x - b.x), f64::abs(a.y - b.y)),
    )
}

fn selection_box_style(a: Point2D<f64>, b: Point2D<f64>) -> String {
    let rect = points_to_rect(a, b);
    format!(
        "left: {}px; top: {}px; width: {}px; height: {}px;",
        rect.origin.x, rect.origin.y, rect.size.width, rect.size.height
    )
}
