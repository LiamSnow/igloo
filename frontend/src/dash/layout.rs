use dioxus::prelude::*;
use igloo_interface::dash::{HStackElement, VStackElement};

use super::DashComponent;

#[component]
pub(crate) fn HStack(el: HStackElement) -> Element {
    let style = if el.scroll {
        "overflow-x: auto"
    } else {
        "overflow-x: visible; flex-wrap: wrap;"
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: row; justify-content: {el.justify}; align-items: {el.align}; {style}",
            {el.children.iter().map(|child| rsx! {
                DashComponent { el: child.clone() }
            })}
        }
    }
}

#[component]
pub(crate) fn VStack(el: VStackElement) -> Element {
    let style = if el.scroll {
        "overflow-y: auto"
    } else {
        "overflow-y: visible; flex-wrap: wrap;"
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; justify-content: {el.justify}; align-items: {el.align}; {style}",
            {el.children.iter().map(|child| rsx! {
                DashComponent { el: child.clone() }
            })}
        }
    }
}
