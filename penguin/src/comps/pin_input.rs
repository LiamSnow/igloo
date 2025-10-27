use crate::types::*;
use dioxus::prelude::*;
use igloo_interface::{Color, ValueData, ValueType};

#[component]
pub fn PinInput(node: Store<Node>, pin_ref: PinRef, value_type: ValueType) -> Element {
    rsx! {
        match value_type {
            ValueType::Int => rsx! {
                input {
                    class: "penguin-input",
                    r#type: "number",
                    step: "1",
                    value: node
                        .pin_values()
                        .get(pin_ref)
                        .and_then(|v| if let ValueData::Int(i) = v() {
                            Some(i)
                        } else {
                            None
                        })
                        .unwrap_or(0),
                    oninput: move |evt| {
                        if let Ok(val) = evt.value().parse::<i64>() {
                            node.write().pin_values.insert(pin_ref, ValueData::Int(val));
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
                        .pin_values()
                        .get(pin_ref)
                        .and_then(|v| if let ValueData::Real(r) = v() {
                            Some(r)
                        } else {
                            None
                        })
                        .unwrap_or(0.),
                    oninput: move |evt| {
                        if let Ok(val) = evt.value().parse::<f64>() {
                            node.write().pin_values.insert(pin_ref, ValueData::Real(val));
                        }
                    }
                }
            },
            ValueType::Text => rsx! {
                textarea {
                    class: "penguin-input",
                    value: node
                        .pin_values()
                        .get(pin_ref)
                        .and_then(|v| if let ValueData::Text(t) = v() {
                            Some(t)
                        } else {
                            None
                        })
                        .unwrap_or(String::default()),
                    oninput: move |evt| {
                        node.write().pin_values.insert(pin_ref, ValueData::Text(evt.value()));
                    }
                }
            },
            ValueType::Bool => rsx! {
                input {
                    class: "penguin-input",
                    r#type: "checkbox",
                    checked: node
                        .pin_values()
                        .get(pin_ref)
                        .and_then(|v| if let ValueData::Bool(b) = v() {
                            Some(b)
                        } else {
                            None
                        })
                        .unwrap_or(false),
                    oninput: move |evt| {
                        node.write().pin_values.insert(pin_ref, ValueData::Bool(evt.checked()));
                    }
                }
            },
            ValueType::Color => rsx! {
                input {
                    class: "penguin-input",
                    r#type: "color",
                    value: node
                        .pin_values()
                        .get(pin_ref)
                        .and_then(|v| if let ValueData::Color(c) = v() {
                            Some(format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b))
                        } else {
                            None
                        })
                        .unwrap_or_else(|| "#000000".to_string()),
                    oninput: move |evt| {
                        if let Some(color) = Color::from_hex(&evt.value()) {
                            node.write().pin_values.insert(pin_ref, ValueData::Color(color));
                        }
                    }
                }
            },
        }
    }
}
