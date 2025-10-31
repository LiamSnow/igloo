use std::any::Any;

use crate::app::APP;
use wasm_bindgen::prelude::*;
use web_sys::{ClipboardEvent, Document, HtmlElement, KeyboardEvent, MouseEvent, WheelEvent};

pub fn init(penguin_el: &HtmlElement) -> [Box<dyn Any>; 9] {
    let document = document();
    [
        attach_onmousemove(&document),
        attach_onmouseup(&document),
        attach_onmousedown(penguin_el),
        attach_oncontextmenu(penguin_el),
        attach_onwheel(penguin_el),
        attach_onkeydown(penguin_el),
        attach_oncopy(penguin_el),
        attach_onpaste(penguin_el),
        attach_oncut(penguin_el),
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

fn attach_onkeydown(penguin_el: &HtmlElement) -> Box<dyn Any> {
    let c = Closure::wrap(Box::new(move |e: KeyboardEvent| {
        APP.with(|app| {
            if let Some(app) = app.borrow_mut().as_mut() {
                app.onkeydown(e);
            }
        });
    }) as Box<dyn FnMut(_)>);

    penguin_el
        .add_event_listener_with_callback("keydown", c.as_ref().unchecked_ref())
        .unwrap();

    Box::new(c)
}

fn attach_oncopy(penguin_el: &HtmlElement) -> Box<dyn Any> {
    let c = Closure::wrap(Box::new(move |e: ClipboardEvent| {
        APP.with(|app| {
            if let Some(app) = app.borrow_mut().as_mut() {
                app.oncopy(e);
            }
        });
    }) as Box<dyn FnMut(_)>);

    penguin_el
        .add_event_listener_with_callback("copy", c.as_ref().unchecked_ref())
        .unwrap();

    Box::new(c)
}

fn attach_onpaste(penguin_el: &HtmlElement) -> Box<dyn Any> {
    let c = Closure::wrap(Box::new(move |e: ClipboardEvent| {
        APP.with(|app| {
            if let Some(app) = app.borrow_mut().as_mut() {
                app.onpaste(e);
            }
        });
    }) as Box<dyn FnMut(_)>);

    penguin_el
        .add_event_listener_with_callback("paste", c.as_ref().unchecked_ref())
        .unwrap();

    Box::new(c)
}

fn attach_oncut(penguin_el: &HtmlElement) -> Box<dyn Any> {
    let c = Closure::wrap(Box::new(move |e: ClipboardEvent| {
        APP.with(|app| {
            if let Some(app) = app.borrow_mut().as_mut() {
                app.oncut(e);
            }
        });
    }) as Box<dyn FnMut(_)>);

    penguin_el
        .add_event_listener_with_callback("cut", c.as_ref().unchecked_ref())
        .unwrap();

    Box::new(c)
}

pub fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}
