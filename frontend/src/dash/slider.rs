use crate::ws::{send_msg, CURRENT_DASHBOARD, ELEMENT_VALUES};
use dioxus::prelude::*;
use igloo_interface::{dash::SliderElement, Component, SetQuery};

#[component]
pub(crate) fn Slider(el: SliderElement) -> Element {
    let watch_id = el.watch_id.unwrap();
    let filter = el.binding.filter.clone();
    let comp_type = el.binding.comp_type;
    let dash = CURRENT_DASHBOARD.read();
    let targets = &dash.as_ref().unwrap().targets;
    let target = targets.get(&el.binding.target).unwrap().clone();

    let step = match el.step {
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
                min: el.min,
                max: el.max,
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
