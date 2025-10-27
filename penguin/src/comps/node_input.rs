use crate::{ffi, types::*};
use dioxus::prelude::*;
use igloo_interface::{Color, NodeConfig, PenguinRegistry, ValueData, ValueType};

#[component]
pub fn NodeInput(node: Store<Node>) -> Element {
    let registry = use_context::<PenguinRegistry>();

    let defn_memo = use_memo(move || {
        let node_ref = node.defn_ref()();
        registry
            .get_defn(&node_ref.library, &node_ref.name)
            .cloned()
    });

    let input_configs_memo = use_memo(move || {
        defn_memo()
            .map(|defn| {
                defn.cfg
                    .iter()
                    .filter_map(|cfg| {
                        if let NodeConfig::Input(config) = cfg {
                            Some(config.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });

    if input_configs_memo().is_empty() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "penguin-node-content",
            for config in input_configs_memo() {
                {
                    let input_id = config.id;
                    let value_type = config.r#type;

                    match value_type {
                        ValueType::Int => rsx! {
                            input {
                                class: "penguin-input",
                                r#type: "number",
                                step: "1",
                                value: node
                                    .value_inputs()
                                    .get(input_id)
                                    .and_then(|v| if let ValueData::Int(i) = v() {
                                        Some(i)
                                    } else {
                                        None
                                    })
                                    .unwrap_or(0),
                                oninput: move |evt| {
                                    if let Ok(val) = evt.value().parse::<i64>() {
                                        node.write().value_inputs.insert(input_id, ValueData::Int(val));
                                    }
                                }
                            }
                        },
                        ValueType::Real => rsx! {
                            input {
                                class: "penguin-input",
                                r#type: "number",
                                step: "any",
                                value: node
                                    .value_inputs()
                                    .get(input_id)
                                    .and_then(|v| if let ValueData::Real(i) = v() {
                                        Some(i)
                                    } else {
                                        None
                                    })
                                    .unwrap_or(0.),
                                oninput: move |evt| {
                                    if let Ok(val) = evt.value().parse::<f64>() {
                                        node.write().value_inputs.insert(input_id, ValueData::Real(val));
                                    }
                                }
                            }
                        },
                        ValueType::Text => rsx! {
                            textarea {
                                class: "penguin-input",
                                value: node
                                    .value_inputs()
                                    .get(input_id)
                                    .and_then(|v| if let ValueData::Text(i) = v() {
                                        Some(i)
                                    } else {
                                        None
                                    })
                                    .unwrap_or(String::default()),
                                oninput: move |evt| {
                                    node.write().value_inputs.insert(input_id, ValueData::Text(evt.value()));
                                },
                                onresize: move |_| {
                                    ffi::rerender();
                                },
                            }
                        },
                        ValueType::Bool => rsx! {
                            input {
                                class: "penguin-input",
                                r#type: "checkbox",
                                checked: node
                                    .value_inputs()
                                    .get(input_id)
                                    .and_then(|v| if let ValueData::Bool(i) = v() {
                                        Some(i)
                                    } else {
                                        None
                                    })
                                    .unwrap_or(false),
                                oninput: move |evt| {
                                    node.write().value_inputs.insert(input_id, ValueData::Bool(evt.checked()));
                                }
                            }
                        },
                        ValueType::Color => rsx! {
                            input {
                                class: "penguin-input",
                                r#type: "color",
                                value: node
                                    .value_inputs()
                                    .get(input_id)
                                    .and_then(|v| if let ValueData::Color(c) = v() {
                                        Some(format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b))
                                    } else {
                                        None
                                    })
                                    .unwrap_or_else(|| "#000000".to_string()),
                                oninput: move |evt| {
                                    if let Some(color) = Color::from_hex(&evt.value()) {
                                        node.write().value_inputs.insert(input_id, ValueData::Color(color));
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}
