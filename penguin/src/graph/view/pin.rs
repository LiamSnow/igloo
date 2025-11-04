use std::mem;

use igloo_interface::{
    PenguinPinDefn, PenguinPinID, PenguinPinRef, PenguinPinType,
    graph::{PenguinNode, PenguinWireID},
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement, SvgElement};

use crate::{
    app::event::{EventTarget, ListenerBuilder, Listeners, document},
    graph::input::{WebInput, WebInputType},
};

#[derive(Debug)]
pub struct WebPin {
    pref: PenguinPinRef,
    defn: PenguinPinDefn,
    wrapper: Element,
    pub hitbox: HtmlElement,
    /// flow=polygon, value=div
    pin_el: Element,
    pub(super) input_el: Option<WebInput>,
    connections: Vec<PenguinWireID>,
    listeners: Listeners,
}

impl Drop for WebPin {
    fn drop(&mut self) {
        self.wrapper.remove();
    }
}

const UNCONNECTED_COLOR: &str = "#111";

pub fn make(
    parent: &Element,
    defn: &PenguinPinDefn,
    id: &PenguinPinID,
    is_output: bool,
) -> Result<(Element, HtmlElement, Element), JsValue> {
    let document = document();

    let wrapper = document.create_element("div")?;
    wrapper.set_class_name(if is_output {
        "penguin-pin-wrapper output"
    } else {
        "penguin-pin-wrapper input"
    });
    parent.append_child(&wrapper)?;

    let hitbox = document.create_element("div")?.dyn_into::<HtmlElement>()?;
    hitbox.set_class_name("penguin-pin-hitbox");
    wrapper.append_child(&hitbox)?;

    let pin_el = match defn.r#type {
        PenguinPinType::Flow => {
            let svg = document
                .create_element_ns(Some("http://www.w3.org/2000/svg"), "svg")?
                .dyn_into::<SvgElement>()?;

            svg.set_attribute("class", "penguin-pin flow")?;
            svg.set_attribute("width", "16")?;
            svg.set_attribute("height", "16")?;
            svg.set_attribute("viewbox", "0 0 16 16")?;

            hitbox.append_child(&svg)?;

            let polygon =
                document.create_element_ns(Some("http://www.w3.org/2000/svg"), "polygon")?;

            polygon.set_attribute("points", "1,1 12,1 15,8 12,15 1,15")?;
            polygon.set_attribute("fill", UNCONNECTED_COLOR)?;
            polygon.set_attribute("stroke", defn.r#type.color())?;
            polygon.set_attribute("strokeWidth", "2")?;

            svg.append_child(&polygon)?;

            polygon
        }
        PenguinPinType::Value(vt) => {
            let pin_el = document.create_element("div")?;

            pin_el.set_class_name("penguin-pin value");
            pin_el.set_attribute("style", &format!("border-color: {};", vt.color()))?;

            hitbox.append_child(&pin_el)?;

            pin_el
        }
    };

    if !defn.hide_name {
        let name = document.create_element("span")?.dyn_into::<HtmlElement>()?;

        name.set_class_name("penguin-pin-name");
        name.set_inner_text(&id.0);

        wrapper.append_child(&name)?;
    }

    Ok((wrapper, hitbox, pin_el))
}

impl WebPin {
    pub fn new(
        parent: &Element,
        node: &mut PenguinNode,
        pref: PenguinPinRef,
        defn: PenguinPinDefn,
        connections: Vec<PenguinWireID>,
        is_reroute: bool,
    ) -> Result<Self, JsValue> {
        let (wrapper, hitbox, pin_el) = make(parent, &defn, &pref.id, pref.is_output)?;

        let mut input_el = None;

        if !pref.is_output
            && let PenguinPinType::Value(vt) = defn.r#type
        {
            node.ensure_input_pin_value(&pref.id, &vt);
            let pin_value = node.input_pin_values.get(&pref.id).unwrap();

            input_el = Some(WebInput::new(
                &wrapper,
                pref.node_id,
                WebInputType::Pin(pref.id.clone()),
                vt,
                &pin_value.value.to_string(),
                pin_value.size,
            )?);
        }

        let mut builder =
            ListenerBuilder::new(&hitbox, EventTarget::Pin(pref.clone())).add_mouseup()?;
        if !is_reroute {
            builder = builder.add_mousedown()?.add_contextmenu()?;
        }
        let listeners = builder.build();

        let me = Self {
            pref,
            defn,
            wrapper,
            hitbox,
            pin_el,
            input_el,
            connections,
            listeners,
        };

        me.update_fill()?;

        Ok(me)
    }

    pub fn connections(&self) -> &[PenguinWireID] {
        &self.connections
    }

    /// WARN: make sure to update node wires
    pub fn remove_connection(&mut self, wire_id: PenguinWireID) -> Result<(), JsValue> {
        self.connections.retain(|&id| id != wire_id);
        self.update_fill()?;
        Ok(())
    }

    pub fn take_connections(&mut self) -> Result<Vec<PenguinWireID>, JsValue> {
        let mut o = Vec::new();
        mem::swap(&mut self.connections, &mut o);
        self.update_fill()?;
        Ok(o)
    }

    pub fn add_connection(&mut self, connection: PenguinWireID) -> Result<(), JsValue> {
        self.connections.push(connection);
        self.update_fill()
    }

    fn update_fill(&self) -> Result<(), JsValue> {
        let connected = !self.connections.is_empty();

        match self.defn.r#type {
            PenguinPinType::Flow => self.pin_el.set_attribute(
                "fill",
                if connected {
                    self.defn.r#type.color()
                } else {
                    UNCONNECTED_COLOR
                },
            )?,
            PenguinPinType::Value(vt) => {
                self.pin_el.set_attribute(
                    "style",
                    &format!(
                        "border-color: {}; background-color: {};",
                        vt.color(),
                        if connected {
                            vt.color()
                        } else {
                            UNCONNECTED_COLOR
                        },
                    ),
                )?;
            }
        }

        if let Some(input) = &self.input_el {
            input.set_visible(!connected)?;
        }

        Ok(())
    }

    pub fn show_wiring(&mut self, start: &PenguinPinRef) {
        let class = if start.can_connect_to(&self.pref) {
            if start.r#type == self.defn.r#type {
                "penguin-pin-hitbox valid-target"
            } else {
                "penguin-pin-hitbox castable-target"
            }
        } else {
            "penguin-pin-hitbox invalid-target"
        };

        self.hitbox.set_class_name(class);
    }

    pub fn hide_wiring(&mut self) {
        self.hitbox.set_class_name("penguin-pin-hitbox");
    }
}
