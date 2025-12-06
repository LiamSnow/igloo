use igloo_interface::{
    penguin::{NodeInputFeatureID, NodeInputType, PenguinPinID, graph::PenguinNodeID},
    types::IglooType,
};

use crate::dom::{self, Input, TextArea, events::EventTarget, node::DomNode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebInputType {
    Pin(PenguinPinID),
    NodeFeature(NodeInputFeatureID),
}

#[derive(Debug)]
pub struct WebInput {
    el: WebInputElement,
    value_type: IglooType,
}

#[derive(Debug)]
enum WebInputElement {
    Input(DomNode<Input>),
    TextArea(DomNode<TextArea>),
}

impl WebInput {
    pub fn new<T>(
        parent: &DomNode<T>,
        node_id: PenguinNodeID,
        mode: WebInputType,
        value_type: IglooType,
        input_type: NodeInputType,
        initial_value: &str,
        initial_size: Option<(i32, i32)>,
    ) -> Self {
        let el = match input_type {
            NodeInputType::Input => {
                let mut el = dom::input()
                    .class("penguin-input")
                    .event_target(EventTarget::NodeInput(node_id, mode.clone()))
                    .listen_mousedown()
                    .listen_mousemove()
                    .listen_contextmenu()
                    .listen_keydown()
                    .listen_copy()
                    .listen_paste()
                    .listen_cut()
                    .value(initial_value)
                    .remove_on_drop()
                    .mount(parent);

                let inner = el.element.clone();
                el.listen_input(move || match value_type {
                    IglooType::Boolean => dom::js::get_checked(&inner).to_string(),
                    _ => dom::js::get_value(&inner),
                });

                match value_type {
                    IglooType::Integer => {
                        el.set_type("number");
                        el.set_attr("step", "1");
                    }
                    IglooType::Real => {
                        el.set_type("number");
                        el.set_attr("step", "any");
                    }
                    IglooType::Text => {
                        el.set_type("text");
                    }
                    IglooType::Boolean => {
                        el.set_type("checkbox");
                        el.set_checked(initial_value == "true");
                    }
                    IglooType::Color => {
                        el.set_type("color");
                    }
                    IglooType::Date => {
                        el.set_type("date");
                    }
                    IglooType::Time => {
                        el.set_type("time");
                    }
                    _ => unimplemented!(),
                }

                WebInputElement::Input(el)
            }
            NodeInputType::TextArea => {
                let (width, height) = initial_size.unwrap();

                let mut el = dom::textarea()
                    .class("penguin-input")
                    .size(width as f64, height as f64)
                    .event_target(EventTarget::NodeInput(node_id, mode.clone()))
                    .listen_mousedown()
                    .listen_mousemove()
                    .listen_contextmenu()
                    .listen_keydown()
                    .listen_copy()
                    .listen_paste()
                    .listen_cut()
                    .listen_resize()
                    .value(initial_value)
                    .remove_on_drop()
                    .mount(parent);

                let inner = el.element.clone();
                el.listen_input(move || dom::js::get_value(&inner));

                WebInputElement::TextArea(el)
            }
            NodeInputType::Select(_) => todo!(),
        };

        Self { el, value_type }
    }

    pub fn set_visible(&self, visible: bool) {
        match &self.el {
            WebInputElement::Input(el) => {
                el.set_visible(visible);
            }
            WebInputElement::TextArea(el) => {
                el.set_visible(visible);
            }
        }
    }

    pub fn update_value(&self, value: &str) {
        match &self.el {
            WebInputElement::Input(el) => match self.value_type {
                IglooType::Boolean => el.set_checked(value == "true"),
                _ => el.set_value(value),
            },
            WebInputElement::TextArea(el) => el.set_value(value),
        }
    }

    pub fn update_size(&self, size: (i32, i32)) {
        if let WebInputElement::Input(el) = &self.el {
            el.set_size(size.0 as f64, size.1 as f64);
        }
    }
}
