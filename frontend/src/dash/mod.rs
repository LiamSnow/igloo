use std::collections::HashMap;

use crate::ws::{send_msg, CURRENT_DASHBOARD, ELEMENT_VALUES};
use dioxus::prelude::*;
use igloo_interface::{dash::DashElement, Component, QueryTarget, SetQuery};

#[component]
pub fn Dash(id: u16) -> Element {
    match CURRENT_DASHBOARD.read().as_ref() {
        Some(dash) => {
            rsx! {
                div { class: "dashboard",
                    h1 { "Dashboard {id}" }
                    DashComponent {
                        el: dash.child.clone(),
                        targets: dash.targets.clone()
                    }
                }
            }
        }
        None => {
            rsx! {
                div { class: "dashboard" }
            }
        }
    }
}

#[derive(PartialEq, Props, Clone)]
struct DashComponentProps {
    el: DashElement,
    targets: HashMap<String, QueryTarget>,
}

#[component]
fn DashComponent(props: DashComponentProps) -> Element {
    match props.el {
        DashElement::HStack(e) => {
            let overflow = if e.scroll { "auto" } else { "visible" };
            rsx! {
                div {
                    style: "display: flex; flex-direction: row; justify-content: {e.justify}; align-items: {e.align}; overflow-x: {overflow};",
                    {e.children.iter().map(|child| rsx! {
                        DashComponent {
                            el: child.clone(),
                            targets: props.targets.clone()
                        }
                    })}
                }
            }
        }
        DashElement::VStack(e) => {
            let overflow = if e.scroll { "auto" } else { "visible" };
            rsx! {
                div {
                    style: "display: flex; flex-direction: column; justify-content: {e.justify}; align-items: {e.align}; overflow-y: {overflow};",
                    {e.children.iter().map(|child| rsx! {
                        DashComponent {
                            el: child.clone(),
                            targets: props.targets.clone()
                        }
                    })}
                }
            }
        }
        DashElement::Slider(e) => {
            let min_val = e.min.as_ref().and_then(|c| match c {
                Component::Int(v) => Some(v.to_string()),
                Component::Float(v) => Some(v.to_string()),
                _ => None,
            });

            let max_val = e.max.as_ref().and_then(|c| match c {
                Component::Int(v) => Some(v.to_string()),
                Component::Float(v) => Some(v.to_string()),
                _ => None,
            });

            let step_val = match e.step {
                Some(Component::Int(v)) => v.to_string(),
                Some(Component::Float(v)) => v.to_string(),
                _ => "any".to_string(),
            };

            let watch_id = e.watch_id.unwrap();
            let filter = e.binding.filter.clone();
            let comp_type = e.binding.comp_type;
            let target = props.targets.get(&e.binding.target).unwrap().clone();

            let current_value = ELEMENT_VALUES
                .read()
                .get(&watch_id)
                .and_then(|c| c.inner_string());

            rsx! {
                input {
                    r#type: "range",
                    min: min_val,
                    max: max_val,
                    step: step_val,
                    value: current_value,
                    oninput: move |event| {
                        let value_str = event.value();
                        let value = Component::from_string(comp_type, &value_str).unwrap(); // FIXME unwrap
                        send_msg(SetQuery {
                            filter: filter.clone(),
                            target: target.clone(),
                            values: vec![value]
                        }.into());
                    }
                }
            }
        }
        DashElement::ColorPicker(e) => {
            rsx! {
                input {
                    r#type: "color",
                }
            }
        }
        _ => rsx! { div { "Unsupported element" } },
    }
}
