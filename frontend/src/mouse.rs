use dioxus::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::window;

pub static MOUSE_POSITION: GlobalSignal<(f64, f64)> = Global::new(|| (0.0, 0.0));
pub static MOUSE_BUTTON_DOWN: GlobalSignal<bool> = Global::new(|| false);

pub fn init() {
    let window = window().expect("window should exist");

    let mousemove = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        *MOUSE_POSITION.write() = (event.client_x() as f64, event.client_y() as f64);
    }) as Box<dyn FnMut(_)>);

    let mousedown = Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
        *MOUSE_BUTTON_DOWN.write() = true;
    }) as Box<dyn FnMut(_)>);

    let mouseup = Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
        *MOUSE_BUTTON_DOWN.write() = false;
    }) as Box<dyn FnMut(_)>);

    window
        .add_event_listener_with_callback("mousemove", mousemove.as_ref().unchecked_ref())
        .unwrap();
    window
        .add_event_listener_with_callback("mousedown", mousedown.as_ref().unchecked_ref())
        .unwrap();
    window
        .add_event_listener_with_callback("mouseup", mouseup.as_ref().unchecked_ref())
        .unwrap();

    mousemove.forget();
    mousedown.forget();
    mouseup.forget();
}
