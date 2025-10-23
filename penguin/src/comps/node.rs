use std::collections::HashSet;

use crate::{
    comps::PinComponent,
    state::{GraphState, InteractionMode, Selected, SelectedStoreExt, ViewportState},
    types::*,
};
use dioxus::{html::input_data::MouseButton, prelude::*};

#[component]
pub fn NodeComponent(
    graph: Store<GraphState>,
    id: NodeId,
    node: Store<Node>,
    selected: Store<Selected>,
    interaction: Signal<InteractionMode>,
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
            selected.nodes().write().insert(id);
        }
        // replace selection
        else {
            let mut s = HashSet::new();
            s.insert(id);
            selected.nodes().set(s);
            selected.wires().set(HashSet::default());
        }

        interaction.set(InteractionMode::Dragging);
    };

    let oncontextmenu = move |e: Event<MouseData>| {
        e.stop_propagation();
        e.prevent_default();
        // TODO edit node
    };

    rsx! {
        div {
            class: if selected.nodes().read().contains(&id) { "penguin-node selected" } else { "penguin-node" },
            "data-node-id": id.0,
            transform: "translate({node.position()().x}px, {node.position()().y}px)",
            onmousedown,
            oncontextmenu,

            div {
                class: "penguin-node-title",
                "{node.title()}"
            }

            div {
                class: "penguin-node-inputs",
                for (pin_id, pin) in node.inputs().iter() {
                    PinComponent {
                        graph,
                        node_id: id,
                        pin_id,
                        pin,
                        is_output: false,
                        interaction,
                        viewport,
                    }
                }
            }

            div {
                class: "penguin-node-outputs",
                for (pin_id, pin) in node.outputs().iter() {
                    PinComponent {
                        graph,
                        node_id: id,
                        pin_id,
                        pin,
                        is_output: true,
                        interaction,
                        viewport,
                    }
                }
            }
        }
    }
}
