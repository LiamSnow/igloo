use std::collections::HashMap;
use std::sync::Arc;

use crate::ws::{send_msg, ELEMENT_VALUES};
use dioxus::prelude::*;
use igloo_interface::{dash::SliderElement, Component, QueryTarget, SetQuery};

#[derive(PartialEq, Props, Clone)]
pub(crate) struct SliderProps {
    pub element: SliderElement,
    pub targets: Arc<HashMap<String, QueryTarget>>,
}

#[component]
pub(crate) fn Slider(props: SliderProps) -> Element {
    let watch_id = props.element.watch_id.unwrap();
    let filter = props.element.binding.filter.clone();
    let comp_type = props.element.binding.comp_type;
    let target = props
        .targets
        .get(&props.element.binding.target)
        .unwrap()
        .clone();

    let step = match props.element.step {
        Some(s) => s.to_string(),
        None => "any".to_string(),
    };

    let cur_value = ELEMENT_VALUES
        .read()
        .get(&watch_id)
        .and_then(|c| c.inner_string());

    rsx! {
        div {
            class: "slider-box {comp_type.kebab_name()}",
            input {
                class: "slider {comp_type.kebab_name()}",
                r#type: "range",
                min: props.element.min,
                max: props.element.max,
                step: step,
                value: cur_value,
                onchange: move |event| {
                    send_msg(SetQuery {
                        filter: filter.clone(),
                        target: target.clone(),
                        values: vec![
                            Component::from_string(
                                comp_type,
                                &event.value()
                            ).unwrap() // FIXME unwrap
                        ]
                    }.into());
                }
            }
        }
    }
}
