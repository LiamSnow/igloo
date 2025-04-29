use crate::cli::model::Cli;
use crate::state::IglooState;
use axum::{
    extract::{
        ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use axum_extra::{headers::Cookie, TypedHeader};
use futures_util::{SinkExt, StreamExt};
use tracing::{debug, span, warn, Level};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn ws_handler(
    State(state): State<Arc<IglooState>>,
    cookies: Option<TypedHeader<Cookie>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(cookies, state, socket))
}

async fn close_socket(mut socket: WebSocket, code: u16, reason: String) {
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        })))
        .await;

    //wait for proper closure
    loop {
        if let Some(Ok(msg)) = socket.recv().await {
            match msg {
                Message::Close(_) => break,
                _ => {}
            }
        }
    }
}

async fn handle_socket(
    cookies: Option<TypedHeader<Cookie>>,
    state: Arc<IglooState>,
    socket: WebSocket,
) {
    let span = span!(Level::INFO, "WebSocket");
    let _enter = span.enter();
    debug!("checking permissions");

    let uid = match cookies
        .as_ref()
        .and_then(|cookies| cookies.get("auth_token"))
    {
        Some(token) => match state.auth.token_db.validate(token.to_string()).await {
            Ok(username_opt) => {
                match username_opt.and_then(|username| state.auth.uid_lut.get(&username)) {
                    Some(uid) => *uid,
                    None => {
                        return close_socket(socket, 1008, "Invalid token".to_string()).await;
                    }
                }
            }
            Err(e) => return close_socket(socket, 1011, e.to_string()).await,
        },
        None => return close_socket(socket, 1008, "Not authenticated".to_string()).await,
    };

    debug!("uid={uid}");

    let (ws_tx, mut ws_rx) = socket.split();
    let ws_tx = Arc::new(Mutex::new(ws_tx));
    let ws_tx_copy = ws_tx.clone();
    let mut on_change_rx = state.elements.on_change.subscribe();

    let mut tx_task = tokio::spawn(async move {
        while let Ok(json) = on_change_rx.recv().await {
            ws_tx.lock().await.send(Message::Text(json)).await.unwrap()
        }
    });

    let mut rx_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Text(cmd_str) => {
                    if let Some(json) = parse_execute_wscmd(&state, &cmd_str, uid).await {
                        ws_tx_copy
                            .lock()
                            .await
                            .send(Message::Text(json.into()))
                            .await
                            .unwrap()
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

async fn parse_execute_wscmd(state: &Arc<IglooState>, cmd_str: &str, uid: usize) -> Option<String> {
    let cmd = match Cli::parse(cmd_str) {
        Ok(r) => r,
        Err(e) => {
            let e = serde_json::to_string(&e.render().to_string()).unwrap();
            warn!("command failed: {e}");
            return Some(e)
        },
    };

    match cmd.dispatch(&state, Some(uid), true).await {
        Ok(r) => r,
        Err(e) => {
            let e = serde_json::to_string(&e).unwrap(); //FIXME
            warn!("command failed: {e}");
            Some(e)
        }
    }
}
