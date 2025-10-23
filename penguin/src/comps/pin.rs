use crate::{
    state::{GraphState, GraphStateStoreExt, InteractionMode, ViewportState},
    types::*,
    update_wires,
};
use dioxus::{html::input_data::MouseButton, prelude::*};

#[component]
pub fn PinComponent(
    graph: Store<GraphState>,
    node_id: NodeId,
    pin_id: PinId,
    pin: Store<Pin>,
    is_output: bool,
    interaction: Signal<InteractionMode>,
    viewport: Signal<ViewportState>,
) -> Element {
    let pin_id_copy = pin_id.clone();
    let onmousedown = move |e: Event<MouseData>| {
        if e.trigger_button() != Some(MouseButton::Primary) {
            return;
        }

        e.stop_propagation();
        e.prevent_default();

        interaction.set(InteractionMode::Wiring {
            start: PinRef {
                node_id,
                pin_id: pin_id_copy.clone(),
            },
            is_output,
            typ: pin.peek().typ.clone(),
        });
    };

    let pin_id_copy = pin_id.clone();
    let onmouseup = move |e: Event<MouseData>| {
        if e.trigger_button() != Some(MouseButton::Primary) {
            return;
        }

        e.stop_propagation();
        e.prevent_default();

        let m = interaction.peek().clone();
        if let InteractionMode::Wiring {
            start,
            is_output: start_is_output,
            typ,
        } = m
        {
            interaction.set(InteractionMode::Idle);

            if is_output == start_is_output || typ != pin.peek().typ.clone() {
                return;
            }

            let me = PinRef {
                node_id,
                pin_id: pin_id_copy.clone(),
            };
            let mut w = graph.wires();
            let mut wires = w.write();
            let (from_pin, to_pin) = if is_output { (me, start) } else { (start, me) };

            let mut id = 0;
            loop {
                if !wires.contains_key(&WireId(id)) {
                    break;
                }
                id += 1;
            }

            wires.insert(
                WireId(id),
                Wire {
                    from_pin,
                    to_pin,
                    ..Default::default()
                },
            );

            update_wires(&mut graph, &viewport);
        }
    };

    let cursor_class = use_memo(move || {
        let m = interaction();
        match m {
            InteractionMode::Wiring {
                is_output: start_is_output,
                typ,
                ..
            } => Some(if is_output == start_is_output || typ != pin.read().typ {
                "invalid-target"
            } else {
                "valid-target"
            }),
            _ => None,
        }
    });

    // this ought to be horribly slow,
    // probably need restructuring to be able to
    // fix it
    let pin_id_copy = pin_id.clone();
    let is_connected = use_memo(move || {
        for (_, wire) in graph.wires().read().iter() {
            let comp = if is_output {
                &wire.from_pin
            } else {
                &wire.to_pin
            };
            if comp.node_id == node_id && comp.pin_id == pin_id_copy {
                return true;
            }
        }
        false
    });

    rsx! {
        div {
            class: if let Some(cursor) = cursor_class() {
                format!("penguin-pin-hitbox {cursor}")
            } else {
                "penguin-pin-hitbox".to_string()
            },
            "data-pin-id": pin_id.0,
            "data-node-id": node_id.0,
            "data-is-output": "{is_output}",
            onmousedown,
            onmouseup,

            match &pin.read().typ {
                PinType::Flow => rsx! {
                    svg {
                        class: "penguin-pin flow",
                        width: "16",
                        height: "16",
                        view_box: "0 0 16 16",
                        "x-pin": "{pin_id.0}",

                        polygon {
                            points: "1,1 12,1 15,8 12,15 1,15",
                            fill: if is_connected() { "white" } else { "transparent" },
                            stroke: "white",
                            stroke_width: "2",
                        }
                    }
                },
                PinType::Value { color, .. } => rsx! {
                    div {
                        class: "penguin-pin value",
                        "x-pin": "{pin_id.0}",
                        border_color: color.to_string(),
                        background_color: if is_connected() { color.to_string() } else { "transparent" },
                    }
                }
            }
        }
    }
}
