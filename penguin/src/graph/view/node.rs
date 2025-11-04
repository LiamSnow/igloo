use igloo_interface::{
    NodeInputFeatureID, NodeStyle, PenguinNodeDefn, PenguinPinID, PenguinPinRef, PenguinRegistry,
    graph::{PenguinNode, PenguinNodeID, PenguinWire, PenguinWireID},
};
use indexmap::IndexMap;
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use web_sys::Element;

use crate::{
    app::event::{EventTarget, ListenerBuilder, Listeners, document},
    graph::{
        input::{WebInput, WebInputType},
        pin::WebPin,
    },
    viewport::{ClientBox, ClientPoint, WorldPoint},
};

#[derive(Debug)]
pub struct WebNode {
    pub inner: PenguinNode,
    id: PenguinNodeID,
    defn: PenguinNodeDefn,
    el: Element,
    listeners: Listeners,
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

        let document = document();

        let el = document.create_element("div")?;
        el.set_class_name("penguin-node");
        parent.append_child(&el)?;

        // style
        match &defn.style {
            NodeStyle::Normal(icon) => {
                el.set_inner_html(&format!(
                    r#"<div class="penguin-node-title">{}</div>"#,
                    defn.title
                ));
            }
            NodeStyle::Background(bg) => {
                el.set_inner_html(&format!(r#"<div class="penguin-node-bg">{}</div>"#, bg));
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
                    WebInputType::NodeFeature(feature.id.clone()),
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

            let pref = PenguinPinRef {
                node_id: id,
                id: pin_id.clone(),
                is_output: false,
                r#type: pin_defn.r#type,
            };

            inputs.insert(
                pin_id.clone(),
                WebPin::new(
                    &inputs_el,
                    &mut inner,
                    pref,
                    pin_defn.clone(),
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

            let pref = PenguinPinRef {
                node_id: id,
                id: pin_id.clone(),
                is_output: true,
                r#type: pin_defn.r#type,
            };

            outputs.insert(
                pin_id.clone(),
                WebPin::new(
                    &outputs_el,
                    &mut inner,
                    pref,
                    pin_defn.clone(),
                    connections.unwrap_or_default(),
                )?,
            );
        }

        // TODO Variadic controls

        let listeners = ListenerBuilder::new(&el, EventTarget::Node(id))
            .add_mousedown()?
            .add_contextmenu()?
            .build();

        let me = WebNode {
            inner,
            id,
            defn,
            el,
            listeners,
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

    pub fn pin(&self, pref: &PenguinPinRef) -> Option<&WebPin> {
        if pref.is_output {
            self.outputs.get(&pref.id)
        } else {
            self.inputs.get(&pref.id)
        }
    }

    pub fn update_input_value(&self, r#type: &WebInputType, value: &str) -> Result<(), JsValue> {
        match r#type {
            WebInputType::Pin(pin_id) => {
                if let Some(pin) = self.inputs.get(pin_id)
                    && let Some(input) = &pin.input_el
                {
                    input.update_value(value)?;
                }
            }
            WebInputType::NodeFeature(feature_id) => {
                if let Some(input) = self.input_feature_els.get(feature_id) {
                    input.update_value(value)?;
                }
            }
        }
        Ok(())
    }

    pub fn update_input_size(
        &self,
        r#type: &WebInputType,
        size: (i32, i32),
    ) -> Result<(), JsValue> {
        match r#type {
            WebInputType::Pin(pin_id) => {
                if let Some(pin) = self.inputs.get(pin_id)
                    && let Some(input) = &pin.input_el
                {
                    input.update_size(size)?;
                }
            }
            WebInputType::NodeFeature(feature_id) => {
                if let Some(input) = self.input_feature_els.get(feature_id) {
                    input.update_size(size)?;
                }
            }
        }
        Ok(())
    }
}
