use crate::cli::model::Cli;
use crate::state::IglooState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn ws_handler(
    State(state): State<Arc<IglooState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(state, socket))
}

async fn handle_socket(state: Arc<IglooState>, socket: WebSocket) {
    let uid = *state.auth.uid_lut.get("liams").unwrap(); //FIXME

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
        //TODO log
        Err(e) => return Some(serde_json::to_string(&e.render().to_string()).unwrap()), //FIXME
    };

    match cmd.dispatch(&state, uid, true).await {
        Ok(r) => r,
        //TODO log
        Err(e) => Some(serde_json::to_string(&e).unwrap()), //FIXME
    }
}
