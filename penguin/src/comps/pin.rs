use crate::{
    ffi,
    graph::{Graph, GraphStoreExt},
    state::WiringData,
    types::*,
};
use dioxus::{html::input_data::MouseButton, prelude::*};

#[component]
pub fn PinComponent(
    graph: Store<Graph>,
    node_id: NodeID,
    pin_id: PinID,
    pin_type: PinType,
    is_output: bool,
    wiring_state: Signal<Option<WiringData>>,
) -> Element {
    let pin_id_1 = pin_id.clone();
    let pin_id_2 = pin_id.clone();
    let pin_id_3 = pin_id.clone();
    let pin_type_1 = pin_type.clone();
    let pin_type_2 = pin_type.clone();
    let pin_type_3 = pin_type.clone();

    // start wiring
    let onmousedown = move |e: Event<MouseData>| {
        if e.data().trigger_button() != Some(MouseButton::Primary) {
            return;
        }
        e.prevent_default();
        e.stop_propagation();

        wiring_state.set(Some(WiringData {
            start_node: node_id,
            start_pin: pin_id_1.clone(),
            is_output,
            wire_type: pin_type_1.clone(),
        }));

        ffi::startWiring(node_id.0, &pin_id_1.0, is_output);
    };

    // complete wiring/place wire
    let onmouseup = move |e: Event<MouseData>| {
        if e.data().trigger_button() != Some(MouseButton::Primary) {
            return;
        }
        e.prevent_default();
        e.stop_propagation();

        let Some(ws) = wiring_state.write().take() else {
            return;
        };

        ffi::stopWiring();

        if ws.is_output == is_output || ws.wire_type != pin_type_2 {
            return;
        }

        let (from_node, from_pin, to_node, to_pin) = if ws.is_output {
            (
                ws.start_node,
                ws.start_pin.clone(),
                node_id,
                pin_id_2.clone(),
            )
        } else {
            (
                node_id,
                pin_id_2.clone(),
                ws.start_node,
                ws.start_pin.clone(),
            )
        };

        let mut g = graph.write();
        let next_id = g.wires.keys().map(|id| id.0).max().unwrap_or(0) + 1;

        g.wires.insert(
            WireID(next_id),
            Wire {
                from_node,
                from_pin,
                to_node,
                to_pin,
                wire_type: ws.wire_type,
            },
        );
    };

    // FIXME this is slow but not casuing issues yet so ill fix later maybe
    let is_connected = use_memo(move || {
        for (_, wire) in graph.wires().read().iter() {
            let other_node_id = if is_output {
                wire.from_node
            } else {
                wire.to_node
            };
            let other_pin_id = if is_output {
                wire.from_pin.clone()
            } else {
                wire.to_pin.clone()
            };
            if other_node_id == node_id && other_pin_id == pin_id_3 {
                return true;
            }
        }
        false
    });

    rsx! {
        div {
            class: if let Some(ws) = wiring_state() {
                if ws.is_output == is_output || ws.wire_type != pin_type_3 {
                    "penguin-pin-hitbox invalid-target"
                } else {
                    "penguin-pin-hitbox valid-target"
                }
            } else {
                "penguin-pin-hitbox"
            },
            "data-pin-id": pin_id.0,
            "data-node-id": node_id.0,
            "data-is-output": "{is_output}",
            onmousedown,
            onmouseup,
            onmount: move |_| {
                ffi::rerender();
            },

            match pin_type {
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
