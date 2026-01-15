use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::any,
};
use axum_extra::{TypedHeader, headers::Cookie};
use futures_util::StreamExt;
use std::error::Error;
use tokio::net::TcpListener;

use crate::core::IglooRequest;

#[derive(Clone)]
struct WState;

/*

Core Idea: lazy observer registration
 - Client navigates to page X
 - Backend ships them the dashboard JSON
 - SolidJS loads all requires elements (and JS from packages as needed)
 - Elements can subscribe to observer
    1. Server registers observer if it doesn't exist
    2. Sends current state
 - Upon navigation to another page, client requests to unsub from all

==================

MVP Requires:
 1. Change QueryEngine for new observer setup
     - Upon registering a observer, checks for duplicates, and can join existing
     - Handle garbage collection (remove observers when no subs)
     - Potentially add lifetime to observers (IE only remove when no subs + time)
     - Add ObserveMeta (includes: dashboard meta, devices meta, group meta, ext meta)
 2.

-------------------

MVP Should:
 1. Host SolidJS website
 2. Handle websockets
     - Each should register their own client with the QueryEngine
     - Proxy queries
     - Unregister from QueryEngine on drop

*/

pub async fn run(_req_tx: kanal::Sender<IglooRequest>) -> Result<(), Box<dyn Error>> {
    let state = WState;

    let app = Router::new()
        // .route("/", get(d))
        .route("/ws", any(ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn ws_handler(
    State(state): State<WState>,
    cookies: Option<TypedHeader<Cookie>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(cookies, state, socket))
}

async fn handle_socket(
    _cookies: Option<TypedHeader<Cookie>>,
    _state: WState,
    mut socket: WebSocket,
) {
    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            Message::Text(bytes) => {
                println!("Got message: {bytes:?}");
            }
            Message::Close(_) => {
                println!("Socket closed.");
                return;
            }
            _ => {}
        }
    }
}
