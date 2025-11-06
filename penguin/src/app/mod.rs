use crate::{
    app::{
        event::{Clientable, Event, EventTarget, EventValue, ListenerBuilder, Listeners, document},
        mode::Mode,
    },
    graph::WebGraph,
    menu::Menu,
    viewport::{ClientPoint, Viewport},
};
use igloo_interface::penguin::{PenguinRegistry, graph::PenguinGraph};
use std::cell::RefCell;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement};

pub mod event;
pub mod mode;

thread_local! {
    pub static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

#[derive(Debug)]
pub struct App {
    pub(self) el: HtmlElement,

    pub(self) mode: Mode,

    pub(self) graph: WebGraph,
    pub(self) viewport: Viewport,
    pub(self) menu: Menu,
    pub(self) box_el: Element,

    pub(self) mouse_pos: ClientPoint,
    pub(self) listeners: [Listeners; 2],
}

impl App {
    pub fn init() -> Result<(), JsValue> {
        let document = document();

        let el = document
            .get_element_by_id("penguin")
            .ok_or(JsValue::from_str("Cannot find #penguin element"))?
            .dyn_into::<HtmlElement>()?;

        el.set_tab_index(0);

        let listeners = [
            ListenerBuilder::new(&document, EventTarget::Global)
                .add_mousemove()?
                .add_mouseup()?
                .build(),
            ListenerBuilder::new(&el, EventTarget::Global)
                .add_mousedown()?
                .add_contextmenu()?
                .add_wheel()?
                .add_keydown()?
                .add_copy()?
                .add_paste()?
                .add_cut()?
                .build(),
        ];

        let grid_svg = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "svg")?;
        grid_svg.set_id("penguin-grid");
        el.append_child(&grid_svg)?;

        let viewport_el = document.create_element("div")?;
        viewport_el.set_id("penguin-viewport");
        el.append_child(&viewport_el)?;

        let box_el = document.create_element("div")?;
        box_el.set_id("penguin-selection-box");
        box_el.set_attribute("style", "display: none;")?;
        el.append_child(&box_el)?;

        let registry = PenguinRegistry::default();
        let menu = Menu::new(&registry, &el)?;
        let mut graph = WebGraph::new(registry, &viewport_el)?;
        let viewport = Viewport::new(el.clone(), viewport_el, grid_svg)?;
        graph.ctw = viewport.client_to_world_transform();

        let me = App {
            mode: Mode::Idle,
            menu,
            graph,
            viewport,
            box_el,
            listeners,
            mouse_pos: ClientPoint::default(),
            el,
        };

        APP.with(|a| {
            let mut b = a.borrow_mut();
            *b = Some(me);
        });

        Ok(())
    }

    pub fn load(&mut self, graph: PenguinGraph) -> Result<(), JsValue> {
        self.graph.load(graph)
    }

    pub fn handle(&mut self, event: Event) -> Result<(), JsValue> {
        if let EventValue::MouseMove(e)
        | EventValue::MouseDown(e)
        | EventValue::MouseUp(e)
        | EventValue::MouseClick(e)
        | EventValue::ContextMenu(e) = &event.value
        {
            self.mouse_pos = e.client_pos();
        }

        if let EventValue::KeyDown(e) = &event.value
            && (e.key() == "z" || e.key() == "Z")
            && (e.ctrl_key() || e.meta_key())
        {
            return if e.shift_key() {
                self.graph.redo()
            } else {
                self.graph.undo()
            };
        }

        // menu
        if matches!(event.target, EventTarget::MenuBackdrop) {
            return if matches!(
                event.value,
                EventValue::MouseDown(_) | EventValue::ContextMenu(_)
            ) {
                self.set_mode(Mode::Idle)
            } else {
                Ok(())
            };
        }

        // node inputs
        if matches!(event.target, EventTarget::NodeInput(..)) &&
            // ignore textarea resizes while in other modes
            (!matches!(event.value, EventValue::MouseMove(_)) || matches!(self.mode, Mode::Idle))
        {
            let EventTarget::NodeInput(node_id, r#type) = event.target else {
                unreachable!()
            };

            match &event.value {
                EventValue::MouseDown(_)
                | EventValue::MouseClick(_)
                | EventValue::Input(_)
                | EventValue::Resize(_) => {
                    self.graph.clear_selection();
                    self.set_mode(Mode::Idle);
                }
                _ => {}
            };

            return match event.value {
                EventValue::Input(value) => self.graph.handle_input_change(node_id, r#type, value),
                EventValue::Resize(size) => self.graph.handle_input_resize(node_id, r#type, size),
                _ => Ok(()),
            };
        }

        if matches!(event.target, EventTarget::Node(..))
            && matches!(event.value, EventValue::Resize(..))
        {
            let EventTarget::Node(node_id) = event.target else {
                unreachable!()
            };

            let EventValue::Resize(size) = event.value else {
                unreachable!()
            };

            return self.graph.handle_node_resize(node_id, size);
        }

        // node variadic
        if matches!(event.target, EventTarget::NodeVariadic(..)) {
            let EventTarget::NodeVariadic(node_id, new_node_path) = event.target else {
                unreachable!()
            };

            return self.graph.swap_node_variant(node_id, new_node_path);
        }

        if matches!(event.target, EventTarget::MenuSearch) {
            return self.menu.handle_search_input();
        }

        // focus #penguin so keyboard input works
        if matches!(event.target, EventTarget::Global)
            && !matches!(
                event.value,
                EventValue::MouseMove(_) | EventValue::MouseUp(_)
            )
        {
            self.focus();
        }

        // mode agnostic
        match (&event.target, &event.value) {
            (EventTarget::Global, EventValue::Wheel(e)) => {
                e.prevent_default();
                self.viewport.handle_wheel(e)?;
                self.graph.ctw = self.viewport.client_to_world_transform();
                return Ok(());
            }

            (EventTarget::Global, EventValue::KeyDown(e)) => match e.key().as_str() {
                "Escape" => {
                    e.prevent_default();

                    if self.mode.is_passive() {
                        self.graph.clear_selection();
                    }

                    self.set_mode(Mode::Idle);
                }
                "Backspace" | "Delete" => {
                    e.prevent_default();

                    if self.mode.is_passive() {
                        self.graph.delete_selection();
                    }
                }
                _ => {}
            },

            (EventTarget::Global, EventValue::Copy(e)) => {
                return self.graph.handle_copy(e);
            }
            (EventTarget::Global, EventValue::Paste(e)) => {
                return self
                    .graph
                    .handle_paste(e, self.viewport.client_to_world(self.mouse_pos));
            }
            (EventTarget::Global, EventValue::Cut(e)) => {
                return self.graph.handle_cut(e);
            }

            // toolbar
            (EventTarget::ToolbarButton(btn), EventValue::MouseClick(_)) => {
                return self.viewport.handle_toolbar_button(*btn);
            }

            _ => {}
        }

        match &self.mode {
            Mode::Idle => self.handle_idle_mode(event),
            Mode::Panning(_) => self.handle_panning_mode(event),
            Mode::Dragging(_) => self.handle_dragging_mode(event),
            Mode::BoxSelecting(_) => self.handle_box_selecting_mode(event),
            Mode::Wiring(_) => self.handle_wiring_mode(event),
            Mode::Menu(_) => self.handle_menu_mode(event),
        }
    }

    pub fn set_mode(&mut self, new_mode: Mode) -> Result<(), JsValue> {
        // complete last mode
        match &self.mode {
            Mode::Wiring(_) => self.finish_wiring_mode()?,
            Mode::Menu(_) => self.finish_menu_mode()?,
            Mode::BoxSelecting(_) => self.finish_box_selecting_mode()?,
            Mode::Dragging(_) => self.finish_dragging_mode()?,
            Mode::Idle => {}
            Mode::Panning(_) => {}
        }

        self.mode = new_mode;
        Ok(())
    }

    pub fn focus(&self) -> Result<(), JsValue> {
        self.el.focus()
    }
}
