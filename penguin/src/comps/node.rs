use crate::{
    comps::PinComponent, context::ContextMenuState, ffi, graph::Graph, state::WiringData, types::*,
};
use dioxus::prelude::*;

#[component]
pub fn NodeComponent(
    graph: Store<Graph>,
    id: NodeID,
    node: Store<Node>,
    wiring_state: Signal<Option<WiringData>>,
    context_menu_state: Signal<ContextMenuState>,
) -> Element {
    rsx! {
        div {
            class: "penguin-node",
            "data-node-id": id.0,
            transform: "translate({node.pos().read().x}px, {node.pos().read().y}px)",
            oncontextmenu: move |e: Event<MouseData>| {
                e.prevent_default();
                e.stop_propagation();
                context_menu_state.write().open_node(e.client_coordinates(), id);
            },
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
