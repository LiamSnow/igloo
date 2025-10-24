use crate::{
    comps::PinComponent,
    ffi,
    state::{GraphState, WiringData},
    types::*,
};
use dioxus::prelude::*;

#[component]
pub fn NodeComponent(
    graph: Store<GraphState>,
    id: NodeId,
    node: Store<Node>,
    wiring_state: Signal<Option<WiringData>>,
) -> Element {
    rsx! {
        div {
            class: "penguin-node",
            "data-node-id": id.0,
            transform: "translate({node.pos().read().x}px, {node.pos().read().y}px)",
            onmount: move |_| {
                ffi::rerender();
            },

            div {
                class: "penguin-node-title",
                "{node.title()}"
            }

            div {
                class: "penguin-node-inputs",
                for (pin_id, pin_type) in node.inputs()() {
                    PinComponent {
                        graph,
                        node_id: id,
                        pin_id,
                        pin_type,
                        is_output: false,
                        wiring_state,
                    }
                }
            }

            div {
                class: "penguin-node-outputs",
                for (pin_id, pin_type) in node.outputs()() {
                    PinComponent {
                        graph,
                        node_id: id,
                        pin_id,
                        pin_type,
                        is_output: true,
                        wiring_state,
                    }
                }
            }
        }
    }
}
