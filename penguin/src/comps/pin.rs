use crate::{
    comps::PinInput,
    ffi,
    graph::{Graph, GraphStoreExt},
    state::WiringData,
    types::*,
};
use dioxus::{html::input_data::MouseButton, prelude::*};
use igloo_interface::PinType;

#[component]
pub fn PinComponent(
    graph: Store<Graph>,
    node: Store<Node>,
    node_id: NodeID,
    pin_ref: PinRef,
    pin_type: PinType,
    pin_name: String,
    is_output: bool,
    wiring_state: Signal<Option<WiringData>>,
) -> Element {
    // start wiring
    let onmousedown = move |e: Event<MouseData>| {
        if e.data().trigger_button() != Some(MouseButton::Primary) {
            return;
        }
        e.prevent_default();
        e.stop_propagation();

        wiring_state.set(Some(WiringData {
            start_node: node_id,
            start_pin: pin_ref,
            is_output,
            wire_type: pin_type,
        }));

        ffi::startWiring(
            node_id.0,
            pin_ref.defn_index,
            pin_ref.phantom_instance,
            is_output,
        );
    };

    let onmouseup = move |e: Event<MouseData>| {
        if e.data().trigger_button() != Some(MouseButton::Primary) {
            return;
        }
        e.prevent_default();
        e.stop_propagation();

        let Some(ws) = wiring_state.write().take() else {
            return;
        };

        ffi::changeInteractionMode("idle".to_string());

        graph
            .write()
            .complete_wire(ws, node_id, pin_ref, pin_type, is_output);
    };

    let is_connected = use_memo(move || {
        graph
            .wires()
            .read()
            .values()
            .any(|wire| wire.connects_to(node_id, &pin_ref, is_output))
    });

    rsx! {
        div {
            class: if is_output { "penguin-pin-wrapper output" } else { "penguin-pin-wrapper input" },

            div {
                class: if let Some(ws) = wiring_state() {
                    if ws.is_valid_end(node_id, pin_type, is_output) {
                        if ws.wire_type == pin_type {
                            "penguin-pin-hitbox valid-target"
                        } else {
                            "penguin-pin-hitbox castable-target"
                        }
                    } else {
                        "penguin-pin-hitbox invalid-target"
                    }
                } else {
                    "penguin-pin-hitbox"
                },
                "data-pin-defn": pin_ref.defn_index,
                "data-pin-phantom": pin_ref.phantom_instance,
                "data-node-id": node_id.0,
                "data-is-output": "{is_output}",
                onmousedown,
                onmouseup,

                match pin_type {
                    PinType::Flow => rsx! {
                        svg {
                            class: "penguin-pin flow",
                            width: "16",
                            height: "16",
                            view_box: "0 0 16 16",

                            polygon {
                                points: "1,1 12,1 15,8 12,15 1,15",
                                fill: if is_connected() { "white" } else { "transparent" },
                                stroke: "white",
                                stroke_width: "2",
                            }
                        }
                    },
                    PinType::Value(vt) => rsx! {
                        div {
                            class: "penguin-pin value",
                            border_color: vt.color(),
                            background_color: if is_connected() { vt.color() } else { "transparent" },
                        }
                    }
                }
            }

            if !pin_name.is_empty() {
                span { class: "penguin-pin-name", "{pin_name}" }
            }

            if !is_output && !is_connected() {
                if let PinType::Value(vt) = pin_type {
                    PinInput {
                        node: node,
                        pin_ref,
                        value_type: vt,
                    }
                }
            }
        }
    }
}
