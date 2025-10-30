use std::collections::HashMap;

use igloo_interface::{
    NodeConfig, NodeStyle, PenguinNodeDefn, PenguinPinID, PenguinRegistry, PenguinType,
    graph::{PenguinNode, PenguinNodeID, PenguinWire, PenguinWireID},
};
use indexmap::IndexMap;
use maud::html;
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{Element, MouseEvent, ResizeObserver};

use crate::{
    app::APP,
    ffi,
    graph::WebPin,
    interaction::Interaction,
    viewport::{ClientBox, ClientPoint, WorldPoint},
};

#[derive(Debug)]
pub struct WebNode {
    inner: PenguinNode,
    id: PenguinNodeID,
    defn: PenguinNodeDefn,
    el: Element,
    closures: Vec<Closure<dyn FnMut(MouseEvent)>>,
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
        inner: PenguinNode,
        id: PenguinNodeID,
    ) -> Result<Self, JsValue> {
        let defn = registry
            .get_defn(&inner.defn_ref)
            .cloned()
            .ok_or(JsValue::from_str("Unknown Node Definition"))?;

        let document = ffi::document();

        let el = document.create_element("div")?;
        el.set_class_name("penguin-node");
        el.set_attribute(
            "oncontextmenu",
            "event.stopPropagation(); event.preventDefault();",
        )?;

        parent.append_child(&el)?;

        let mut closures = Vec::with_capacity(5);
        let mousedown = Closure::wrap(Box::new(move |e: MouseEvent| {
            if e.button() != 0 {
                return;
            }

            e.prevent_default();
            e.stop_propagation();

            let client_pos = ClientPoint::new(e.client_x(), e.client_y());

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

                app.set_interaction(Interaction::Dragging {
                    node_id: id,
                    start_client_pos: client_pos,
                    start_node_pos,
                });

                app.graph.select_node(id, e.ctrl_key() || e.shift_key());
            });
        }) as Box<dyn FnMut(_)>);

        el.add_event_listener_with_callback("mousedown", mousedown.as_ref().unchecked_ref())?;
        closures.push(mousedown);

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

                let input = document.create_element(el_type)?;

                input.set_class_name("penguin-input");
                // TODO set inner value

                input.set_attribute("onmousedown", "event.stopPropagation()")?;
                match cfg.r#type {
                    PenguinType::Int => {
                        input.set_attribute("type", "number")?;
                        input.set_attribute("step", "1")?;
                        // input.set_inner_text()
                    }
                    PenguinType::Real => {
                        input.set_attribute("type", "number")?;
                        input.set_attribute("step", "any")?;
                    }
                    PenguinType::Text => {
                        let onresize = Closure::wrap(Box::new(move |_| {
                            APP.with(|app| {
                                let mut b = app.borrow_mut();
                                let Some(app) = b.as_mut() else {
                                    return;
                                };

                                app.graph.redraw_node_wires(&id);
                            });
                        })
                            as Box<dyn FnMut(_)>);

                        let observer = ResizeObserver::new(onresize.as_ref().unchecked_ref())?;
                        observer.observe(&input);
                        closures.push(onresize);
                    }
                    PenguinType::Bool => {
                        input.set_attribute("type", "checkbox")?;
                    }
                    PenguinType::Color => {
                        input.set_attribute("type", "color")?;
                    }
                }

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
}
