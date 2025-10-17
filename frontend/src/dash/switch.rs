use crate::ws::{send_msg, CURRENT_DASHBOARD, ELEMENT_VALUES};
use dioxus::prelude::*;
use igloo_interface::{dash::SwitchElement, Component, SetQuery};

#[component]
pub(crate) fn Switch(el: SwitchElement) -> Element {
    let watch_id = el.watch_id.unwrap();
    let filter = el.binding.filter.clone();
    let dash = CURRENT_DASHBOARD.read();
    let targets = &dash.as_ref().unwrap().targets;
    let target = targets.get(&el.binding.target).unwrap().clone();

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
            class: "switch-box {el.size}",
            div {
                class: "switch {state_class} {el.size}",
                onclick: move |_| {
                    send_msg(SetQuery {
                        filter: filter.clone(),
                        target: target.clone(),
                        values: vec![
                            Component::from_string(
                                el.binding.comp_type,
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
