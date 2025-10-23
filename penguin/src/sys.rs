use dioxus::prelude::*;
use euclid::default::Point2D;
use euclid::default::Vector2D;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Document;

pub const MIN_ZOOM: f64 = 0.1;
pub const MAX_ZOOM: f64 = 3.0;
pub const DRAG_THRESHOLD: f64 = 5.0;

pub static MOUSE_DELTA: GlobalSignal<Vector2D<f64>> = Signal::global(Vector2D::default);
pub static MOUSE_POSITION: GlobalSignal<Point2D<f64>> = Signal::global(Point2D::default);
pub static LMB_DOWN: GlobalSignal<bool> = Signal::global(|| false);
pub static RMB_DOWN: GlobalSignal<bool> = Signal::global(|| false);

pub fn setup() {
    let document = web_sys::window().unwrap().document().unwrap();

    setup_mousemove(&document);
    setup_mouseup(&document);
    setup_mousedown(&document);
}

fn setup_mousemove(document: &Document) {
    let c = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        *MOUSE_DELTA.write() = Vector2D::new(e.movement_x() as f64, e.movement_y() as f64);
        *MOUSE_POSITION.write() = Point2D::new(e.client_x() as f64, e.client_y() as f64);
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
