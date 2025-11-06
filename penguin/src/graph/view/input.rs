use crate::app::event::{EventTarget, ListenerBuilder, Listeners, document};
use igloo_interface::{
    IglooType,
    penguin::{NodeInputFeatureID, PenguinPinID, graph::PenguinNodeID},
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement, HtmlInputElement, HtmlTextAreaElement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebInputType {
    Pin(PenguinPinID),
    NodeFeature(NodeInputFeatureID),
}

#[derive(Debug)]
pub struct WebInput {
    el: Element,
    node_id: PenguinNodeID,
    mode: WebInputType,
    value_type: IglooType,
    listeners: Listeners,
}

impl Drop for WebInput {
    fn drop(&mut self) {
        self.el.remove();
    }
}

impl WebInput {
    pub fn new(
        parent: &Element,
        node_id: PenguinNodeID,
        mode: WebInputType,
        value_type: IglooType,
        initial_value: &str,
        initial_size: Option<(i32, i32)>,
    ) -> Result<Self, JsValue> {
        let document = document();

        let el_type = if matches!(value_type, IglooType::Text) {
            "textarea"
        } else {
            "input"
        };

        let el = document.create_element(el_type)?;
        el.set_class_name("penguin-input");

        let el_clone = el.clone();
        let mut listeners =
            ListenerBuilder::new(&el, EventTarget::NodeInput(node_id, mode.clone()))
                .add_input(move || match value_type {
                    IglooType::Text => el_clone.dyn_ref::<HtmlTextAreaElement>().unwrap().value(),
                    IglooType::Boolean => el_clone
                        .dyn_ref::<HtmlInputElement>()
                        .unwrap()
                        .checked()
                        .to_string(),
                    _ => el_clone.dyn_ref::<HtmlInputElement>().unwrap().value(),
                })?
                .add_mousedown()?
                .add_mousemove()?
                .add_contextmenu()?
                .add_keydown()?
                .add_copy()?
                .add_paste()?
                .add_cut()?
                .build();

        match value_type {
            IglooType::Integer => {
                el.set_attribute("type", "number")?;
                el.set_attribute("step", "1")?;
                el.set_attribute("value", initial_value)?;
            }
            IglooType::Real => {
                el.set_attribute("type", "number")?;
                el.set_attribute("step", "any")?;
                el.set_attribute("value", initial_value)?;
            }
            IglooType::Text => {
                let textarea = el.dyn_ref::<HtmlTextAreaElement>().unwrap().clone();
                textarea.set_value(initial_value);
                let (width, height) = initial_size.unwrap();

                let s = textarea.style();
                s.set_property("width", &format!("{width}px"))?;
                s.set_property("height", &format!("{height}px"))?;
                s.set_property("resize", "both")?;

                listeners.add_resize(
                    &el,
                    textarea.dyn_into::<HtmlElement>().unwrap(),
                    EventTarget::NodeInput(node_id, mode.clone()),
                )?;
            }
            IglooType::Boolean => {
                el.set_attribute("type", "checkbox")?;
                let input = el.dyn_ref::<HtmlInputElement>().unwrap();
                input.set_checked(initial_value == "true");
            }
            IglooType::Color => {
                el.set_attribute("type", "color")?;
                el.set_attribute("value", initial_value)?;
            }
            IglooType::Date => {
                el.set_attribute("type", "date")?;
                el.set_attribute("value", initial_value)?;
            }
            IglooType::Time => {
                el.set_attribute("type", "time")?;
                el.set_attribute("value", initial_value)?;
            }
            IglooType::IntegerList => todo!(),
            IglooType::RealList => todo!(),
            IglooType::TextList => todo!(),
            IglooType::BooleanList => todo!(),
            IglooType::ColorList => todo!(),
            IglooType::DateList => todo!(),
            IglooType::TimeList => todo!(),
            IglooType::Enum(igloo_enum_type) => todo!(),
        }

        parent.append_child(&el)?;

        Ok(Self {
            el,
            node_id,
            mode,
            value_type,
            listeners,
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

    pub fn update_value(&self, value: &str) -> Result<(), JsValue> {
        match self.value_type {
            IglooType::Text => {
                self.el
                    .dyn_ref::<HtmlTextAreaElement>()
                    .unwrap()
                    .set_value(value);
            }
            IglooType::Boolean => {
                self.el
                    .dyn_ref::<HtmlInputElement>()
                    .unwrap()
                    .set_checked(value == "true");
            }
            _ => {
                self.el
                    .dyn_ref::<HtmlInputElement>()
                    .unwrap()
                    .set_value(value);
            }
        }
        Ok(())
    }

    pub fn update_size(&self, size: (i32, i32)) -> Result<(), JsValue> {
        if let IglooType::Text = self.value_type {
            let (width, height) = size;
            let textarea = self.el.dyn_ref::<HtmlTextAreaElement>().unwrap();
            let s = textarea.style();
            s.set_property("width", &format!("{width}px"))?;
            s.set_property("height", &format!("{height}px"))?;
            s.set_property("resize", "both")?;
        }
        Ok(())
    }
}
