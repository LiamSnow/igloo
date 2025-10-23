use crate::{
    state::{GraphState, GraphStateStoreExt, ViewportState},
    sys::MOUSE_POSITION,
    types::*,
};
use dioxus::prelude::*;
use euclid::default::Vector2D;

#[component]
pub fn TempWireComponent(
    graph: Store<GraphState>,
    start: PinRef,
    is_output: bool,
    typ: PinType,
    viewport: Signal<ViewportState>,
) -> Element {
    let mut d = use_signal(|| None);

    use_effect(move || {
        let zoom = viewport().zoom;
        let pan = viewport().pan;
        let mouse = MOUSE_POSITION();

        let n = graph.nodes();
        let nodes = n.read();
        let Some(start_node) = nodes.get(&start.node_id) else {
            return;
        };

        let start_node_world = start_node.position;

        let start_pin = if is_output {
            start_node.outputs.get(&start.pin_id)
        } else {
            start_node.inputs.get(&start.pin_id)
        };

        let Some(start_pin) = start_pin else {
            return;
        };

        let Some(start_node_el) = start_node.el.clone() else {
            return;
        };
        let Some(start_pin_el) = start_pin.el.clone() else {
            return;
        };

        spawn(async move {
            let Ok(start_node_rect) = start_node_el.get_client_rect().await else {
                return;
            };
            let Ok(start_pin_rect) = start_pin_el.get_client_rect().await else {
                return;
            };

            let pin_offset = start_pin_rect.center() - start_node_rect.origin;
            let pin_world_offset = Vector2D::new(pin_offset.x, pin_offset.y) / zoom;

            let start_world_x = start_node_world.x + pin_world_offset.x;
            let start_world_y = start_node_world.y + pin_world_offset.y;

            let doc = web_sys::window().unwrap().document().unwrap();
            let Some(penguin_el) = doc.query_selector("#penguin").ok().flatten() else {
                return;
            };
            let penguin_rect = penguin_el.get_bounding_client_rect();

            let mouse_world_x = (mouse.x - penguin_rect.left() - pan.x) / zoom;
            let mouse_world_y = (mouse.y - penguin_rect.top() - pan.y) / zoom;

            let (from_x, from_y, to_x, to_y) = if is_output {
                (start_world_x, start_world_y, mouse_world_x, mouse_world_y)
            } else {
                (mouse_world_x, mouse_world_y, start_world_x, start_world_y)
            };

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

    let (stroke, stroke_width) = match typ {
        PinType::Flow => ("white".to_string(), 4),
        PinType::Value { color, .. } => (color.clone(), 2),
    };

    rsx! {
        if let Some(path) = d() {
            path {
                class: "penguin-wire temp",
                fill: "none",
                stroke,
                stroke_width,
                stroke_dasharray: "5, 5",
                d: path,
            }
        }
    }
}
