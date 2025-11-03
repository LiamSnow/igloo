use std::any::Any;

use crate::app::{APP, PenguinApp};
use wasm_bindgen::{convert::FromWasmAbi, prelude::*};
use web_sys::{Document, EventTarget, HtmlElement};

pub fn init(penguin_el: &HtmlElement) -> [Box<dyn Any>; 9] {
    let document = document();
    let mut closures = Vec::with_capacity(9);

    add_app_event_listener(
        &document,
        "mousemove",
        &mut closures,
        PenguinApp::onmousemove,
    )
    .unwrap();
    add_app_event_listener(&document, "mouseup", &mut closures, PenguinApp::onmouseup).unwrap();
    add_app_event_listener(
        penguin_el,
        "mousedown",
        &mut closures,
        PenguinApp::onmousedown,
    )
    .unwrap();
    add_app_event_listener(
        penguin_el,
        "contextmenu",
        &mut closures,
        PenguinApp::oncontextmenu,
    )
    .unwrap();

    add_app_event_listener(penguin_el, "wheel", &mut closures, PenguinApp::onwheel).unwrap();

    add_app_event_listener(penguin_el, "keydown", &mut closures, PenguinApp::onkeydown).unwrap();
    add_app_event_listener(penguin_el, "copy", &mut closures, PenguinApp::oncopy).unwrap();
    add_app_event_listener(penguin_el, "paste", &mut closures, PenguinApp::onpaste).unwrap();
    add_app_event_listener(penguin_el, "cut", &mut closures, PenguinApp::oncut).unwrap();

    closures.try_into().unwrap()
}

pub fn add_app_event_listener<E, F>(
    element: &impl AsRef<EventTarget>,
    event_name: &str,
    closures: &mut Vec<Box<dyn Any>>,
    handler: F,
) -> Result<(), JsValue>
where
    E: FromWasmAbi + 'static,
    F: Fn(&mut PenguinApp, E) + 'static,
{
    let closure = Closure::wrap(Box::new(move |e: E| {
        APP.with(|app| {
            if let Some(app) = app.borrow_mut().as_mut() {
                handler(app, e);
            }
        });
    }) as Box<dyn FnMut(E)>);
    element
        .as_ref()
        .add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())?;
    closures.push(Box::new(closure));
    Ok(())
}

pub fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}
