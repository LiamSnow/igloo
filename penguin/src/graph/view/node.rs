use crate::{
    dom::{self, Div, events::EventTarget, node::DomNode},
    graph::{
        input::{WebInput, WebInputType},
        pin::{self, WebPin},
    },
    viewport::{ClientSpace, WorldPoint},
};
use euclid::Box2D;
use igloo_interface::penguin::{
    NodeInputFeatureID, PenguinNodeDefn, PenguinNodeDefnRef, PenguinPinID, PenguinPinRef,
    PenguinRegistry,
    graph::{PenguinNode, PenguinNodeID, PenguinWire, PenguinWireID},
};
use indexmap::IndexMap;
use std::collections::HashMap;
use web_sys::MouseEvent;

#[derive(Debug)]
pub struct WebNode {
    pub inner: PenguinNode,
    // id: PenguinNodeID,
    // defn: PenguinNodeDefn,
    el: DomNode<Div>,
    pub inputs: IndexMap<PenguinPinID, WebPin>,
    pub outputs: IndexMap<PenguinPinID, WebPin>,
    input_feature_els: IndexMap<NodeInputFeatureID, WebInput>,
    section: Option<DomNode<Div>>,
}

fn make<T>(
    parent: &DomNode<T>,
    defn: &PenguinNodeDefn,
    defn_ref: &PenguinNodeDefnRef,
) -> (DomNode<Div>, DomNode<Div>, DomNode<Div>) {
    let el = dom::div()
        .class("penguin-node")
        .attr("data-defn", &defn_ref.to_string())
        .mount(parent);

    if let Some(title_bar) = &defn.title_bar {
        dom::div()
            .class("penguin-node-title")
            .text(title_bar)
            .mount(&el);
    }

    if defn.icon_bg {
        dom::div()
            .class("penguin-node-bg")
            .text(&defn.icon)
            .mount(&el);
    }

    if defn.is_reroute {
        el.set_attr("data-is-reroute", "true");
    }

    let inputs_el = dom::div().class("penguin-node-inputs").mount(&el);
    let outputs_el = dom::div().class("penguin-node-outputs").mount(&el);

    (el, inputs_el, outputs_el)
}

/// used for search
pub fn make_dummy<T>(parent: &DomNode<T>, defn: &PenguinNodeDefn, defn_ref: &PenguinNodeDefnRef) {
    let (el, inputs_el, outputs_el) = make(parent, defn, defn_ref);
    el.set_class("penguin-node penguin-dummy-node");

    for (pin_id, pin_defn) in &defn.inputs {
        pin::make(&inputs_el, pin_defn, pin_id, false);
    }

    for (pin_id, pin_defn) in &defn.outputs {
        pin::make(&outputs_el, pin_defn, pin_id, true);
    }
}

impl WebNode {
    pub fn new<T>(
        parent: &DomNode<T>,
        registry: &PenguinRegistry,
        wires: Option<&HashMap<PenguinWireID, PenguinWire>>,
        mut inner: PenguinNode,
        id: PenguinNodeID,
    ) -> Self {
        let defn = registry
            .get_defn(&inner.defn_ref)
            .cloned()
            .unwrap_or_else(|| panic!("Unknown Node Definition {}", inner.defn_ref));

        let (mut el, inputs_el, outputs_el) = make(parent, &defn, &inner.defn_ref);

        el.remove_on_drop();

        el.event_target(EventTarget::Node(id));
        el.listen_mousedown();
        el.listen_contextmenu();

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
                ),
            );
        }

        if defn.is_reroute {
            let defn = &defn.outputs[0];
            let color = defn.r#type.color();
            el.set_style("background-color", color);
            el.set_style("border-color", color);
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
                ),
            );
        }

        // input feature
        let mut input_feature_els = IndexMap::with_capacity(defn.input_features.len());
        if !defn.input_features.is_empty() {
            let content_el = dom::div().class("penguin-node-content").mount(&el);

            for feature in &defn.input_features {
                inner.ensure_input_feature_value(feature);
                let feature_value = inner.input_feature_values.get(&feature.id).unwrap();

                let input = WebInput::new(
                    &content_el,
                    id,
                    WebInputType::NodeFeature(feature.id.clone()),
                    feature.value_type,
                    feature.input_type.clone(),
                    &feature_value.value.to_string(),
                    feature_value.size,
                );

                input_feature_els.insert(feature.id.clone(), input);
            }
        }

        if let Some(vf) = &defn.variadic_feature {
            let controls = dom::div().class("penguin-variadic-controls").mount(&el);

            if let Some(prev) = &vf.prev {
                dom::button()
                    .class("penguin-variadic-button")
                    .text("-")
                    .event_target(EventTarget::NodeVariadic(id, prev.clone()))
                    .listen_mousedown()
                    .listen_click()
                    .mount(&controls);
            }

            if let Some(next) = &vf.next {
                dom::button()
                    .class("penguin-variadic-button")
                    .text("+")
                    .event_target(EventTarget::NodeVariadic(id, next.clone()))
                    .listen_mousedown()
                    .listen_click()
                    .mount(&controls);
            }
        }

        let section = if defn.is_section {
            inputs_el.remove();
            outputs_el.remove();

            el.set_attr("data-is-section", "true");

            let (width, height) = match inner.size {
                Some(size) => size,
                None => {
                    let size = (350, 100);
                    inner.size = Some(size);
                    size
                }
            };

            let mut section = dom::div()
                .class("penguin-node-section")
                .width(width as f64)
                .height(height as f64)
                .event_target(EventTarget::Global)
                .listen_contextmenu()
                .mount(&el);

            let section_el = section.element.clone();
            section.listen_resize(&section_el);
            section.listen_mousedown_conditional(move |e: &MouseEvent| {
                let rect = section_el.get_bounding_client_rect();
                let x = e.client_x() as f64 - rect.left();
                let y = e.client_y() as f64 - rect.top();
                !(x > rect.width() - 20.0 && y > rect.height() - 20.0)
            });

            section.event_target(EventTarget::Node(id));

            Some(section)
        } else {
            None
        };

        let me = WebNode {
            inner,
            // id,
            // defn,
            el,
            inputs,
            outputs,
            input_feature_els,
            section,
        };

        me.update_transform();

        me
    }

    pub fn update_transform(&self) {
        self.el.translate(self.inner.x, self.inner.y);
    }

    pub fn set_pos(&mut self, pos: WorldPoint) {
        self.inner.x = pos.x;
        self.inner.y = pos.y;
        self.update_transform();
    }

    pub fn pos(&self) -> WorldPoint {
        WorldPoint::new(self.inner.x, self.inner.y)
    }

    pub fn select(&self, selected: bool) {
        if selected {
            self.el.set_class("penguin-node selected");
        } else {
            self.el.set_class("penguin-node");
        }
    }

    pub fn client_box(&self) -> Box2D<f64, ClientSpace> {
        self.el.client_box()
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

    pub fn update_input_value(&self, r#type: &WebInputType, value: &str) {
        match r#type {
            WebInputType::Pin(pin_id) => {
                if let Some(pin) = self.inputs.get(pin_id)
                    && let Some(input) = &pin.input_el
                {
                    input.update_value(value);
                }
            }
            WebInputType::NodeFeature(feature_id) => {
                if let Some(input) = self.input_feature_els.get(feature_id) {
                    input.update_value(value);
                }
            }
        }
    }

    pub fn update_input_size(&self, r#type: &WebInputType, size: (i32, i32)) {
        match r#type {
            WebInputType::Pin(pin_id) => {
                if let Some(pin) = self.inputs.get(pin_id)
                    && let Some(input) = &pin.input_el
                {
                    input.update_size(size);
                }
            }
            WebInputType::NodeFeature(feature_id) => {
                if let Some(input) = self.input_feature_els.get(feature_id) {
                    input.update_size(size);
                }
            }
        }
    }

    pub fn update_size(&self, size: (i32, i32)) {
        if let Some(section) = &self.section {
            section.set_width(size.0 as f64);
            section.set_height(size.1 as f64);
        }
    }
}
