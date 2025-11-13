use axum::{
    Router,
    body::Bytes,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::any,
};
use axum_extra::{TypedHeader, headers::Cookie};
use futures_util::StreamExt;
use igloo_interface::web::ws::{ClientMessage, ClientPage, DashboardMeta, ServerMessage};
use std::error::Error;
use tokio::net::TcpListener;

use crate::{DashboardRequest, GlobalState};

mod test_dashes;
mod watch;

pub async fn run(state: GlobalState) -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        // .route("/", get(d))
        .route("/ws", any(ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn ws_handler(
    State(state): State<GlobalState>,
    cookies: Option<TypedHeader<Cookie>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(cookies, state, socket))
}

async fn handle_socket(
    _cookies: Option<TypedHeader<Cookie>>,
    state: GlobalState,
    mut socket: WebSocket,
) {
    // test_dashes::make(&state).await.unwrap();

    let mut cast_rx = state.cast.subscribe();
    let mut cur_dash_idx = u16::MAX;
    loop {
        tokio::select! {
            Ok((dash_idx, msg)) = cast_rx.recv() => {
                if dash_idx != cur_dash_idx {
                    continue;
                }

                socket.send(msg).await.unwrap();
            }

            Some(Ok(msg)) = socket.next() => {
                match msg {
                    Message::Binary(bytes) => {
                        let res = handle_client_msg(
                            &state,
                            &mut socket,
                            &mut cur_dash_idx,
                            bytes
                        ).await;

                        if let Err(e) = res {
                            eprintln!("Error handling client message: {e}");
                        }
                    }
                    Message::Close(_) => {
                        return;
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn handle_client_msg(
    state: &GlobalState,
    socket: &mut WebSocket,
    cur_dash_idx: &mut u16,
    bytes: Bytes,
) -> Result<(), Box<dyn Error>> {
    let msg: ClientMessage = borsh::from_slice(&bytes)?;

    match msg {
        // ClientMessage::ExecSetQuery(q) => state.query_tx.send(q.into()).await.unwrap(),
        ClientMessage::Init => {
            let dashs = state.dashs.read().await;
            let mut metas = Vec::with_capacity(dashs.len());
            for (id, dash) in dashs.clone().into_iter() {
                metas.push(DashboardMeta {
                    is_default: id == "main", // FIXME use user pref
                    id,
                    display_name: dash.display_name,
                });
            }
            drop(dashs);
            let msg: ServerMessage = metas.into();
            let bytes = borsh::to_vec(&msg)?;
            socket.send(Message::Binary(bytes.into())).await?;
        }
        ClientMessage::GetPageData(ClientPage::Dashboard(dash_id)) => {
            let Some(dash_id) = dash_id else {
                *cur_dash_idx = u16::MAX;
                return Ok(());
            };

            let dash = state
                .dashs
                .read()
                .await
                .get(&dash_id)
                .ok_or("invalid dashboard ID")?
                .clone();

            *cur_dash_idx = dash.idx.unwrap(); // always init
            let msg: ServerMessage = (Some(dash_id), Box::new(dash)).into();
            let bytes = borsh::to_vec(&msg)?;

            socket.send(Message::Binary(bytes.into())).await?;

            state
                .dash_tx
                .send((*cur_dash_idx, DashboardRequest::DumpData))?;
        }
        ClientMessage::GetPageData(ClientPage::Tree) => {
            *cur_dash_idx = u16::MAX;
            todo!()
            // let (response_tx, response_rx) = oneshot::channel();
            // state
            //     .query_tx
            //     .send(Query::Snapshot(SnapshotQuery { response_tx }))
            //     .await?;
            // let msg: ServerMessage = Box::new(response_rx.await?).into();
            // let bytes = borsh::to_vec(&msg)?;
            // socket.send(Message::Binary(bytes.into())).await?;
        }
        ClientMessage::GetPageData(ClientPage::Settings) => {
            *cur_dash_idx = u16::MAX;
        }
        ClientMessage::GetPageData(ClientPage::Penguin) => {
            *cur_dash_idx = u16::MAX;
        }
    }

    Ok(())
}
