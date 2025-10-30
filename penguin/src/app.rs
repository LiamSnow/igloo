use crate::ffi;
use crate::graph::WebGraph;
use crate::interaction::{Interaction, WiringState};
use crate::viewport::{ClientPoint, PenguinVector, Viewport, mouse_client_pos};
use igloo_interface::graph::{PenguinGraph, PenguinNodeID};
use igloo_interface::{PenguinPinID, PenguinPinType, PenguinRegistry};
use std::mem;
use std::{any::Any, cell::RefCell};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlElement, MouseEvent, WheelEvent};

thread_local! {
    pub static APP: RefCell<Option<PenguinApp>> = const { RefCell::new(None) };
}

#[derive(Debug)]
pub struct PenguinApp {
    pub registry: PenguinRegistry,

    pub penguin_el: HtmlElement,

    pub graph: WebGraph,
    pub viewport: Viewport,
    interaction: Interaction,

    pub closures: Vec<Box<dyn Any>>,
}

impl PenguinApp {
    pub fn new() -> Result<Self, JsValue> {
        let document = ffi::document();

        let penguin_el = document
            .get_element_by_id("penguin")
            .ok_or(JsValue::from_str("Cannot find #penguin element"))?
            .dyn_into::<HtmlElement>()?;

        penguin_el.set_tab_index(0);

        let grid_svg = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "svg")?;
        grid_svg.set_id("penguin-grid");
        penguin_el.append_child(&grid_svg)?;

        let viewport_el = document.create_element("div")?;
        viewport_el.set_id("penguin-viewport");
        penguin_el.append_child(&viewport_el)?;

        Ok(PenguinApp {
            closures: ffi::init(&penguin_el),
            registry: PenguinRegistry::default(),
            penguin_el: penguin_el.clone(),
            graph: WebGraph::new(&viewport_el)?,
            viewport: Viewport::new(penguin_el, viewport_el, grid_svg)?,
            interaction: Interaction::default(),
        })
    }

    pub fn load(&mut self, graph: PenguinGraph) -> Result<(), JsValue> {
        self.graph.load(&self.registry, graph)
    }

    pub fn penguin_graph(&self) -> PenguinGraph {
        self.graph.penguin_graph()
    }

    pub fn set_interaction(&mut self, interaction: Interaction) {
        // TODO append node moves to history

        if matches!(self.interaction, Interaction::Wiring(_)) {
            self.graph.temp_wire.hide();
            self.graph.hide_wiring();
        }

        self.interaction = interaction;
    }

    pub fn start_wiring(
        &mut self,
        e: MouseEvent,
        start_hitbox: &HtmlElement,
        start_node: PenguinNodeID,
        start_pin: PenguinPinID,
        is_output: bool,
        wire_type: PenguinPinType,
    ) -> Result<(), JsValue> {
        self.graph.temp_wire.show(
            start_hitbox,
            self.graph.get_node_pos(&start_node)?,
            wire_type,
            is_output,
        )?;

        self.graph
            .temp_wire
            .update(self.viewport.client_to_world(mouse_client_pos(&e)))?;

        let ws = WiringState {
            start_node,
            start_pin,
            is_output,
            wire_type,
        };

        self.graph.show_wiring(&ws);

        self.set_interaction(Interaction::Wiring(ws));

        Ok(())
    }

    pub fn stop_wiring(
        &mut self,
        end_node: PenguinNodeID,
        end_pin: PenguinPinID,
        end_type: PenguinPinType,
        end_is_out: bool,
    ) -> Result<(), JsValue> {
        let Interaction::Wiring(ws) = mem::take(&mut self.interaction) else {
            return Ok(());
        };

        self.graph.temp_wire.hide()?;
        self.graph.hide_wiring();

        if !ws.is_valid_end(end_node, end_type, end_is_out) {
            return Ok(());
        }

        if end_is_out {
            self.graph.add_wire(
                end_node,
                end_pin,
                end_type,
                ws.start_node,
                ws.start_pin,
                ws.wire_type,
            )
        } else {
            self.graph.add_wire(
                ws.start_node,
                ws.start_pin,
                ws.wire_type,
                end_node,
                end_pin,
                end_type,
            )
        }
    }

    pub fn onmousedown(&mut self, e: MouseEvent) {
        if e.button() == 0 {
            let client_pos = mouse_client_pos(&e);

            self.set_interaction(Interaction::Panning {
                start_pos: client_pos,
                last_pos: client_pos,
            });
        }
    }

    pub fn onmouseup(&mut self, _e: MouseEvent) {
        self.set_interaction(Interaction::Idle);
    }

    pub fn oncontextmenu(&mut self, e: MouseEvent) {}

    pub fn onmousemove(&mut self, e: MouseEvent) {
        let client_pos = mouse_client_pos(&e);

        match &self.interaction {
            Interaction::Panning {
                last_pos,
                start_pos,
            } => {
                let delta = (client_pos - *last_pos).cast::<f64>();

                // Client delta == Penguin delta (just diff origins)
                self.viewport.pan_by(PenguinVector::new(delta.x, delta.y));

                self.set_interaction(Interaction::Panning {
                    start_pos: *start_pos,
                    last_pos: client_pos,
                });
            }
            Interaction::Dragging {
                node_id,
                start_client_pos,
                start_node_pos,
            } => {
                let start_penguin = self.viewport.client_to_penguin(*start_client_pos);
                let current_penguin = self.viewport.client_to_penguin(client_pos);

                let start_world = self.viewport.penguin_to_world(start_penguin);
                let current_world = self.viewport.penguin_to_world(current_penguin);

                let delta = current_world - start_world;
                let new_pos = *start_node_pos + delta;

                self.graph.move_node(node_id, new_pos);
            }
            Interaction::Wiring(_) => {
                self.graph
                    .temp_wire
                    .update(self.viewport.client_to_world(client_pos));
            }
            _ => {}
        }
    }

    pub fn onwheel(&mut self, e: WheelEvent) {
        e.prevent_default();

        let client_pos = mouse_client_pos(&e);
        let penguin_pos = self.viewport.client_to_penguin(client_pos);

        let delta = if e.delta_y() > 0.0 { 0.9 } else { 1.1 };
        self.viewport.zoom_at(penguin_pos, delta);
    }
}
