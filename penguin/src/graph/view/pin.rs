use crate::{
    dom::{self, Div, Polygon, events::EventTarget, node::DomNode},
    graph::input::{WebInput, WebInputType},
};
use either::Either;
use igloo_interface::penguin::{
    NodeInputType, PenguinPinDefn, PenguinPinID, PenguinPinRef, PenguinPinType,
    graph::{PenguinNode, PenguinWireID},
};

type PinElement = Either<DomNode<Polygon>, DomNode<Div>>;

#[derive(Debug)]
pub struct WebPin {
    pref: PenguinPinRef,
    defn: PenguinPinDefn,
    #[allow(dead_code)]
    wrapper: DomNode<Div>,
    pub hitbox: DomNode<Div>,
    /// flow=polygon, value=div
    pin_el: PinElement,
    pub(super) input_el: Option<WebInput>,
    connections: Vec<PenguinWireID>,
}

const UNCONNECTED_COLOR: &str = "#111";

pub fn make<T>(
    parent: &DomNode<T>,
    defn: &PenguinPinDefn,
    id: &PenguinPinID,
    is_output: bool,
) -> (DomNode<Div>, DomNode<Div>, PinElement) {
    let wrapper = dom::div()
        .class(if is_output {
            "penguin-pin-wrapper output"
        } else {
            "penguin-pin-wrapper input"
        })
        .mount(parent);

    let hitbox = dom::div().class("penguin-pin-hitbox").mount(&wrapper);

    let pin_el = match defn.r#type {
        PenguinPinType::Flow => {
            let svg = dom::svg()
                .attr("class", "penguin-pin flow")
                .width(16.)
                .height(16.)
                .viewbox(0., 0., 16., 16.)
                .mount(&hitbox);

            Either::Left(
                dom::polygon()
                    .points("1,1 12,1 15,8 12,15 1,15")
                    .fill(UNCONNECTED_COLOR)
                    .stroke(defn.r#type.color())
                    .stroke_width(2.)
                    .mount(&svg),
            )
        }
        PenguinPinType::Value(vt) => Either::Right(
            dom::div()
                .class("penguin-pin value")
                .style("border-color", vt.color())
                .mount(&hitbox),
        ),
    };

    if !defn.hide_name {
        dom::div()
            .class("penguin-pin-name")
            .text(&id.0)
            .mount(&wrapper);
    }

    (wrapper, hitbox, pin_el)
}

impl WebPin {
    pub fn new<T>(
        parent: &DomNode<T>,
        node: &mut PenguinNode,
        pref: PenguinPinRef,
        defn: PenguinPinDefn,
        connections: Vec<PenguinWireID>,
        is_reroute: bool,
    ) -> Self {
        let (mut wrapper, mut hitbox, pin_el) = make(parent, &defn, &pref.id, pref.is_output);

        wrapper.remove_on_drop();

        hitbox.event_target(EventTarget::Pin(pref.clone()));
        hitbox.listen_mouseup();
        if !is_reroute {
            hitbox.listen_mousedown();
            hitbox.listen_contextmenu();
        }

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
                NodeInputType::Input,
                &pin_value.value.to_string(),
                pin_value.size,
            ));
        }

        let me = Self {
            pref,
            defn,
            wrapper,
            hitbox,
            pin_el,
            input_el,
            connections,
        };

        me.update_fill();

        me
    }

    pub fn connections(&self) -> &[PenguinWireID] {
        &self.connections
    }

    /// WARN: make sure to update node wires
    pub fn remove_connection(&mut self, wire_id: PenguinWireID) {
        self.connections.retain(|&id| id != wire_id);
        self.update_fill();
    }

    // pub fn take_connections(&mut self) -> Vec<PenguinWireID> {
    //     let mut o = Vec::new();
    //     mem::swap(&mut self.connections, &mut o);
    //     self.update_fill();
    //     o
    // }

    pub fn add_connection(&mut self, connection: PenguinWireID) {
        self.connections.push(connection);
        self.update_fill();
    }

    fn update_fill(&self) {
        let connected = !self.connections.is_empty();

        let color = if connected {
            self.defn.r#type.color()
        } else {
            UNCONNECTED_COLOR
        };

        match &self.pin_el {
            Either::Left(polygon) => {
                polygon.set_fill(color);
            }
            Either::Right(div) => {
                div.set_style("background-color", color);
            }
        }

        if let Some(input) = &self.input_el {
            input.set_visible(!connected);
        }
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

        self.hitbox.set_class(class);
    }

    pub fn hide_wiring(&mut self) {
        self.hitbox.set_class("penguin-pin-hitbox");
    }
}
