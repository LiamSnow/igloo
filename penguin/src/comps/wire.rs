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
            "data-from-pin": wire.from_pin.0,
            "data-to-node": wire.to_node.0,
            "data-to-pin": wire.to_pin.0,
            fill: "none",
            stroke: wire.wire_type.stroke(),
            stroke_width: wire.wire_type.stroke_width(),
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
