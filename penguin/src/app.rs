use crate::context::ContextMenu;
use crate::ffi;
use crate::graph::WebGraph;
use crate::interaction::{Interaction, WiringState};
use crate::viewport::{ClientBox, ClientPoint, PenguinVector, Viewport, mouse_client_pos};
use igloo_interface::graph::{PenguinGraph, PenguinNode, PenguinNodeID};
use igloo_interface::{
    PenguinNodeDefn, PenguinNodeDefnRef, PenguinPinID, PenguinPinType, PenguinRegistry,
};
use std::collections::HashMap;
use std::{any::Any, cell::RefCell};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement, KeyboardEvent, MouseEvent, WheelEvent};

thread_local! {
    pub static APP: RefCell<Option<PenguinApp>> = const { RefCell::new(None) };
}

#[derive(Debug)]
pub struct PenguinApp {
    pub registry: PenguinRegistry,
    pub penguin_el: HtmlElement,
    pub box_el: Element,
    pub graph: WebGraph,
    pub viewport: Viewport,
    interaction: Interaction,
    context: ContextMenu,
    pub closures: [Box<dyn Any>; 6],
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

        let box_el = document.create_element("div")?;
        box_el.set_id("penguin-selection-box");
        box_el.set_attribute("style", "display: none;")?;
        penguin_el.append_child(&box_el)?;

        let registry = PenguinRegistry::default();

        Ok(PenguinApp {
            closures: ffi::init(&penguin_el),
            context: ContextMenu::new(&registry, &penguin_el)?,
            registry,
            penguin_el: penguin_el.clone(),
            graph: WebGraph::new(&viewport_el)?,
            viewport: Viewport::new(penguin_el, viewport_el, grid_svg)?,
            interaction: Interaction::default(),
            box_el,
        })
    }

    pub fn load(&mut self, graph: PenguinGraph) -> Result<(), JsValue> {
        self.graph.load(&self.registry, graph)
    }

    pub fn penguin_graph(&self) -> PenguinGraph {
        self.graph.penguin_graph()
    }

    pub fn set_interaction(&mut self, interaction: Interaction) {
        match self.interaction {
            Interaction::Panning {
                start_pos: a,
                last_pos: b,
            } => {
                let a = a.cast::<f64>();
                let b = b.cast::<f64>();
                if a.distance_to(b) < 10. {
                    self.graph.clear_selection();
                }
            }
            Interaction::Wiring(_) => {
                self.graph.hide_wiring();
                if !matches!(interaction, Interaction::Context { .. }) {
                    self.graph.temp_wire.hide();
                }
            }
            Interaction::BoxSelecting {
                start_pos: a,
                last_pos: b,
                append,
            } => {
                // TODO complete box selection
                self.box_el.set_attribute("style", "display: none;");

                if a.cast::<f64>().distance_to(b.cast::<f64>()) < 10. {
                    self.context.show_search(&self.registry, &b, &None);
                    self.interaction = Interaction::Context {
                        wpos: self.viewport.client_to_world(b),
                        ws: None,
                    };
                    return;
                } else {
                    self.graph.box_select(
                        ClientBox::new(
                            ClientPoint::new(i32::min(a.x, b.x), i32::min(a.y, b.y)),
                            ClientPoint::new(i32::max(a.x, b.x), i32::max(a.y, b.y)),
                        ),
                        self.viewport.client_to_world_transform(),
                        append,
                    );
                }
            }
            Interaction::Context { .. } => {
                self.context.hide();
                self.graph.temp_wire.hide();
            }
            _ => {}
        }

        // TODO append node moves to history

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

    pub fn place_wire(
        &mut self,
        ws: WiringState,
        end_node: PenguinNodeID,
        end_pin: PenguinPinID,
        end_type: PenguinPinType,
        end_is_out: bool,
    ) -> Result<(), JsValue> {
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
            )?;
        } else {
            self.graph.add_wire(
                ws.start_node,
                ws.start_pin,
                ws.wire_type,
                end_node,
                end_pin,
                end_type,
            )?;
        }

        self.set_interaction(Interaction::Idle);

        Ok(())
    }

    pub fn onmousedown(&mut self, e: MouseEvent) {
        self.penguin_el.focus();

        if e.button() == 0 {
            let client_pos = mouse_client_pos(&e);

            self.set_interaction(Interaction::Panning {
                start_pos: client_pos,
                last_pos: client_pos,
            });
        }
    }

    pub fn onmouseup(&mut self, e: MouseEvent) {
        let cpos = mouse_client_pos(&e);
        let wpos = self.viewport.client_to_world(cpos);

        if let Interaction::Wiring(ws) = &self.interaction {
            let ws = Some(ws.clone());
            self.context.show_search(&self.registry, &cpos, &ws);
            self.set_interaction(Interaction::Context { wpos, ws });
        } else {
            self.set_interaction(Interaction::Idle);
        }
    }

    pub fn oncontextmenu(&mut self, e: MouseEvent) {
        e.prevent_default();

        let cpos = mouse_client_pos(&e);

        self.set_interaction(Interaction::BoxSelecting {
            start_pos: cpos,
            last_pos: cpos,
            append: e.ctrl_key() || e.shift_key(),
        });
    }

    pub fn onmousemove(&mut self, e: MouseEvent) {
        let mouse_pos = mouse_client_pos(&e);

        match &self.interaction {
            Interaction::Panning {
                last_pos,
                start_pos,
            } => {
                let delta = (mouse_pos - *last_pos).cast::<f64>();

                // Client delta == Penguin delta
                self.viewport.pan_by(PenguinVector::new(delta.x, delta.y));

                self.interaction = Interaction::Panning {
                    start_pos: *start_pos,
                    last_pos: mouse_pos,
                };
            }
            Interaction::Dragging {
                primary_node,
                primary_node_pos,
                start_pos: start_world,
                node_poses,
                ..
            } => {
                let current_penguin = self.viewport.client_to_penguin(mouse_pos);
                let current_world = self.viewport.penguin_to_world(current_penguin);

                let delta = current_world - *start_world;

                let mut primary_new_pos = *primary_node_pos + delta;
                let gs = self.viewport.grid.settings();
                if gs.snap {
                    primary_new_pos.x = f64::round(primary_new_pos.x / gs.size) * gs.size;
                    primary_new_pos.y = f64::round(primary_new_pos.y / gs.size) * gs.size;
                }

                let actual_delta = primary_new_pos - *primary_node_pos;

                for (node_id, initial_pos) in node_poses {
                    let new_pos = *initial_pos + actual_delta;
                    self.graph.move_node(node_id, new_pos);
                }
            }
            Interaction::Wiring(_) => {
                self.graph
                    .temp_wire
                    .update(self.viewport.client_to_world(mouse_pos));
            }
            Interaction::BoxSelecting {
                start_pos, append, ..
            } => {
                let left = i32::min(start_pos.x, mouse_pos.x);
                let top = i32::min(start_pos.y, mouse_pos.y);
                let width = i32::abs(start_pos.x - mouse_pos.x);
                let height = i32::abs(start_pos.y - mouse_pos.y);

                let style = format!(
                    "display: block; left: {left}px; top: {top}px; width: {width}px; height: {height}px;"
                );

                self.box_el.set_attribute("style", &style);

                self.interaction = Interaction::BoxSelecting {
                    start_pos: *start_pos,
                    last_pos: mouse_pos,
                    append: *append,
                };
            }
            _ => {}
        }
    }

    pub fn onwheel(&mut self, e: WheelEvent) -> Result<(), JsValue> {
        e.prevent_default();

        self.penguin_el.focus()?;

        let client_pos = mouse_client_pos(&e);
        let penguin_pos = self.viewport.client_to_penguin(client_pos);

        let delta = if e.delta_y() > 0.0 { 0.9 } else { 1.1 };
        self.viewport.zoom_at(penguin_pos, delta)
    }

    pub fn onkeydown(&mut self, e: KeyboardEvent) {
        match e.key().as_str() {
            "Escape" => {
                self.graph.clear_selection();
            }
            "Backspace" | "Delete" => {
                self.graph.delete_selection();
            }
            _ => {}
        }

        //
    }

    pub fn context_add_node(
        &mut self,
        defn: PenguinNodeDefn,
        defn_ref: PenguinNodeDefnRef,
        close: bool,
    ) -> Result<(), JsValue> {
        let Interaction::Context { wpos, ws } = &self.interaction else {
            return Ok(());
        };

        let node_id = self.graph.place_node(
            &self.registry,
            PenguinNode {
                defn_ref,
                x: wpos.x,
                y: wpos.y,
                input_cfg_values: HashMap::with_capacity(defn.num_input_configs()),
                input_pin_values: HashMap::with_capacity(defn.inputs.len()),
            },
        )?;

        if let Some(ws) = ws
            && let Some((pin_id, pin_defn)) = ws.find_compatible(&defn)
        {
            self.place_wire(
                ws.clone(),
                node_id,
                pin_id.clone(),
                pin_defn.r#type,
                !ws.is_output,
            )?;
        }

        if close {
            self.set_interaction(Interaction::Idle);
        }

        Ok(())
    }

    pub fn interaction(&self) -> &Interaction {
        &self.interaction
    }

    pub fn start_dragging(
        &mut self,
        primary_node: PenguinNodeID,
        start_pos: ClientPoint,
    ) -> Result<(), JsValue> {
        self.set_interaction(Interaction::Dragging {
            primary_node_pos: self.graph.get_node_pos(&primary_node)?,
            primary_node,
            start_pos: self.viewport.client_to_world(start_pos),
            node_poses: self.graph.selection_poses()?,
        });
        Ok(())
    }
}
