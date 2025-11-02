use std::{any::Any, collections::HashMap};

use igloo_interface::{
    NodeConfig, NodeStyle, PenguinNodeDefn, PenguinPinID, PenguinRegistry, PenguinType,
    graph::{PenguinNode, PenguinNodeID, PenguinWire, PenguinWireID},
};
use indexmap::IndexMap;
use maud::html;
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{
    Element, Event, HtmlElement, HtmlInputElement, HtmlTextAreaElement, InputEvent, MouseEvent,
    ResizeObserver,
};

use crate::{
    app::APP,
    ffi,
    graph::WebPin,
    viewport::{ClientBox, ClientPoint, WorldPoint},
};

#[derive(Debug)]
pub struct WebNode {
    pub(super) inner: PenguinNode,
    id: PenguinNodeID,
    defn: PenguinNodeDefn,
    el: Element,
    closures: Vec<Box<dyn Any>>,
    pub inputs: IndexMap<PenguinPinID, WebPin>,
    pub outputs: IndexMap<PenguinPinID, WebPin>,
}

impl Drop for WebNode {
    fn drop(&mut self) {
        self.el.remove();
    }
}

impl WebNode {
    pub fn new(
        parent: &Element,
        registry: &PenguinRegistry,
        wires: Option<&HashMap<PenguinWireID, PenguinWire>>,
        mut inner: PenguinNode,
        id: PenguinNodeID,
    ) -> Result<Self, JsValue> {
        let defn = registry
            .get_defn(&inner.defn_ref)
            .cloned()
            .ok_or(JsValue::from_str(&format!(
                "Unknown Node Definition {}",
                inner.defn_ref
            )))?;

        let document = ffi::document();

        let el = document.create_element("div")?;
        el.set_class_name("penguin-node");
        el.set_attribute(
            "oncontextmenu",
            "event.stopPropagation(); event.preventDefault();",
        )?;

        parent.append_child(&el)?;

        let mut closures: Vec<Box<dyn Any>> = Vec::with_capacity(5);
        let mousedown = Closure::wrap(Box::new(move |e: MouseEvent| {
            if e.button() != 0 {
                return;
            }

            e.prevent_default();
            e.stop_propagation();

            let cpos = ClientPoint::new(e.client_x(), e.client_y());

            APP.with(|app| {
                let mut b = app.borrow_mut();
                let Some(app) = b.as_mut() else {
                    return;
                };

                let start_node_pos = app
                    .graph
                    .nodes
                    .get(&id)
                    .map(|n| n.pos())
                    .unwrap_or_default();

                app.graph.select_node(id, e.ctrl_key() || e.shift_key());

                app.start_dragging(id, cpos);
            });
        }) as Box<dyn FnMut(_)>);

        el.add_event_listener_with_callback("mousedown", mousedown.as_ref().unchecked_ref())?;
        closures.push(Box::new(mousedown));

        match &defn.style {
            NodeStyle::Normal(icon) => {
                el.set_inner_html(
                    &html! {
                        .penguin-node-title {
                            (defn.title)
                        }
                    }
                    .into_string(),
                );
            }
            NodeStyle::Background(bg) => {
                el.set_inner_html(
                    &html! {
                        .penguin-node-bg {
                            (bg)
                        }
                    }
                    .into_string(),
                );
            }
            NodeStyle::None => {
                el.set_inner_html("");
            }
        }

        let input_cfgs: Vec<_> = defn
            .cfg
            .iter()
            .filter_map(|cfg| {
                if let NodeConfig::Input(config) = cfg {
                    Some(config)
                } else {
                    None
                }
            })
            .collect();

        if !input_cfgs.is_empty() {
            let content_el = document.create_element("div")?;
            content_el.set_class_name("penguin-node-content");
            el.append_child(&content_el)?;
            for cfg in input_cfgs {
                let el_type = if matches!(cfg.r#type, PenguinType::Text) {
                    "textarea"
                } else {
                    "input"
                };

                let input = document
                    .create_element(el_type)?
                    .dyn_into::<HtmlElement>()?;
                input.set_class_name("penguin-input");
                input.set_attribute("onmousedown", "event.stopPropagation()")?;
                inner.ensure_input_cfg_value(cfg);
                let cfg_value = inner.input_cfg_values.get(&cfg.id).unwrap();
                input.set_attribute("value", &cfg_value.value.to_string())?;

                match cfg.r#type {
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
                        textarea.set_value(&cfg_value.value.to_string());
                        let (width, height) = cfg_value.size.unwrap();
                        textarea
                            .style()
                            .set_property("width", &format!("{width}px"))?;
                        textarea
                            .style()
                            .set_property("height", &format!("{height}px"))?;
                        textarea.style().set_property("resize", "both")?;

                        let cfg_id = cfg.id.clone();
                        let onresize = Closure::wrap(Box::new(move |_: Event| {
                            APP.with(|app| {
                                let mut b = app.borrow_mut();
                                let Some(app) = b.as_mut() else {
                                    return;
                                };

                                if let Err(e) = app.graph.redraw_node_wires(&id) {
                                    log::error!("Error redrawing node wires: {e:?}");
                                }

                                match app.graph.node_mut(&id) {
                                    Ok(node) => {
                                        let Some(cfg_value) =
                                            node.inner.input_cfg_values.get_mut(&cfg_id)
                                        else {
                                            // TODO log
                                            return;
                                        };

                                        cfg_value.size = Some((
                                            textarea.offset_width(),
                                            textarea.offset_height(),
                                        ));
                                    }
                                    Err(e) => {
                                        log::error!("Error getting node: {e:?}");
                                    }
                                }
                            });
                        })
                            as Box<dyn FnMut(_)>);

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

                let cfg_id = cfg.id.clone();
                let cfg_type = cfg.r#type.clone();
                let input_1 = input.clone();
                let oninput = Closure::wrap(Box::new(move |e: InputEvent| {
                    APP.with(|app| {
                        let mut b = app.borrow_mut();
                        let Some(app) = b.as_mut() else {
                            return;
                        };

                        match app.graph.node_mut(&id) {
                            Ok(node) => {
                                let Some(cfg_value) = node.inner.input_cfg_values.get_mut(&cfg_id)
                                else {
                                    // TODO log
                                    return;
                                };

                                match cfg_type {
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

                input
                    .add_event_listener_with_callback("input", oninput.as_ref().unchecked_ref())?;
                closures.push(Box::new(oninput));

                content_el.append_child(&input)?;
            }
        }

        let inputs_el = document.create_element("div")?;
        inputs_el.set_class_name("penguin-node-inputs");
        el.append_child(&inputs_el)?;
        let mut inputs = IndexMap::with_capacity(defn.inputs.len());
        let fwires: Option<Vec<_>> = wires.map(|wires| {
            wires
                .iter()
                .filter(|(_, wire)| wire.to_node == id)
                .collect()
        });
        for (pin_id, pin_defn) in &defn.inputs {
            let connections = fwires.as_ref().map(|wires| {
                wires
                    .iter()
                    .filter(|(_, wire)| wire.to_pin == pin_id.clone())
                    .map(|(id, _)| **id)
                    .collect()
            });

            inputs.insert(
                pin_id.clone(),
                WebPin::new(
                    &inputs_el,
                    id,
                    &mut inner,
                    pin_id.clone(),
                    pin_defn.clone(),
                    false,
                    connections.unwrap_or_default(),
                )?,
            );
        }

        let outputs_el = document.create_element("div")?;
        outputs_el.set_class_name("penguin-node-outputs");
        el.append_child(&outputs_el)?;
        let mut outputs = IndexMap::with_capacity(defn.outputs.len());
        let fwires: Option<Vec<_>> = wires.map(|wires| {
            wires
                .iter()
                .filter(|(_, wire)| wire.from_node == id)
                .collect()
        });
        for (pin_id, pin_defn) in &defn.outputs {
            let connections = fwires.as_ref().map(|wires| {
                wires
                    .iter()
                    .filter(|(_, wire)| wire.from_pin == pin_id.clone())
                    .map(|(id, _)| **id)
                    .collect()
            });

            outputs.insert(
                pin_id.clone(),
                WebPin::new(
                    &outputs_el,
                    id,
                    &mut inner,
                    pin_id.clone(),
                    pin_defn.clone(),
                    true,
                    connections.unwrap_or_default(),
                )?,
            );
        }

        // TODO Variadic controls

        let me = WebNode {
            inner,
            id,
            defn,
            el,
            closures,
            inputs,
            outputs,
        };

        me.update_transform()?;

        Ok(me)
    }

    pub fn update_transform(&self) -> Result<(), JsValue> {
        let transform = format!(
            "transform: translate({}px, {}px);",
            self.inner.x, self.inner.y
        );
        self.el.set_attribute("style", &transform)
    }

    pub fn set_pos(&mut self, pos: WorldPoint) -> Result<(), JsValue> {
        self.inner.x = pos.x;
        self.inner.y = pos.y;
        self.update_transform()
    }

    pub fn pos(&self) -> WorldPoint {
        WorldPoint::new(self.inner.x, self.inner.y)
    }

    pub fn select(&self, selected: bool) {
        if selected {
            self.el.set_class_name("penguin-node selected");
        } else {
            self.el.set_class_name("penguin-node");
        }
    }

    pub fn client_box(&self) -> ClientBox {
        let rect = self.el.get_bounding_client_rect();
        ClientBox::new(
            ClientPoint::new(rect.x() as i32, rect.y() as i32),
            ClientPoint::new(
                (rect.x() + rect.width()) as i32,
                (rect.y() + rect.height()) as i32,
            ),
        )
    }

    pub fn inner(&self) -> &PenguinNode {
        &self.inner
    }
}
