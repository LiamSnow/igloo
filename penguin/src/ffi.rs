use dioxus::{
    html::geometry::ClientPoint,
    signals::{GlobalSignal, ReadableExt, Signal},
};
use euclid::default::Point2D;
use wasm_bindgen::prelude::*;
use web_sys::Document;

#[wasm_bindgen(module = "/assets/penguin.js")]
extern "C" {
    pub fn init();
    pub fn rerender();
    pub fn delayedRerender();
    pub fn clearSelection();
    fn getSelectedNodeIds() -> Vec<u16>;
    fn getSelectedWireIds() -> Vec<u16>;
    pub fn startWiring(
        start_node: u16,
        start_pin_defn: u32,
        start_pin_phantom: u8,
        is_output: bool,
    );
    pub fn stopWiring();
    pub fn getAllNodePositions() -> Vec<JsValue>;
    pub fn setGridSettings(enabled: bool, snap: bool, size: f64);
    pub fn isInputFocused() -> bool;
    pub fn clientToWorld(client_x: f64, client_y: f64) -> Vec<f64>;
}

pub static LMB_DOWN: GlobalSignal<bool> = Signal::global(|| false);
pub static RMB_DOWN: GlobalSignal<bool> = Signal::global(|| false);
pub static MOUSE_POS: GlobalSignal<ClientPoint> = Signal::global(ClientPoint::default);

pub fn register_listeners() {
    let document = web_sys::window().unwrap().document().unwrap();
    setup_mousemove(&document);
    setup_mouseup(&document);
    setup_mousedown(&document);
}

pub struct Selection {
    pub node_ids: Vec<u16>,
    pub wire_ids: Vec<u16>,
}

pub fn get_selection() -> Selection {
    Selection {
        node_ids: getSelectedNodeIds(),
        wire_ids: getSelectedWireIds(),
    }
}

pub fn get_mouse_world_pos() -> Point2D<f64> {
    let mouse = *MOUSE_POS.peek();
    let world = clientToWorld(mouse.x, mouse.y);
    Point2D::new(world[0], world[1])
}

fn setup_mousemove(document: &Document) {
    let c = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        *MOUSE_POS.write() = ClientPoint::new(e.client_x() as f64, e.client_y() as f64);
    }) as Box<dyn FnMut(_)>);

    document
        .add_event_listener_with_callback("mousemove", c.as_ref().unchecked_ref())
        .unwrap();
    c.forget();
}

fn setup_mouseup(document: &Document) {
    let c = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| match e.button() {
        0 => {
            *LMB_DOWN.write() = false;
        }
        2 => {
            *RMB_DOWN.write() = false;
        }
        _ => {}
    }) as Box<dyn FnMut(_)>);

    document
        .add_event_listener_with_callback("mouseup", c.as_ref().unchecked_ref())
        .unwrap();
    c.forget();
}

fn setup_mousedown(document: &Document) {
    let c = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| match e.button() {
        0 => {
            *LMB_DOWN.write() = true;
        }
        2 => {
            *RMB_DOWN.write() = true;
        }
        _ => {}
    }) as Box<dyn FnMut(_)>);

    document
        .add_event_listener_with_callback("mousedown", c.as_ref().unchecked_ref())
        .unwrap();
    c.forget();
}
