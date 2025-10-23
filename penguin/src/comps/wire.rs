use std::collections::HashSet;

use crate::{
    state::{GraphState, GraphStateStoreExt, Selected, SelectedStoreExt, ViewportState},
    types::*,
};
use dioxus::{html::input_data::MouseButton, prelude::*};
use euclid::default::Vector2D;

#[component]
pub fn WireComponent(
    graph: Store<GraphState>,
    id: WireId,
    wire: Store<Wire>,
    selected: Store<Selected>,
    viewport: Signal<ViewportState>,
) -> Element {
    let onmousedown = move |e: Event<MouseData>| {
        if e.trigger_button() != Some(MouseButton::Primary) {
            return;
        }

        e.stop_propagation();
        e.prevent_default();

        // append selection
        let mods = e.data().modifiers();
        if mods.shift() || mods.ctrl() {
            selected.wires().write().insert(id);
        }
        // replace selection
        else {
            let mut s = HashSet::new();
            s.insert(id);
            selected.wires().set(s);
            selected.nodes().set(HashSet::default());
        }
    };

    let oncontextmenu = move |e: Event<MouseData>| {
        e.stop_propagation();
        e.prevent_default();
        // TODO edit node
    };

    let smem = use_memo(move || {
        if let Some(from_node) = graph.nodes().read().get(&wire.read().from.node_id) {
            if let Some(from_pin) = from_node.outputs.get(&wire.read().from.pin_id) {
                return match &from_pin.typ {
                    PinType::Flow => ("white".to_string(), 4),
                    PinType::Value { color, .. } => (color.clone(), 2),
                };
            }
        }
        ("white".to_string(), 2)
    });

    let mut d = use_signal(|| None);

    use_effect(move || {
        let zoom = viewport().zoom;
        let n = graph.nodes();
        let nodes = n.read();
        let wire_data = wire.read();

        let from_node_id = wire_data.from.node_id;
        let to_node_id = wire_data.to.node_id;
        let from_pin_id = wire_data.from.pin_id.clone();
        let to_pin_id = wire_data.to.pin_id.clone();

        let Some(from_node) = nodes.get(&from_node_id) else {
            return;
        };
        let Some(to_node) = nodes.get(&to_node_id) else {
            return;
        };

        let Some(from_pin) = from_node.outputs.get(&from_pin_id) else {
            return;
        };
        let Some(to_pin) = to_node.inputs.get(&to_pin_id) else {
            return;
        };

        let Some(from_node_el) = from_node.el.clone() else {
            return;
        };
        let Some(to_node_el) = to_node.el.clone() else {
            return;
        };
        let Some(from_pin_el) = from_pin.el.clone() else {
            return;
        };
        let Some(to_pin_el) = to_pin.el.clone() else {
            return;
        };

        let from_node_world = from_node.position;
        let to_node_world = to_node.position;

        spawn(async move {
            let Ok(from_node_rect) = from_node_el.get_client_rect().await else {
                return;
            };
            let Ok(to_node_rect) = to_node_el.get_client_rect().await else {
                return;
            };
            let Ok(from_pin_rect) = from_pin_el.get_client_rect().await else {
                return;
            };
            let Ok(to_pin_rect) = to_pin_el.get_client_rect().await else {
                return;
            };

            let from_offset = from_pin_rect.center() - from_node_rect.origin;
            let to_offset = to_pin_rect.center() - to_node_rect.origin;

            let from_world_offset = Vector2D::new(from_offset.x, from_offset.y) / zoom;
            let to_world_offset = Vector2D::new(to_offset.x, to_offset.y) / zoom;

            let from_x = from_node_world.x + from_world_offset.x;
            let from_y = from_node_world.y + from_world_offset.y;

            let to_x = to_node_world.x + to_world_offset.x;
            let to_y = to_node_world.y + to_world_offset.y;

            let dist = (to_x - from_x).abs();
            let coff = (dist / 2.0).min(100.0);

            d.set(Some(format!(
                "M {} {} C {} {}, {} {}, {} {}",
                from_x,
                from_y,
                from_x + coff,
                from_y,
                to_x - coff,
                to_y,
                to_x,
                to_y
            )));
        });
    });

    rsx! {
        path {
            class: if selected().wires.contains(&id) { "penguin-wire selected" } else { "penguin-wire" },
            fill: "none",
            stroke: smem().0,
            stroke_width: smem().1,
            onmousedown,
            oncontextmenu,
            onmounted: move |e| {
                wire.el().set(Some(e.data()));
            },
            d: d(),
        }
    }
}
