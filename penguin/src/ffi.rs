use std::any::Any;

use crate::app::APP;
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlElement, MouseEvent, WheelEvent};

pub fn init(penguin_el: &HtmlElement) -> Vec<Box<dyn Any>> {
    let document = document();
    vec![
        attach_onmousemove(&document),
        attach_onmouseup(&document),
        attach_onmousedown(penguin_el),
        attach_oncontextmenu(penguin_el),
        attach_onwheel(penguin_el),
    ]
}

fn attach_onmousemove(document: &Document) -> Box<dyn Any> {
    let c = Closure::wrap(Box::new(move |e: MouseEvent| {
        APP.with(|app| {
            if let Some(app) = app.borrow_mut().as_mut() {
                app.onmousemove(e);
            }
        });
    }) as Box<dyn FnMut(_)>);

    document
        .add_event_listener_with_callback("mousemove", c.as_ref().unchecked_ref())
        .unwrap();

    Box::new(c)
}

fn attach_onmouseup(document: &Document) -> Box<dyn Any> {
    let c = Closure::wrap(Box::new(move |e: MouseEvent| {
        APP.with(|app| {
            if let Some(app) = app.borrow_mut().as_mut() {
                app.onmouseup(e);
            }
        });
    }) as Box<dyn FnMut(_)>);

    document
        .add_event_listener_with_callback("mouseup", c.as_ref().unchecked_ref())
        .unwrap();

    Box::new(c)
}

fn attach_onmousedown(penguin_el: &HtmlElement) -> Box<dyn Any> {
    let c = Closure::wrap(Box::new(move |e: MouseEvent| {
        APP.with(|app| {
            if let Some(app) = app.borrow_mut().as_mut() {
                app.onmousedown(e);
            }
        });
    }) as Box<dyn FnMut(_)>);

    penguin_el
        .add_event_listener_with_callback("mousedown", c.as_ref().unchecked_ref())
        .unwrap();

    Box::new(c)
}

fn attach_oncontextmenu(penguin_el: &HtmlElement) -> Box<dyn Any> {
    let c = Closure::wrap(Box::new(move |e: MouseEvent| {
        APP.with(|app| {
            if let Some(app) = app.borrow_mut().as_mut() {
                app.oncontextmenu(e);
            }
        });
    }) as Box<dyn FnMut(_)>);

    penguin_el
        .add_event_listener_with_callback("contextmenu", c.as_ref().unchecked_ref())
        .unwrap();

    Box::new(c)
}

fn attach_onwheel(penguin_el: &HtmlElement) -> Box<dyn Any> {
    let c = Closure::wrap(Box::new(move |e: WheelEvent| {
        e.prevent_default();
        APP.with(|app| {
            if let Some(app) = app.borrow_mut().as_mut() {
                app.onwheel(e);
            }
        });
    }) as Box<dyn FnMut(_)>);

    penguin_el
        .add_event_listener_with_callback("wheel", c.as_ref().unchecked_ref())
        .unwrap();

    Box::new(c)
}

pub fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}
