use std::collections::HashMap;
use std::sync::Arc;

use crate::ws::{send_msg, ELEMENT_VALUES};
use dioxus::prelude::*;
use igloo_interface::{dash::SwitchElement, Component, QueryTarget, SetQuery};

#[derive(PartialEq, Props, Clone)]
pub(crate) struct SwitchProps {
    pub element: SwitchElement,
    pub targets: Arc<HashMap<String, QueryTarget>>,
}

#[component]
pub(crate) fn Switch(props: SwitchProps) -> Element {
    let watch_id = props.element.watch_id.unwrap();
    let filter = props.element.binding.filter.clone();
    let target = props
        .targets
        .get(&props.element.binding.target)
        .unwrap()
        .clone();

    let is_on = use_memo(move || {
        ELEMENT_VALUES
            .read()
            .get(&watch_id)
            .and_then(|comp| comp.inner_string())
            .and_then(|s| s.parse().ok())
            .unwrap_or(false)
    });

    let state_class = if is_on() { "switch-on" } else { "switch-off" };

    rsx! {
        div {
            class: "switch-box {props.element.size}",
            div {
                class: "switch {state_class} {props.element.size}",
                onclick: move |_| {
                    send_msg(SetQuery {
                        filter: filter.clone(),
                        target: target.clone(),
                        values: vec![
                            Component::from_string(
                                props.element.binding.comp_type,
                                &(!is_on()).to_string()
                            )
                            .unwrap() // FIXME unwrap
                        ]
                    }.into());
                },
                div {
                    class: "switch-thumb {state_class}"
                }
            }
        }
    }
}
