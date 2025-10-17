use std::collections::HashMap;

use dioxus::prelude::*;
use igloo_interface::{
    dash::Dashboard,
    ws::{ClientMessage, ClientPage, DashboardMeta, ServerMessage},
    Component, Snapshot,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{js_sys, ErrorEvent, MessageEvent, WebSocket};

use crate::Route;

pub static WS_CONNECTED: GlobalSignal<bool> = Global::new(|| false);
pub static DASHBOARDS: GlobalSignal<Vec<DashboardMeta>> = Global::new(Vec::new);
pub static CURRENT_DASHBOARD: GlobalSignal<Option<Dashboard>> = Global::new(|| None);
pub static CURRENT_SNAPSHOT: GlobalSignal<Option<Snapshot>> = Global::new(|| None);
pub static CURRENT_ROUTE: GlobalSignal<Route> = Global::new(|| Route::Settings {});
pub static ELEMENT_VALUES: GlobalSignal<HashMap<u32, Component>> = Global::new(HashMap::new);
static WS_INSTANCE: GlobalSignal<Option<WebSocket>> = Global::new(|| None);

const WS_URL: &str = "ws://localhost:3000/ws";
const MAX_BACKOFF_MS: i32 = 30000;

pub fn connect_websocket() {
    let ws = match WebSocket::new(WS_URL) {
        Ok(ws) => ws,
        Err(e) => {
            log::error!("Failed to create WebSocket: {:?}", e);
            schedule_reconnect(1000);
            return;
        }
    };

    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let ws_clone = ws.clone();
    let onopen = Closure::wrap(Box::new(move |_| {
        log::info!("WebSocket connected");
        *WS_CONNECTED.write() = true;
        send_msg_ws(&ws_clone, ClientMessage::Init);
        send_cur_page_ws(&ws_clone);
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();

    let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
        let Ok(array_buffer) = e.data().dyn_into::<js_sys::ArrayBuffer>() else {
            log::error!("Failed to get ArrayBuffer");
            return;
        };

        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        let bytes: Vec<u8> = uint8_array.to_vec();

        match borsh::from_slice::<ServerMessage>(&bytes) {
            Ok(msg) => match msg {
                ServerMessage::Dashboard(dash_id, dash) => {
                    log::info!("Received dashboard {dash_id:?}");
                    *CURRENT_DASHBOARD.write() = Some(*dash);
                }
                ServerMessage::ElementUpdate(u) => {
                    // log::info!("Received update for watch_id {}", u.watch_id);
                    ELEMENT_VALUES.write().insert(u.watch_id, u.value);
                }
                ServerMessage::Snapshot(snap) => {
                    log::info!("Received snapshot");
                    *CURRENT_SNAPSHOT.write() = Some(*snap);
                }
                ServerMessage::Dashboards(metas) => {
                    *DASHBOARDS.write() = metas;
                }
            },
            Err(e) => {
                log::error!("Failed to deserialize message: {:?}", e);
            }
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    let onerror = Closure::wrap(Box::new(move |e: ErrorEvent| {
        log::error!("WebSocket error: {:?}", e.message());
        *WS_CONNECTED.write() = false;
    }) as Box<dyn FnMut(ErrorEvent)>);
    ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onerror.forget();

    let onclose = Closure::wrap(Box::new(move |_| {
        log::warn!("WebSocket closed");
        *WS_CONNECTED.write() = false;
        *WS_INSTANCE.write() = None;

        schedule_reconnect(1000);
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
    onclose.forget();

    *WS_INSTANCE.write() = Some(ws);
}

fn schedule_reconnect(mut backoff_ms: i32) {
    let window = web_sys::window().expect("no global window");

    let closure = Closure::once(Box::new(move || {
        log::info!("Attempting to reconnect...");
        connect_websocket();
    }) as Box<dyn FnOnce()>);

    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        backoff_ms,
    );
    closure.forget();

    backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
}

fn send_msg_ws(ws: &WebSocket, msg: ClientMessage) {
    if ws.ready_state() != WebSocket::OPEN {
        return;
    }

    match borsh::to_vec(&msg) {
        Ok(bytes) => {
            if let Err(e) = ws.send_with_u8_array(&bytes) {
                log::error!("Failed to send WS message: {e:?}");
            }
        }
        Err(e) => {
            log::error!("Failed to serialize message: {e:?}");
        }
    }
}

pub fn send_msg(msg: ClientMessage) {
    let ws = WS_INSTANCE.read();
    if let Some(ws) = ws.as_ref() {
        send_msg_ws(ws, msg);
    } else {
        log::warn!("WebSocket not connected, cannot send dashboard ID");
    }
}

fn send_cur_page_ws(ws: &WebSocket) {
    send_msg_ws(ws, cur_page().into());
}

pub fn send_cur_page() {
    send_msg(cur_page().into());
}

fn cur_page() -> ClientPage {
    match &*CURRENT_ROUTE.read() {
        Route::Dash { id } => ClientPage::Dashboard(Some(id.clone())),
        Route::DashDefault {} => ClientPage::Dashboard(None),
        Route::Penguin {} => ClientPage::Penguin,
        Route::Settings {} => ClientPage::Settings,
        Route::Tree {} => ClientPage::Tree,
    }
}
