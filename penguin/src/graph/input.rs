use crate::{app::APP, ffi};
use igloo_interface::{NodeInputFeatureID, PenguinPinID, PenguinType, graph::PenguinNodeID};
use std::any::Any;
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{Element, Event, HtmlInputElement, HtmlTextAreaElement, InputEvent, ResizeObserver};

#[derive(Debug, Clone)]
pub enum WebInputMode {
    Pin(PenguinPinID),
    NodeFeature(NodeInputFeatureID),
}

#[derive(Debug)]
pub struct WebInput {
    el: Element,
    node_id: PenguinNodeID,
    mode: WebInputMode,
    value_type: PenguinType,
    closures: Vec<Box<dyn Any>>,
    observer: Option<ResizeObserver>,
}

impl Drop for WebInput {
    fn drop(&mut self) {
        if let Some(observer) = &self.observer {
            observer.disconnect();
        }
        self.el.remove();
    }
}

impl WebInput {
    pub fn new(
        parent: &Element,
        node_id: PenguinNodeID,
        mode: WebInputMode,
        value_type: PenguinType,
        initial_value: &str,
        initial_size: Option<(i32, i32)>,
    ) -> Result<Self, JsValue> {
        let document = ffi::document();

        let el_type = if matches!(value_type, PenguinType::Text) {
            "textarea"
        } else {
            "input"
        };

        let el = document.create_element(el_type)?;
        el.set_class_name("penguin-input");
        el.set_attribute("onmousedown", "event.stopPropagation()")?;
        el.set_attribute("oncopy", "event.stopPropagation()")?;
        el.set_attribute("onpaste", "event.stopPropagation()")?;
        el.set_attribute("oncut", "event.stopPropagation()")?;

        let mut closures: Vec<Box<dyn Any>> = Vec::with_capacity(3);
        let mut observer = None;

        match value_type {
            PenguinType::Int => {
                el.set_attribute("type", "number")?;
                el.set_attribute("step", "1")?;
                el.set_attribute("value", initial_value)?;
            }
            PenguinType::Real => {
                el.set_attribute("type", "number")?;
                el.set_attribute("step", "any")?;
                el.set_attribute("value", initial_value)?;
            }
            PenguinType::Text => {
                let textarea = el.dyn_ref::<HtmlTextAreaElement>().unwrap().clone();
                textarea.set_value(initial_value);
                let (width, height) = initial_size.unwrap();
                textarea
                    .style()
                    .set_property("width", &format!("{width}px"))?;
                textarea
                    .style()
                    .set_property("height", &format!("{height}px"))?;
                textarea.style().set_property("resize", "both")?;

                let mode_clone = mode.clone();
                let onresize = Closure::wrap(Box::new(move |_: Event| {
                    APP.with(|app| {
                        let mut b = app.borrow_mut();
                        let Some(app) = b.as_mut() else {
                            return;
                        };

                        if let Err(e) = app.graph.redraw_node_wires(&node_id) {
                            log::error!("Error redrawing node wires: {e:?}");
                        }

                        let size = (textarea.offset_width(), textarea.offset_height());
                        if size == (0, 0) {
                            // occurs when textarea is not visible
                            return;
                        }

                        match app.graph.node_mut(&node_id) {
                            Ok(node) => match &mode_clone {
                                WebInputMode::Pin(pin_id) => {
                                    let Some(value) = node.inner.input_pin_values.get_mut(pin_id)
                                    else {
                                        return;
                                    };
                                    value.size = Some(size);
                                }
                                WebInputMode::NodeFeature(input_feature_id) => {
                                    let Some(value) =
                                        node.inner.input_feature_values.get_mut(input_feature_id)
                                    else {
                                        return;
                                    };
                                    value.size = Some(size);
                                }
                            },
                            Err(e) => {
                                log::error!("Error getting node: {e:?}");
                            }
                        }
                    });
                }) as Box<dyn FnMut(_)>);

                let o = ResizeObserver::new(onresize.as_ref().unchecked_ref())?;
                o.observe(&el);
                closures.push(Box::new(onresize));
                observer = Some(o);
            }
            PenguinType::Bool => {
                el.set_attribute("type", "checkbox")?;
                let input = el.dyn_ref::<HtmlInputElement>().unwrap();
                input.set_checked(initial_value == "true");
            }
            PenguinType::Color => {
                el.set_attribute("type", "color")?;
                el.set_attribute("value", initial_value)?;
            }
        }

        let el_clone = el.clone();
        let mode_clone = mode.clone();
        let oninput = Closure::wrap(Box::new(move |_: InputEvent| {
            APP.with(|app| {
                let mut b = app.borrow_mut();
                let Some(app) = b.as_mut() else {
                    return;
                };

                match app.graph.node_mut(&node_id) {
                    Ok(node) => {
                        let value = match value_type {
                            PenguinType::Text => {
                                el_clone.dyn_ref::<HtmlTextAreaElement>().unwrap().value()
                            }
                            PenguinType::Bool => el_clone
                                .dyn_ref::<HtmlInputElement>()
                                .unwrap()
                                .checked()
                                .to_string(),
                            _ => el_clone.dyn_ref::<HtmlInputElement>().unwrap().value(),
                        };

                        match &mode_clone {
                            WebInputMode::Pin(pin_id) => {
                                let Some(pin_value) = node.inner.input_pin_values.get_mut(pin_id)
                                else {
                                    return;
                                };
                                pin_value.set_from_string(value);
                            }
                            WebInputMode::NodeFeature(input_feature_id) => {
                                let Some(feature_value) =
                                    node.inner.input_feature_values.get_mut(input_feature_id)
                                else {
                                    return;
                                };
                                feature_value.set_from_string(value);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error getting node: {e:?}");
                    }
                }
            });
        }) as Box<dyn FnMut(_)>);

        el.add_event_listener_with_callback("input", oninput.as_ref().unchecked_ref())?;
        closures.push(Box::new(oninput));

        parent.append_child(&el)?;

        Ok(Self {
            el,
            node_id,
            mode,
            value_type,
            closures,
            observer,
        })
    }

    pub fn set_visible(&self, visible: bool) -> Result<(), JsValue> {
        if visible {
            self.el.remove_attribute("style")?;
        } else {
            self.el.set_attribute("style", "display: none;")?;
        }
        Ok(())
    }
}
