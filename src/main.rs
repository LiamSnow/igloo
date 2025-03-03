use std::{error::Error, sync::Arc};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, StatusCode},
    response::{AppendHeaders, IntoResponse},
    routing::{any, post},
    Json, Router,
};
use cli::model::Cli;
use config::IglooConfig;
use map::IglooStack;
use tokio::{net::TcpListener, sync::Mutex};
use futures_util::{SinkExt, StreamExt};
use tower_sessions::{MemoryStore, SessionManagerLayer};

pub mod cli;
pub mod command;
pub mod config;
pub mod effects;
pub mod map;
pub mod providers;
pub mod selector;
pub mod elements;
pub mod permissions;
pub mod auth;

pub const VERSION: f32 = 0.1;
pub const CONFIG_VERSION: f32 = 0.1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = IglooConfig::from_file("./config.ron").unwrap();
    if cfg.version != CONFIG_VERSION {
        panic!(
            "Wrong config version. Got {}, expected {}.",
            cfg.version, CONFIG_VERSION
        );
    }

    let stack = IglooStack::init(cfg).await?;

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false); //FIXME
        //.with_expiry(expiry)

    let app = Router::new()
        .route("/", post(post_cmd))
        .route("/ws", any(ws_handler))
        .with_state(stack)
        .layer(session_layer);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn post_cmd(State(stack): State<Arc<IglooStack>>, cmd_str: String) -> impl IntoResponse {
    let uid = 0; //FIXME

    let cmd = match Cli::parse(&cmd_str) {
        Ok(r) => r,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(e.render().to_string())).into_response(),
    };

    match cmd.dispatch(&stack, uid).await {
        Ok(Some(body)) => (
            StatusCode::OK,
            AppendHeaders([(header::CONTENT_TYPE, "application/json")]),
            body,
        )
            .into_response(),
        Ok(None) => (StatusCode::OK).into_response(),
        //TODO log error
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
    }
}

async fn ws_handler(
    State(stack): State<Arc<IglooStack>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(stack, socket))
}

async fn handle_socket(stack: Arc<IglooStack>, socket: WebSocket) {
    let uid = 0; //FIXME

    let (ws_tx, mut ws_rx) = socket.split();
    let ws_tx = Arc::new(Mutex::new(ws_tx));
    let ws_tx_copy = ws_tx.clone();
    let mut broadcast_rx = stack.ws_broadcast.subscribe();

    let mut tx_task = tokio::spawn(async move {
        while let Ok(json) = broadcast_rx.recv().await {
            ws_tx.lock().await.send(Message::Text(json)).await.unwrap()
        }
    });

    let mut rx_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Text(cmd_str) => {
                    if let Some(json) = parse_execute_wscmd(&stack, &cmd_str, uid).await {
                        ws_tx_copy.lock().await.send(Message::Text(json.into())).await.unwrap()
                    }
                }
                Message::Close(_) => {
                    return;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut tx_task) => {
            rx_task.abort();
        },
        _ = (&mut rx_task) => {
            tx_task.abort();
        }
    }
}

async fn parse_execute_wscmd(stack: &Arc<IglooStack>, cmd_str: &str, uid: usize) -> Option<String> {
    let cmd = match Cli::parse(cmd_str) {
        Ok(r) => r,
        //TODO log
        Err(e) => return Some(serde_json::to_string(&e.render().to_string()).unwrap()), //FIXME
    };

    match cmd.dispatch(&stack, uid).await {
        Ok(r) => r,
        //TODO log
        Err(e) => Some(serde_json::to_string(&e).unwrap()), //FIXME
    }
}
