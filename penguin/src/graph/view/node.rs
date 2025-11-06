use igloo_interface::penguin::{
    NodeInputFeatureID, PenguinNodeDefn, PenguinPinID, PenguinPinRef, PenguinRegistry,
    graph::{PenguinNode, PenguinNodeID, PenguinWire, PenguinWireID},
};
use indexmap::IndexMap;
use std::collections::HashMap;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement, MouseEvent};

use crate::{
    app::event::{EventTarget, ListenerBuilder, Listeners, document},
    graph::{
        input::{WebInput, WebInputType},
        pin::{self, WebPin},
    },
    viewport::{ClientBox, ClientPoint, WorldPoint},
};

#[derive(Debug)]
pub struct WebNode {
    pub inner: PenguinNode,
    id: PenguinNodeID,
    defn: PenguinNodeDefn,
    el: HtmlElement,
    listeners: Vec<Listeners>,
    pub inputs: IndexMap<PenguinPinID, WebPin>,
    pub outputs: IndexMap<PenguinPinID, WebPin>,
    input_feature_els: IndexMap<NodeInputFeatureID, WebInput>,
    section: Option<HtmlElement>,
}

impl Drop for WebNode {
    fn drop(&mut self) {
        self.el.remove();
    }
}

fn make(
    parent: &Element,
    defn: &PenguinNodeDefn,
) -> Result<(HtmlElement, Element, Element), JsValue> {
    let document = document();

    let el = document.create_element("div")?.dyn_into::<HtmlElement>()?;
    el.set_class_name("penguin-node");
    parent.append_child(&el)?;

    if let Some(title_bar) = &defn.title_bar {
        let title_el = document.create_element("div")?;
        title_el.set_class_name("penguin-node-title");
        title_el.set_inner_html(title_bar);
        el.append_child(&title_el)?;
    }

    if defn.icon_bg {
        let icon_el = document.create_element("div")?;
        icon_el.set_class_name("penguin-node-bg");
        icon_el.set_inner_html(&defn.icon);
        el.append_child(&icon_el)?;
    }

    if defn.is_reroute {
        el.set_attribute("data-is-reroute", "true")?;
        let circle = document.create_element("div")?;
        circle.set_class_name("penguin-reroute-circle");
        el.append_child(&circle)?;
    }

    let inputs_el = document.create_element("div")?;
    inputs_el.set_class_name("penguin-node-inputs");
    el.append_child(&inputs_el)?;

    let outputs_el = document.create_element("div")?;
    outputs_el.set_class_name("penguin-node-outputs");
    el.append_child(&outputs_el)?;

    Ok((el, inputs_el, outputs_el))
}

/// used for search
pub fn make_dummy(parent: &Element, defn: &PenguinNodeDefn) -> Result<(), JsValue> {
    let (el, inputs_el, outputs_el) = make(parent, defn)?;
    el.set_class_name("penguin-node penguin-dummy-node");

    for (pin_id, pin_defn) in &defn.inputs {
        pin::make(&inputs_el, pin_defn, pin_id, false)?;
    }

    for (pin_id, pin_defn) in &defn.outputs {
        pin::make(&outputs_el, pin_defn, pin_id, true)?;
    }

    Ok(())
}

impl WebNode {
    pub fn new(
        parent: &Element,
        registry: &PenguinRegistry,
        wires: Option<&HashMap<PenguinWireID, PenguinWire>>,
        mut inner: PenguinNode,
        id: PenguinNodeID,
    ) -> Result<Self, JsValue> {
        let document = document();

        let defn = registry
            .get_defn(&inner.defn_ref)
            .cloned()
            .ok_or(JsValue::from_str(&format!(
                "Unknown Node Definition {}",
                inner.defn_ref
            )))?;

        let (el, inputs_el, outputs_el) = make(parent, &defn)?;
        el.set_attribute("data-defn", &inner.defn_ref.to_string())?;

        // input pins
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
                    defn.is_reroute,
                )?,
            );
        }

        if defn.is_reroute {
            let s = el.style();
            let defn = &defn.outputs[0];
            let color = defn.r#type.color();
            s.set_property("background-color", color)?;
            s.set_property("border-color", color)?;
        }

        // output pins
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
                    defn.is_reroute,
                )?,
            );
        }

        // input feature
        let mut input_feature_els = IndexMap::with_capacity(defn.input_features.len());
        if !defn.input_features.is_empty() {
            let content_el = document.create_element("div")?;
            content_el.set_class_name("penguin-node-content");
            el.append_child(&content_el)?;

            for feature in &defn.input_features {
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

        let mut listeners = Vec::with_capacity(3);

        if let Some(vf) = &defn.variadic_feature {
            let controls = document.create_element("div")?;
            controls.set_class_name("penguin-variadic-controls");
            el.append_child(&controls)?;

            if let Some(prev) = &vf.prev {
                let btn = document.create_element("button")?;
                btn.set_class_name("penguin-variadic-button");
                btn.set_inner_html("-");
                controls.append_child(&btn)?;
                listeners.push(
                    ListenerBuilder::new(&btn, EventTarget::NodeVariadic(id, prev.clone()))
                        .add_mousedown()?
                        .add_mouseclick()?
                        .build(),
                );
            }

            if let Some(next) = &vf.next {
                let btn = document.create_element("button")?;
                btn.set_class_name("penguin-variadic-button");
                btn.set_inner_html("+");
                controls.append_child(&btn)?;
                listeners.push(
                    ListenerBuilder::new(&btn, EventTarget::NodeVariadic(id, next.clone()))
                        .add_mousedown()?
                        .add_mouseclick()?
                        .build(),
                );
            }
        }

        let section = if defn.is_section {
            inputs_el.remove();
            outputs_el.remove();

            el.set_attribute("data-is-section", "true")?;

            let section = document.create_element("div")?.dyn_into::<HtmlElement>()?;
            section.set_class_name("penguin-node-section");
            el.append_child(&section)?;

            let (width, height) = match inner.size {
                Some(size) => size,
                None => {
                    let size = (350, 100);
                    inner.size = Some(size);
                    size
                }
            };

            let style = section.style();
            style.set_property("width", &format!("{width}px"))?;
            style.set_property("height", &format!("{height}px"))?;

            let section_1 = section.clone();
            let mut l = ListenerBuilder::new(&section, EventTarget::Global)
                .add_contextmenu()?
                .add_mousedown_conditional(move |e: &MouseEvent| {
                    let rect = section_1.get_bounding_client_rect();
                    let x = e.client_x() as f64 - rect.left();
                    let y = e.client_y() as f64 - rect.top();
                    !(x > rect.width() - 20.0 && y > rect.height() - 20.0)
                })?
                .build();

            l.add_resize(&section, section.clone(), EventTarget::Node(id))?;
            listeners.push(l);

            Some(section)
        } else {
            None
        };

        listeners.push(
            ListenerBuilder::new(&el, EventTarget::Node(id))
                .add_mousedown()?
                .add_contextmenu()?
                .build(),
        );

        let me = WebNode {
            inner,
            id,
            defn,
            el,
            listeners,
            inputs,
            outputs,
            input_feature_els,
            section,
        };

        me.update_transform()?;

        Ok(me)
    }

    pub fn update_transform(&self) -> Result<(), JsValue> {
        let translate = format!("translate({}px, {}px)", self.inner.x, self.inner.y);
        self.el.style().set_property("transform", &translate)
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

    pub fn update_size(&self, size: (i32, i32)) -> Result<(), JsValue> {
        if let Some(section) = &self.section {
            let style = section.style();
            style.set_property("width", &format!("{}px", size.0))?;
            style.set_property("height", &format!("{}px", size.1))?;
        }
        Ok(())
    }
}
