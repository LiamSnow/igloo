use crate::app::mode::Mode;
use crate::dom::{
    self, Div,
    events::{Clientable, Event, EventTarget, EventValue},
    node::DomNode,
};
use crate::graph::WebGraph;
use crate::menu::Menu;
use crate::viewport::{ClientPoint, Viewport};
use igloo_interface::penguin::{PenguinRegistry, graph::PenguinGraph};
use std::cell::RefCell;

pub mod mode;

thread_local! {
    pub static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

#[derive(Debug)]
pub struct App {
    pub(self) el: DomNode<Div>,
    _doc: DomNode<()>,

    pub(self) mode: Mode,

    pub(self) graph: WebGraph,
    pub(self) viewport: Viewport,
    pub(self) menu: Menu,
    pub(self) box_el: DomNode<Div>,

    pub(self) mouse_pos: ClientPoint,
}

impl App {
    pub fn init() {
        let el = dom::wrap::<Div>(dom::query_id("penguin").expect("Cannot find #penguin"))
            .tab_index(0)
            .event_target(EventTarget::Global)
            .listen_mousedown()
            .listen_contextmenu()
            .listen_wheel()
            .listen_keydown()
            .listen_copy()
            .listen_paste()
            .listen_cut()
            .remove_on_drop()
            .build();

        let _doc = dom::wrap::<()>(dom::document())
            .event_target(EventTarget::Global)
            .listen_mousemove()
            .listen_mouseup()
            .build();

        let grid_svg = dom::svg().id("penguin-grid").mount(&el);

        let viewport_el = dom::div().id("penguin-viewport").mount(&el);

        let box_el = dom::div().id("penguin-selection-box").hide().mount(&el);

        let registry = PenguinRegistry::default();
        let menu = Menu::new(&registry, &el);
        let mut graph = WebGraph::new(registry, &viewport_el);
        let viewport = Viewport::new(el.dupe(), viewport_el, grid_svg);
        graph.ctw = viewport.client_to_world_transform();

        let me = App {
            mode: Mode::Idle,
            menu,
            graph,
            viewport,
            box_el,
            mouse_pos: ClientPoint::default(),
            el,
            _doc,
        };

        APP.with(|a| {
            let mut b = a.borrow_mut();
            *b = Some(me);
        });
    }

    pub fn load(&mut self, graph: PenguinGraph) {
        self.graph.load(graph);
    }

    pub fn handle(&mut self, event: Event) {
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
            if e.shift_key() {
                self.graph.redo()
            } else {
                self.graph.undo()
            }

            return;
        }

        // menu
        if matches!(event.target, EventTarget::MenuBackdrop) {
            if matches!(
                event.value,
                EventValue::MouseDown(_) | EventValue::ContextMenu(_)
            ) {
                self.set_mode(Mode::Idle);
            };

            return;
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

            match event.value {
                EventValue::Input(value) => self.graph.handle_input_change(node_id, r#type, value),
                EventValue::Resize(size) => self.graph.handle_input_resize(node_id, r#type, size),
                _ => {}
            }

            return;
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

            self.graph.handle_node_resize(node_id, size);

            return;
        }

        // node variadic
        if matches!(event.target, EventTarget::NodeVariadic(..)) {
            let EventTarget::NodeVariadic(node_id, new_node_path) = event.target else {
                unreachable!()
            };

            self.graph.swap_node_variant(node_id, new_node_path);

            return;
        }

        if matches!(event.target, EventTarget::MenuSearch) {
            self.menu.handle_search_input();
            return;
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
                self.viewport.handle_wheel(e);
                self.graph.ctw = self.viewport.client_to_world_transform();
                return;
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
                self.graph.handle_copy(e);
                return;
            }
            (EventTarget::Global, EventValue::Paste(e)) => {
                self.graph
                    .handle_paste(e, self.viewport.client_to_world(self.mouse_pos));
                return;
            }
            (EventTarget::Global, EventValue::Cut(e)) => {
                self.graph.handle_cut(e);
                return;
            }

            // toolbar
            (EventTarget::ToolbarButton(btn), EventValue::MouseClick(_)) => {
                self.viewport.handle_toolbar_button(*btn);
                return;
            }

            _ => {}
        }

        match &self.mode {
            Mode::Idle => {
                self.handle_idle_mode(event);
            }
            Mode::Panning(_) => {
                self.handle_panning_mode(event);
            }
            Mode::Dragging(_) => {
                self.handle_dragging_mode(event);
            }
            Mode::BoxSelecting(_) => {
                self.handle_box_selecting_mode(event);
            }
            Mode::Wiring(_) => {
                self.handle_wiring_mode(event);
            }
            Mode::Menu(_) => {
                self.handle_menu_mode(event);
            }
        }
    }

    pub fn set_mode(&mut self, new_mode: Mode) {
        // complete last mode
        match &self.mode {
            Mode::Wiring(_) => self.finish_wiring_mode(),
            Mode::Menu(_) => self.finish_menu_mode(),
            Mode::BoxSelecting(_) => self.finish_box_selecting_mode(),
            Mode::Dragging(_) => self.finish_dragging_mode(),
            Mode::Idle => {}
            Mode::Panning(_) => {}
        }

        self.mode = new_mode;
    }

    pub fn focus(&self) {
        self.el.focus();
    }
}
