use dioxus::signals::{GlobalSignal, Signal};
use wasm_bindgen::prelude::*;
use web_sys::Document;

#[wasm_bindgen(module = "/assets/penguin.js")]
extern "C" {
    pub fn init();
    pub fn rerender();
    pub fn delayedRerender();
    pub fn getSelectedNodeIds() -> Vec<u16>;
    pub fn getSelectedWireIds() -> Vec<u16>;
    pub fn startWiring(start_node: u16, start_pin: &str, is_output: bool);
    pub fn stopWiring();
    pub fn getAllNodePositions() -> Vec<JsValue>;
    pub fn setGridSettings(enabled: bool, snap: bool, size: f64);
}

pub static LMB_DOWN: GlobalSignal<bool> = Signal::global(|| false);
pub static RMB_DOWN: GlobalSignal<bool> = Signal::global(|| false);

pub fn register_listeners() {
    let document = web_sys::window().unwrap().document().unwrap();
    setup_mouseup(&document);
    setup_mousedown(&document);
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
