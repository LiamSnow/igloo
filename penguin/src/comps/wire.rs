use crate::{context::ContextMenuState, ffi, types::*};
use dioxus::prelude::*;

#[component]
pub fn WireComponent(
    id: WireID,
    wire: Wire,
    context_menu_state: Signal<ContextMenuState>,
) -> Element {
    rsx! {
        path {
            class: "penguin-wire",
            "data-wire-id": id.0,
            "data-from-node": wire.from_node.0,
            "data-from-pin-defn": wire.from_pin.defn_index,
            "data-from-pin-phantom": wire.from_pin.phantom_instance,
            "data-to-node": wire.to_node.0,
            "data-to-pin-defn": wire.to_pin.defn_index,
            "data-to-pin-phantom": wire.to_pin.phantom_instance,
            fill: "none",
            stroke: wire.r#type.stroke(),
            stroke_width: wire.r#type.stroke_width(),
            oncontextmenu: move |e: Event<MouseData>| {
                e.prevent_default();
                e.stop_propagation();
                context_menu_state.write().open_wire(e.client_coordinates(), id);
            },
            onmount: move |_| {
                ffi::rerender();
            },
        }
    }
}
