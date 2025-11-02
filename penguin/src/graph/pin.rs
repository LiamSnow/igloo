use std::{any::Any, mem};

use igloo_interface::{
    PenguinPinDefn, PenguinPinID, PenguinPinType, PenguinType,
    graph::{PenguinNode, PenguinNodeID, PenguinWireID},
};
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{
    Element, Event, HtmlElement, HtmlInputElement, HtmlTextAreaElement, InputEvent, MouseEvent,
    ResizeObserver, SvgElement,
};

use crate::{
    app::APP,
    ffi,
    interaction::{Interaction, WiringState},
};

#[derive(Debug)]
pub struct WebPin {
    node_id: PenguinNodeID,
    id: PenguinPinID,
    defn: PenguinPinDefn,
    is_output: bool,
    wrapper: Element,
    pub hitbox: HtmlElement,
    /// flow=polygon, value=div
    pin_el: Element,
    input_el: Option<Element>,
    connections: Vec<PenguinWireID>,
    closures: Vec<Box<dyn Any>>,
}

impl Drop for WebPin {
    fn drop(&mut self) {
        self.wrapper.remove();
    }
}

impl WebPin {
    pub fn new(
        parent: &Element,
        node_id: PenguinNodeID,
        node: &mut PenguinNode,
        id: PenguinPinID,
        defn: PenguinPinDefn,
        is_output: bool,
        connections: Vec<PenguinWireID>,
    ) -> Result<Self, JsValue> {
        let document = ffi::document();

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
                polygon.set_attribute("fill", "#111")?;
                polygon.set_attribute("stroke", "white")?;
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

        let mut closures: Vec<Box<dyn Any>> = Vec::with_capacity(4);
        let id_1 = id.clone();
        let id_2 = id.clone();
        let hitbox_1 = hitbox.clone();

        let mousedown = Closure::wrap(Box::new(move |e: MouseEvent| {
            if e.button() != 0 {
                return;
            }

            e.prevent_default();
            e.stop_propagation();

            APP.with(|app| {
                let mut b = app.borrow_mut();
                let Some(app) = b.as_mut() else {
                    return;
                };

                if e.shift_key() {
                    app.graph.select_pin_wires(&node_id, &id_1, is_output);
                } else if e.alt_key() {
                    app.graph.delete_pin_wires(&node_id, &id_1, is_output);
                } else {
                    app.start_wiring(e, &hitbox_1, node_id, id_1.clone(), is_output, defn.r#type);
                }
            });
        }) as Box<dyn FnMut(_)>);

        hitbox.add_event_listener_with_callback("mousedown", mousedown.as_ref().unchecked_ref())?;
        closures.push(Box::new(mousedown));

        let mouseup = Closure::wrap(Box::new(move |e: MouseEvent| {
            if e.button() != 0 {
                return;
            }

            e.prevent_default();
            e.stop_propagation();

            APP.with(|app| {
                let mut b = app.borrow_mut();
                let Some(app) = b.as_mut() else {
                    return;
                };

                if let Interaction::Wiring(ws) = app.interaction() {
                    let ws = ws.clone();
                    let res = app.place_wire(ws, node_id, id_2.clone(), defn.r#type, is_output);
                    if let Err(e) = res {
                        log::error!("Failed to place wire: {e:?}");
                    }
                }
            });
        }) as Box<dyn FnMut(_)>);

        hitbox.add_event_listener_with_callback("mouseup", mouseup.as_ref().unchecked_ref())?;
        closures.push(Box::new(mouseup));

        let mut input_el = None;

        if !is_output && let PenguinPinType::Value(vt) = defn.r#type {
            // TODO on input -> save to undo tree?

            let el_type = if matches!(vt, PenguinType::Text) {
                "textarea"
            } else {
                "input"
            };

            let input = document.create_element(el_type)?;
            input.set_class_name("penguin-input");
            input.set_attribute("onmousedown", "event.stopPropagation()")?;
            node.ensure_input_pin_value(&id, &vt);
            let pin_value = node.input_pin_values.get(&id).unwrap();
            input.set_attribute("value", &pin_value.value.to_string())?;

            match vt {
                PenguinType::Int => {
                    input.set_attribute("type", "number")?;
                    input.set_attribute("step", "1")?;
                }
                PenguinType::Real => {
                    input.set_attribute("type", "number")?;
                    input.set_attribute("step", "any")?;
                }
                PenguinType::Text => {
                    let textarea = input.dyn_ref::<HtmlTextAreaElement>().unwrap().clone();
                    textarea.set_value(&pin_value.value.to_string());
                    let (width, height) = pin_value.size.unwrap();

                    textarea.set_attribute(
                        "style",
                        &format!("width: {width}px; height: {height}px; resize: both;"),
                    )?;

                    let id_1 = id.clone();
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
                                Ok(node) => {
                                    let Some(cfg_value) =
                                        node.inner.input_pin_values.get_mut(&id_1)
                                    else {
                                        // TODO log
                                        return;
                                    };

                                    cfg_value.size = Some(size);
                                }
                                Err(e) => {
                                    log::error!("Error getting node: {e:?}");
                                }
                            }
                        });
                    }) as Box<dyn FnMut(_)>);

                    let observer = ResizeObserver::new(onresize.as_ref().unchecked_ref())?;
                    observer.observe(&input);
                    closures.push(Box::new(onresize));
                }
                PenguinType::Bool => {
                    input.set_attribute("type", "checkbox")?;
                }
                PenguinType::Color => {
                    input.set_attribute("type", "color")?;
                }
            }

            let input_1 = input.clone();
            let id_1 = id.clone();
            let oninput = Closure::wrap(Box::new(move |e: InputEvent| {
                APP.with(|app| {
                    let mut b = app.borrow_mut();
                    let Some(app) = b.as_mut() else {
                        return;
                    };

                    match app.graph.node_mut(&node_id) {
                        Ok(node) => {
                            let Some(cfg_value) = node.inner.input_pin_values.get_mut(&id_1) else {
                                // TODO log
                                return;
                            };

                            match vt {
                                PenguinType::Text => {
                                    // FIXME unwrap
                                    let el = input_1.dyn_ref::<HtmlTextAreaElement>().unwrap();
                                    cfg_value.set_from_string(el.value());
                                }
                                _ => {
                                    // FIXME unwrap
                                    let el = input_1.dyn_ref::<HtmlInputElement>().unwrap();
                                    cfg_value.set_from_string(el.value());
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Error getting node: {e:?}");
                        }
                    }
                });
            }) as Box<dyn FnMut(_)>);

            input.add_event_listener_with_callback("input", oninput.as_ref().unchecked_ref())?;
            closures.push(Box::new(oninput));

            wrapper.append_child(&input)?;

            input_el = Some(input);
        }

        let me = Self {
            node_id,
            id,
            defn,
            is_output,
            wrapper,
            hitbox,
            pin_el,
            input_el,
            connections,
            closures,
        };

        me.update_fill()?;

        Ok(me)
    }

    pub fn connections(&self) -> &[PenguinWireID] {
        &self.connections
    }

    /// Make sure to update node wires
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
            PenguinPinType::Flow => self
                .pin_el
                .set_attribute("fill", if connected { "white" } else { "#111" })?,
            PenguinPinType::Value(vt) => {
                self.pin_el.set_attribute(
                    "style",
                    &format!(
                        "border-color: {}; background-color: {};",
                        vt.color(),
                        if connected { vt.color() } else { "#111" },
                    ),
                )?;
            }
        }

        if let Some(el) = &self.input_el {
            if connected {
                el.set_attribute("style", "display: none;")?;
            } else {
                el.remove_attribute("style")?;
            }
        }

        Ok(())
    }

    pub fn show_wiring(&mut self, ws: &WiringState) {
        let class = if ws.is_valid_end(self.node_id, self.defn.r#type, self.is_output) {
            if ws.wire_type == self.defn.r#type {
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
