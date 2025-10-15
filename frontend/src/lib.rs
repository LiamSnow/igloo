use igloo_interface::ws::{ClientMessage, ServerMessage};
use log::Level;
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{Document, ErrorEvent, HtmlInputElement, MessageEvent, WebSocket, js_sys};

use crate::dash::RenderDash;

mod dash;
mod elements;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    console_log::init_with_level(Level::Debug).expect("Failed to init logger");
    log::info!("Connecting to WS");

    connect_websocket().unwrap();

    Ok(())
}

pub fn connect_websocket() -> Result<(), JsValue> {
    let ws = WebSocket::new("ws://localhost:3000/ws")?;

    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let onmessage_callback = create_onmessage_callback(ws.clone());
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    let onerror_callback = create_onerror_callback();
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let onopen_callback = create_onopen_callback(ws.clone());
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    Ok(())
}

fn create_onmessage_callback(ws: WebSocket) -> Closure<dyn FnMut(MessageEvent)> {
    Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        let Ok(array_buffer) = e.data().dyn_into::<js_sys::ArrayBuffer>() else {
            log::error!("Failed to get ArrayBuffer");
            return;
        };

        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        let bytes: Vec<u8> = uint8_array.to_vec();

        let msg: ServerMessage = borsh::from_slice(&bytes).unwrap();
        match msg {
            ServerMessage::Dashboard(dash) => {
                log::info!("Rendering Dashboard: {}", dash.name);
                if let Err(e) = dash.render(0, &ws) {
                    log::error!("Error rendering dashboard: {e}");
                }
            }
            ServerMessage::ElementUpdate(update) => {
                // TODO pull into another method?
                let Some(el) = document().get_element_by_id(&update.elid.to_string()) else {
                    log::error!("Update for Missing Element: elid={}", update.elid);
                    return;
                };

                let Some(inner) = update.value.inner_string() else {
                    log::error!("Update for Non-Valued Component: {:?}", update.value);
                    return;
                };

                if let Some(input) = el.dyn_ref::<HtmlInputElement>() {
                    input.set_value(&inner);
                } else {
                    el.set_text_content(Some(&inner));
                }
            }
        }
    })
}

fn create_onerror_callback() -> Closure<dyn FnMut(ErrorEvent)> {
    Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        log::error!("WS Error: {:?}", e);
    })
}

fn create_onopen_callback(ws: WebSocket) -> Closure<dyn FnMut()> {
    Closure::<dyn FnMut()>::new(move || {
        log::info!("WS Connected!");
        let msg = ClientMessage::SetDashboard(0);
        let v = borsh::to_vec(&msg).unwrap();
        ws.send_with_u8_array(&v).unwrap();
    })
}

pub fn document() -> Document {
    web_sys::window()
        .expect("missing window")
        .document()
        .expect("missing document")
}
