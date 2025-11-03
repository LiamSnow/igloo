use std::{any::Any, collections::HashMap};

use igloo_interface::{
    NodeInputFeatureID, NodeStyle, PenguinNodeDefn, PenguinPinID, PenguinRegistry,
    graph::{PenguinNode, PenguinNodeID, PenguinWire, PenguinWireID},
};
use indexmap::IndexMap;
use maud::html;
use wasm_bindgen::JsValue;
use web_sys::{Element, MouseEvent};

use crate::{
    ffi::{self, add_app_event_listener},
    graph::WebPin,
    graph::input::{WebInput, WebInputMode},
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
    input_feature_els: IndexMap<NodeInputFeatureID, WebInput>,
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

        let mut closures = Vec::with_capacity(5);
        add_app_event_listener(
            &el,
            "mousedown",
            &mut closures,
            move |app, e: MouseEvent| {
                if e.button() != 0 {
                    return;
                }

                e.prevent_default();
                e.stop_propagation();

                app.focus();

                app.graph.select_node(id, e.ctrl_key() || e.shift_key());

                let cpos = ClientPoint::new(e.client_x(), e.client_y());
                app.start_dragging(id, cpos);
            },
        )?;

        // style
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

        // input configs
        let input_features = defn.input_features();
        let mut input_feature_els = IndexMap::with_capacity(input_features.len());
        if !input_features.is_empty() {
            let content_el = document.create_element("div")?;
            content_el.set_class_name("penguin-node-content");
            el.append_child(&content_el)?;

            for feature in input_features {
                inner.ensure_input_feature_value(feature);
                let feature_value = inner.input_feature_values.get(&feature.id).unwrap();

                let input = WebInput::new(
                    &content_el,
                    id,
                    WebInputMode::NodeFeature(feature.id.clone()),
                    feature.r#type,
                    &feature_value.value.to_string(),
                    feature_value.size,
                )?;

                input_feature_els.insert(feature.id.clone(), input);
            }
        }

        // input pins
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

        // output pins
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
            input_feature_els,
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
