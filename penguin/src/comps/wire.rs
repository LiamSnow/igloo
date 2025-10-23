use std::collections::HashSet;

use crate::{
    state::{Selected, SelectedStoreExt},
    types::*,
};
use dioxus::{html::input_data::MouseButton, prelude::*};

#[component]
pub fn WireComponent(
    /// Some if placed, None if placing/temp
    id: Option<WireId>,
    wire: Store<Wire>,
    selected: Store<Selected>,
) -> Element {
    let onmousedown = move |e: Event<MouseData>| {
        if e.trigger_button() != Some(MouseButton::Primary) {
            return;
        }

        let Some(id) = id else {
            return;
        };

        e.stop_propagation();
        e.prevent_default();

        // append selection
        let mods = e.data().modifiers();
        if mods.shift() || mods.ctrl() {
            selected.wires().write().insert(id);
        }
        // replace selection
        else {
            let mut s = HashSet::new();
            s.insert(id);
            selected.wires().set(s);
            selected.nodes().set(HashSet::default());
        }
    };

    let oncontextmenu = move |e: Event<MouseData>| {
        if id.is_some() {
            return;
        }

        e.stop_propagation();
        e.prevent_default();
        // TODO open edit context menu (later on)
    };

    let d = use_memo(move || {
        let dist = (wire.to_pos().read().x - wire.from_pos().read().x).abs();
        let coff = (dist / 2.0).min(100.0);

        format!(
            "M {} {} C {} {}, {} {}, {} {}",
            wire.from_pos().read().x,
            wire.from_pos().read().y,
            wire.from_pos().read().x + coff,
            wire.from_pos().read().y,
            wire.to_pos().read().x - coff,
            wire.to_pos().read().y,
            wire.to_pos().read().x,
            wire.to_pos().read().y
        )
    });

    let class = use_memo(move || {
        if let Some(id) = id {
            if selected().wires.contains(&id) {
                return "penguin-wire selected";
            }
        }
        "penguin-wire"
    });

    rsx! {
        path {
            class,
            "data-wire-id": if let Some(id) = id { id.0 },
            fill: "none",
            stroke: wire.stroke(),
            stroke_width: wire.stroke_width(),
            stroke_dasharray: if id.is_none() { "5, 5" } else { "" },
            onmousedown,
            oncontextmenu,
            d,
        }
    }
}
