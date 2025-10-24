use crate::{ffi, types::*};
use dioxus::prelude::*;

#[component]
pub fn WireComponent(id: WireId, wire: Wire) -> Element {
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
            onmount: move |_| {
                ffi::rerender();
            },
        }
    }
}
