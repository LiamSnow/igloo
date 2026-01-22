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
use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::core::{ClientMsg, IglooRequest, IglooResponse};

#[derive(Clone)]
struct WState {
    req_tx: kanal::AsyncSender<IglooRequest>,
}

/*

Core Idea: lazy observer registration
 - Client navigates to page X
 - Backend ships them the dashboard JSON
 - SolidJS loads all requires elements (and JS from packages as needed)
 - Elements can subscribe to watcher
    1. Server registers watcher if it doesn't exist
    2. Sends current state
 - Upon navigation to another page, client requests to unsub from all

-------------------

MVP Should:
 1. Host SolidJS website
 2. Handle websockets
     - Each should register their own client with the QueryEngine
     - Proxy ClientMsg
     - Unregister from QueryEngine on drop

*/

pub async fn run(req_tx: kanal::Sender<IglooRequest>) -> Result<(), Box<dyn Error>> {
    let state = WState {
        req_tx: req_tx.to_async(),
    };

    let app = Router::new()
        .route("/ws", any(ws_handler))
        .nest_service("/plugins", ServeDir::new("./plugins"))
        .nest_service("/dashboards", ServeDir::new("./dashboards"))
        .fallback_service(ServeDir::new("./web").append_index_html_on_directories(true))
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

async fn handle_socket(_cookies: Option<TypedHeader<Cookie>>, state: WState, socket: WebSocket) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    // register w/ igloo core
    let (res_tx, res_rx) = kanal::bounded::<IglooResponse>(50);
    if let Err(e) = state
        .req_tx
        .send(IglooRequest::RegisterClient(res_tx))
        .await
    {
        eprintln!("Failed to send registration request: {e}");
        return;
    }
    let res_rx = res_rx.to_async();
    let client_id = match res_rx.recv().await {
        Ok(response @ IglooResponse::Registered { client_id }) => {
            println!("Client {client_id} registered");

            // Send registration confirmation to client
            if let Ok(json) = serde_json::to_string(&response) {
                if let Err(e) = ws_tx.send(Message::Text(json.into())).await {
                    eprintln!("Failed to send registration to websocket: {e}");
                    return;
                }
            }

            client_id
        }
        Ok(other) => {
            eprintln!("Unexpected response during registration: {other:?}");
            return;
        }
        Err(e) => {
            eprintln!("Failed to receive registration response: {e}");
            return;
        }
    };

    // run proxy
    loop {
        tokio::select! {
            // Client -> Core
            ws_msg = ws_rx.next() => {
                match ws_msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientMsg>(&text) {
                            Ok(msg) => {
                                if let Err(e) = state.req_tx.send(IglooRequest::Client {
                                    client_id,
                                    msg,
                                }).await {
                                    eprintln!("Failed to send message to core: {e}");
                                    break;
                                }
                            }
                            Err(e) => {
                                // TODO send err over ws
                                eprintln!("Failed to deserialize ClientMsg: {e}\nRaw:{text}");
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        println!("Client {client_id} closed connection");
                        break;
                    }
                    Some(Err(e)) => {
                        eprintln!("Websocket error for client {client_id}: {e}");
                        break;
                    }
                    None => {
                        println!("Client {client_id} disconnected");
                        break;
                    }
                    _ => { }
                }
            }

            // Core -> Client
            res = res_rx.recv() => {
                match res {
                    Ok(res) => {
                        match serde_json::to_string(&res) {
                            Ok(json) => {
                                if let Err(e) = ws_tx.send(Message::Text(json.into())).await {
                                    eprintln!("Failed to send response to websocket: {e}");
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to serialize IglooResponse: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Response channel closed for client {client_id}: {e}");
                        break;
                    }
                }
            }
        }
    }

    println!("Unregistering client {client_id}");
    if let Err(e) = state
        .req_tx
        .send(IglooRequest::Client {
            client_id,
            msg: ClientMsg::Unregister,
        })
        .await
    {
        eprintln!("Failed to unregister client {client_id}: {e}");
    }
}
