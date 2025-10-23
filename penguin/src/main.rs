#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use dioxus::html::geometry::{PixelsRect, WheelDelta};
use dioxus::html::input_data::MouseButton;
use dioxus::logger::tracing;
use dioxus::prelude::*;

mod comps;
mod state;
mod sys;
mod types;

use comps::*;
use euclid::default::{Point2D, Rect};
use euclid::Size2D;
use state::*;
use sys::*;
use wasm_bindgen::{JsCast, JsValue};

use crate::types::{NodeId, Pin, PinId, PinRef, PinType, Wire, WireId};

fn main() {
    sys::setup();
    dioxus::launch(App);
}

fn App() -> Element {
    let mut graph = use_store(GraphState::new);
    let mut selected = use_store(Selected::default);
    let mut viewport = use_signal(ViewportState::new);
    let mut interaction = use_signal(|| InteractionMode::Idle);
    let mut view_box = use_signal(|| None);

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

                drop(vp);

                let _ = update_view_box(&mut view_box, &viewport);
                update_wires(&mut graph, &viewport);
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

                update_wires(&mut graph, &viewport);
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
                let select_rect = points_to_rect(start, *MOUSE_POSITION.peek());
                complete_box_selection(&graph, &viewport, &mut selected, select_rect, append);
            }
        }
    });

    let onwheel = move |e: Event<WheelData>| {
        e.prevent_default();

        let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };
        let Ok(Some(penguin_el)) = doc.query_selector("#penguin") else {
            return;
        };

        let rect = penguin_el.get_bounding_client_rect();
        let cc = e.client_coordinates();
        let mouse_x = cc.x - rect.left();
        let mouse_y = cc.y - rect.top();

        let mut vp = viewport.write();

        let delta_y = match e.delta() {
            WheelDelta::Pixels(v) => v.y,
            WheelDelta::Lines(v) => v.y * 20.0,
            WheelDelta::Pages(v) => v.y * 800.0,
        };

        let zoom_delta = if delta_y > 0.0 { 0.9 } else { 1.1 };

        let new_zoom = (vp.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);

        let zoom_ratio = new_zoom / vp.zoom;
        vp.pan.x = mouse_x - (mouse_x - vp.pan.x) * zoom_ratio;
        vp.pan.y = mouse_y - (mouse_y - vp.pan.y) * zoom_ratio;
        vp.zoom = new_zoom;

        drop(vp);
        let _ = update_view_box(&mut view_box, &viewport);
        let _ = update_wires(&mut graph, &viewport);
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
                !s.nodes.contains(&wire.from_pin.node_id) && !s.nodes.contains(&wire.to_pin.node_id)
            });
        }
        _ => {}
    };

    let mut temp_wire = use_store(Wire::default);
    use_effect(move || {
        let InteractionMode::Wiring {
            start,
            is_output,
            typ,
        } = interaction()
        else {
            return;
        };

        let zoom = viewport().zoom;
        let pan = viewport().pan;
        let mouse = MOUSE_POSITION();

        let Ok(node_lut) = build_node_lookup() else {
            return;
        };
        let Ok(pin_lut) = build_pin_lookup() else {
            return;
        };

        let n = graph.nodes();
        let nodes = n.read();
        let Some(start_node) = nodes.get(&start.node_id) else {
            return;
        };

        let Some(start_node_el) = node_lut.get(&start.node_id) else {
            return;
        };
        let start_pin_key = (start.node_id, start.pin_id.0.clone(), is_output);
        let Some(start_pin_el) = pin_lut.get(&start_pin_key) else {
            return;
        };

        let start_pos =
            calculate_pin_world_position(start_node_el, start_pin_el, start_node.position, zoom);

        let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };
        let Ok(Some(penguin_el)) = doc.query_selector("#penguin") else {
            return;
        };
        let penguin_rect = penguin_el.get_bounding_client_rect();

        let mouse_world = Point2D::new(
            (mouse.x - penguin_rect.left() - pan.x) / zoom,
            (mouse.y - penguin_rect.top() - pan.y) / zoom,
        );

        let (from_pos, to_pos) = if is_output {
            (start_pos, mouse_world)
        } else {
            (mouse_world, start_pos)
        };

        let (stroke, stroke_width) = match typ {
            PinType::Flow => ("white".to_string(), 4),
            PinType::Value { color, .. } => (color, 2),
        };

        let mut wire = temp_wire.write();
        wire.from_pin = start.clone();
        wire.from_pos = from_pos;
        wire.to_pos = to_pos;
        wire.stroke = stroke;
        wire.stroke_width = stroke_width;
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

            svg {
                id: "penguin-wires",
                view_box,

                for (id, wire) in graph.wires().iter() {
                    WireComponent {
                        id: Some(id),
                        wire,
                        selected,
                    }
                }

                if let InteractionMode::Wiring { .. } = interaction() {
                    WireComponent {
                        id: None,
                        wire: temp_wire,
                        selected,
                    }
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
                            viewport,
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

fn update_view_box(
    view_box: &mut Signal<Option<String>>,
    viewport: &Signal<ViewportState>,
) -> Result<(), JsValue> {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return Err(JsValue::from_str("No document"));
    };
    let penguin_el = doc
        .query_selector("#penguin")?
        .ok_or(JsValue::from_str("No penguin element"))?;

    let rect = penguin_el.get_bounding_client_rect();
    let vp = viewport.read();

    let x = -vp.pan.x / vp.zoom;
    let y = -vp.pan.y / vp.zoom;
    let width = rect.width() / vp.zoom;
    let height = rect.height() / vp.zoom;

    view_box.set(Some(format!("{x} {y} {width} {height}")));

    Ok(())
}

fn complete_box_selection(
    graph: &Store<GraphState>,
    viewport: &Signal<ViewportState>,
    selected: &mut Store<Selected>,
    select_rect: PixelsRect,
    append: bool,
) -> Result<(), JsValue> {
    let mut selected = selected.write();

    if !append {
        selected.nodes.clear();
        selected.wires.clear();
    }

    let doc = web_sys::window().unwrap().document().unwrap();

    let node_els = doc.query_selector_all(".penguin-node")?;

    for i in 0..node_els.length() {
        let Some(node_el) = node_els.item(i) else {
            continue;
        };

        let Some(html_el) = node_el.dyn_ref::<web_sys::HtmlElement>() else {
            continue;
        };

        let Some(node_id_str) = html_el.get_attribute("data-node-id") else {
            continue;
        };

        let Ok(node_id_val) = node_id_str.parse::<u16>() else {
            continue;
        };

        let node_rect = html_el.get_bounding_client_rect();

        let node_pixels_rect = PixelsRect::new(
            euclid::Point2D::new(node_rect.left(), node_rect.top()),
            Size2D::new(node_rect.width(), node_rect.height()),
        );

        if node_pixels_rect.intersection(&select_rect).is_some() {
            selected.nodes.insert(NodeId(node_id_val));
        }
    }

    let vp = viewport.peek();
    let world_select_rect = Rect::new(
        Point2D::new(
            (select_rect.origin.x - vp.pan.x) / vp.zoom,
            (select_rect.origin.y - vp.pan.y) / vp.zoom,
        ),
        Size2D::new(
            select_rect.size.width / vp.zoom,
            select_rect.size.height / vp.zoom,
        ),
    );

    let wire_elements = doc.query_selector_all(".penguin-wire")?;

    for i in 0..wire_elements.length() {
        let Some(wire_el) = wire_elements.item(i) else {
            continue;
        };

        let Some(path_el) = wire_el.dyn_ref::<web_sys::SvgPathElement>() else {
            continue;
        };

        let Some(wire_id_str) = path_el.get_attribute("data-wire-id") else {
            continue;
        };

        let Ok(wire_id_val) = wire_id_str.parse::<u16>() else {
            continue;
        };

        let total_length = path_el.get_total_length();
        let sample_interval = 5.0;
        let num_samples = (total_length / sample_interval).ceil() as i32;

        let mut intersects = false;
        for j in 0..=num_samples {
            let distance = (j as f32) * sample_interval;
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
            selected.wires.insert(WireId(wire_id_val));
        }
    }

    Ok(())
}

pub fn update_wires(
    graph: &mut Store<GraphState>,
    viewport: &Signal<ViewportState>,
) -> Result<(), JsValue> {
    let zoom = viewport.peek().zoom;

    let node_lut = build_node_lookup()?;
    let pin_lut = build_pin_lookup()?;

    let mut g = graph.write();
    let wire_ids: Vec<_> = g.wires.keys().copied().collect();

    for wire_id in wire_ids {
        let (from_pos, to_pos, stroke, stroke_width) = {
            let wire = g.wires.get(&wire_id).unwrap();

            let Some(from_node) = g.nodes.get(&wire.from_pin.node_id) else {
                continue;
            };
            let Some(to_node) = g.nodes.get(&wire.to_pin.node_id) else {
                continue;
            };
            let Some(from_pin) = from_node.outputs.get(&wire.from_pin.pin_id) else {
                continue;
            };

            let Some(from_node_el) = node_lut.get(&wire.from_pin.node_id) else {
                continue;
            };
            let Some(to_node_el) = node_lut.get(&wire.to_pin.node_id) else {
                continue;
            };

            let from_pin_key = (wire.from_pin.node_id, wire.from_pin.pin_id.0.clone(), true);
            let to_pin_key = (wire.to_pin.node_id, wire.to_pin.pin_id.0.clone(), false);

            let Some(from_pin_el) = pin_lut.get(&from_pin_key) else {
                continue;
            };
            let Some(to_pin_el) = pin_lut.get(&to_pin_key) else {
                continue;
            };

            let from_pos =
                calculate_pin_world_position(from_node_el, from_pin_el, from_node.position, zoom);
            let to_pos =
                calculate_pin_world_position(to_node_el, to_pin_el, to_node.position, zoom);

            let (stroke, stroke_width) = get_pin_stroke(from_pin);

            (from_pos, to_pos, stroke, stroke_width)
        };

        let wire = g.wires.get_mut(&wire_id).unwrap();
        wire.from_pos = from_pos;
        wire.to_pos = to_pos;
        wire.stroke = stroke;
        wire.stroke_width = stroke_width;
    }

    Ok(())
}

fn build_node_lookup() -> Result<HashMap<NodeId, web_sys::HtmlElement>, JsValue> {
    let doc = web_sys::window().unwrap().document().unwrap();
    let mut node_lut = HashMap::new();

    let node_els = doc.query_selector_all(".penguin-node")?;
    for i in 0..node_els.length() {
        let Some(node_el) = node_els.item(i) else {
            continue;
        };
        let Some(html_el) = node_el.dyn_ref::<web_sys::HtmlElement>() else {
            continue;
        };
        let Some(node_id_str) = html_el.get_attribute("data-node-id") else {
            continue;
        };
        let Ok(node_id_val) = node_id_str.parse::<u16>() else {
            continue;
        };

        node_lut.insert(NodeId(node_id_val), html_el.clone());
    }

    Ok(node_lut)
}

fn build_pin_lookup() -> Result<HashMap<(NodeId, String, bool), web_sys::HtmlElement>, JsValue> {
    let doc = web_sys::window().unwrap().document().unwrap();
    let mut pin_lut = HashMap::new();

    let pin_els = doc.query_selector_all(".penguin-pin-hitbox")?;
    for i in 0..pin_els.length() {
        let Some(pin_el) = pin_els.item(i) else {
            continue;
        };
        let Some(html_el) = pin_el.dyn_ref::<web_sys::HtmlElement>() else {
            continue;
        };

        let Some(node_id_str) = html_el.get_attribute("data-node-id") else {
            continue;
        };
        let Some(pin_id_str) = html_el.get_attribute("data-pin-id") else {
            continue;
        };
        let Some(is_output_str) = html_el.get_attribute("data-is-output") else {
            continue;
        };

        let Ok(node_id_val) = node_id_str.parse::<u16>() else {
            continue;
        };
        let is_output = is_output_str == "true";

        pin_lut.insert(
            (NodeId(node_id_val), pin_id_str, is_output),
            html_el.clone(),
        );
    }

    Ok(pin_lut)
}

fn calculate_pin_world_position(
    node_el: &web_sys::HtmlElement,
    pin_el: &web_sys::HtmlElement,
    node_world_pos: Point2D<f64>,
    zoom: f64,
) -> Point2D<f64> {
    let node_rect = node_el.get_bounding_client_rect();
    let pin_rect = pin_el.get_bounding_client_rect();

    let pin_center_x = pin_rect.left() + pin_rect.width() / 2.0;
    let pin_center_y = pin_rect.top() + pin_rect.height() / 2.0;

    let offset_x = pin_center_x - node_rect.left();
    let offset_y = pin_center_y - node_rect.top();

    let world_offset_x = offset_x / zoom;
    let world_offset_y = offset_y / zoom;

    Point2D::new(
        node_world_pos.x + world_offset_x,
        node_world_pos.y + world_offset_y,
    )
}

fn get_pin_stroke(pin: &Pin) -> (String, u8) {
    match &pin.typ {
        PinType::Flow => ("white".to_string(), 4),
        PinType::Value { color, .. } => (color.clone(), 2),
    }
}
