use crate::{
    comps::{NodeInput, PinComponent},
    context::ContextMenuState,
    ffi,
    graph::{Graph, GraphStoreExt},
    state::WiringData,
    types::*,
};
use dioxus::prelude::*;
use igloo_interface::{NodeConfig, NodeStyle, PenguinRegistry, PinDefnType};
use std::collections::HashSet;

#[component]
pub fn NodeComponent(
    graph: Store<Graph>,
    id: NodeID,
    node: Store<Node>,
    connectivity: Memo<HashSet<(NodeID, PinRef, bool)>>,
    wiring_state: Signal<Option<WiringData>>,
    context_menu_state: Signal<ContextMenuState>,
) -> Element {
    let registry = use_context::<PenguinRegistry>();

    let defn_memo = use_memo(move || {
        let node_ref = node.defn_ref()();
        registry
            .get_defn(&node_ref.library, &node_ref.name)
            .cloned()
    });

    let inputs = use_memo(move || {
        defn_memo()
            .map(|defn| node.read().expand_inputs(&defn))
            .unwrap_or_default()
    });

    let outputs = use_memo(move || {
        defn_memo()
            .map(|defn| node.read().expand_outputs(&defn))
            .unwrap_or_default()
    });

    let phantom_cfgs = use_memo(move || {
        defn_memo()
            .map(|defn| {
                defn.cfg
                    .iter()
                    .filter_map(|cfg| {
                        if let NodeConfig::AddRemovePin(config) = cfg {
                            Some(config.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });

    let is_last_phantom_connected = move |phantom_id: u8, is_input: bool| {
        let count = node
            .read()
            .phantom_state
            .get(&phantom_id)
            .copied()
            .unwrap_or(2);
        if count == 0 {
            return false;
        }

        let defn = defn_memo();
        let defn = match defn {
            Some(d) => d,
            None => return false,
        };

        let pins = if is_input {
            &defn.inputs
        } else {
            &defn.outputs
        };
        let defn_index = pins.iter().position(|pin| {
            matches!(&pin.r#type, igloo_interface::PinDefnType::Phantom(id) if *id == phantom_id)
        });

        let defn_index = match defn_index {
            Some(i) => i,
            None => return false,
        };

        let last_pin_ref = PinRef::with_phantom_inst(defn_index as u32, count - 1);

        graph
            .wires()
            .read()
            .values()
            .any(|wire| wire.connects_to(id, &last_pin_ref, !is_input))
    };

    rsx! {
        div {
            class: "penguin-node",
            "data-ref": "{node.defn_ref()()}",
            "data-node-id": id.0,
            transform: "translate({node.x()}px, {node.y()}px)",
            oncontextmenu: move |e: Event<MouseData>| {
                e.prevent_default();
                e.stop_propagation();
                context_menu_state.write().open_node(e.client_coordinates(), id);
            },
            onmount: move |_| {
                ffi::rerender();
            },

            if let Some(defn) = defn_memo() {
                match &defn.style {
                    NodeStyle::Normal(icon) => rsx! {
                        div {
                            class: "penguin-node-title",
                            "{defn.title}"
                        }
                    },
                    NodeStyle::Background(bg) => rsx! {
                        div {
                            class: "penguin-node-bg",
                            "{bg}"
                        }
                    },
                    NodeStyle::None => rsx! {}
                }
            }

            NodeInput { node }

            div {
                class: "penguin-node-inputs",
                for (pin_ref, pin_type, pin_name) in inputs() {
                    PinComponent {
                        graph,
                        node,
                        node_id: id,
                        pin_ref,
                        pin_type,
                        pin_name,
                        is_output: false,
                        connectivity,
                        wiring_state,
                    }
                }
            }

            div {
                class: "penguin-node-outputs",
                for (pin_ref, pin_type, pin_name) in outputs() {
                    PinComponent {
                        graph,
                        node,
                        node_id: id,
                        pin_ref,
                        pin_type,
                        pin_name,
                        is_output: true,
                        connectivity,
                        wiring_state,
                    }
                }
            }

            if !phantom_cfgs().is_empty() {
                div {
                    class: "penguin-phantom-controls",
                    for config in phantom_cfgs() {
                        {
                            let phantom_id = config.phantom_id;
                            let min = config.min;
                            let max = config.max;
                            let current = node.phantom_state()()
                                .get(&phantom_id)
                                .copied()
                                .unwrap_or(min);

                            let is_input = defn_memo()
                                .map(|defn| {
                                    defn.inputs.iter().any(|pin| {
                                        matches!(&pin.r#type, PinDefnType::Phantom(id) if *id == phantom_id)
                                    })
                                })
                                .unwrap_or(false);

                            rsx! {
                                div {
                                    class: "penguin-phantom-control-group",

                                    button {
                                        class: "penguin-phantom-button",
                                        disabled: current <= min || is_last_phantom_connected(phantom_id, is_input),
                                        onclick: move |_| {
                                            if current > min {
                                                node.write().phantom_state.insert(phantom_id, current - 1);
                                                ffi::delayedRerender();
                                            }
                                        },
                                        "−"
                                    }

                                    button {
                                        class: "penguin-phantom-button",
                                        disabled: current >= max,
                                        onclick: move |_| {
                                            if current < max {
                                                node.write().phantom_state.insert(phantom_id, current + 1);
                                                ffi::delayedRerender();
                                            }
                                        },
                                        "+"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
