use crate::app::event::{EventTarget, ListenerBuilder, Listeners, document};
use igloo_interface::{NodeInputFeatureID, PenguinPinID, PenguinType, graph::PenguinNodeID};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement, HtmlInputElement, HtmlTextAreaElement};

#[derive(Debug, Clone)]
pub enum WebInputType {
    Pin(PenguinPinID),
    NodeFeature(NodeInputFeatureID),
}

#[derive(Debug)]
pub struct WebInput {
    el: Element,
    node_id: PenguinNodeID,
    mode: WebInputType,
    value_type: PenguinType,
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
        value_type: PenguinType,
        initial_value: &str,
        initial_size: Option<(i32, i32)>,
    ) -> Result<Self, JsValue> {
        let document = document();

        let el_type = if matches!(value_type, PenguinType::Text) {
            "textarea"
        } else {
            "input"
        };

        let el = document.create_element(el_type)?;
        el.set_class_name("penguin-input");

        let el_clone = el.clone();
        let mut listeners = ListenerBuilder::new(&el, EventTarget::Input(node_id, mode.clone()))
            .add_input(false, move || match value_type {
                PenguinType::Text => el_clone.dyn_ref::<HtmlTextAreaElement>().unwrap().value(),
                PenguinType::Bool => el_clone
                    .dyn_ref::<HtmlInputElement>()
                    .unwrap()
                    .checked()
                    .to_string(),
                _ => el_clone.dyn_ref::<HtmlInputElement>().unwrap().value(),
            })?
            .add_mousemove(false)?
            .add_mouseup(false)?
            .add_mousedown(false)?
            .add_contextmenu(false)?
            .add_keydown(false)?
            .add_copy(false)?
            .add_paste(false)?
            .add_cut(false)?
            .build();

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

                listeners.add_resize(
                    &el,
                    textarea.dyn_into::<HtmlElement>().unwrap(),
                    EventTarget::Input(node_id, mode.clone()),
                )?;
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
}
