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
use borsh::{BorshDeserialize, BorshSerialize};
use futures_util::StreamExt;
use igloo_interface::{Component, ComponentType};
use rustc_hash::FxHashMap;
use std::error::Error;
use tokio::{net::TcpListener, sync::mpsc};

use crate::{
    GlobalState,
    dash::model::{Dashboard, Element},
    glacier::{
        query::{Query, QueryFilter, QueryKind, QueryTarget, SetQuery},
        tree::DeviceID,
    },
};

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub enum Imsg {
    Dash(Box<Dashboard>),
    Update(u32, Component),
    Set(SetQuery),
    GetDash(u16),
}

pub async fn run(state: GlobalState) -> Result<(), Box<dyn Error>> {
    let mut targets = FxHashMap::default();
    targets.insert(
        "surf".to_string(),
        QueryTarget::Device(DeviceID::from_parts(0, 0)),
    );

    let dash = Dashboard {
        name: "test".to_string(),
        targets,
        child: Element::Slider {
            binding: (
                "surf".to_string(),
                QueryFilter::With(ComponentType::Light),
                ComponentType::Dimmer,
            ),
            disable_validation: false,
            min: Some(Component::Float(0.)),
            max: Some(Component::Float(1.)),
            step: None,
        },
    };

    let dash_id = 0;

    let watchers = dash.get_watchers(dash_id).unwrap();

    let (tx, mut rx) = mpsc::channel(10);
    for (elid, (filter, target, comp_type)) in watchers {
        println!("registering query");
        state
            .query_tx
            .send(Query {
                filter,
                target,
                kind: QueryKind::WatchAll(elid, tx.clone(), comp_type),
            })
            .await
            .unwrap();
    }
    let gs = state.clone();
    tokio::spawn(async move {
        while let Some((elid, _, _, value)) = rx.recv().await {
            // TODO we need some system of collecting all of these,
            // then shipping out all values to new viewers
            println!("elid={elid}, value={value:#?}");

            let msg = Imsg::Update(elid, value);
            let v = borsh::to_vec(&msg).unwrap();
            let dash_id = (elid >> 16) as u16;
            let res = gs.cast.send((dash_id, Message::Binary(v.into())));
            if let Err(e) = res {
                eprintln!("failed to broadcast: {e}");
            }
        }
    });

    let mut dashs = state.dashs.lock().await;
    dashs.insert(dash_id, dash);
    drop(dashs);

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
    let mut cast_rx = state.cast.subscribe();
    loop {
        tokio::select! {
            Ok((dash_id, msg)) = cast_rx.recv() => {
                // TODO check current dash ID before sending
                socket.send(msg).await.unwrap();
            }

            Some(Ok(msg)) = socket.next() => {
                match msg {
                    Message::Binary(bytes) => {
                        let msg: Imsg = borsh::from_slice(&bytes).unwrap();
                        match msg {
                            Imsg::Set(set_query) => {
                                state.query_tx.send(set_query.to_query()).await.unwrap()
                            },
                            Imsg::GetDash(dash_id) => {
                                let dashs = state.dashs.lock().await;
                                let dash = dashs.get(&dash_id).unwrap();
                                let msg = Imsg::Dash(Box::new(dash.clone()));
                                let v = borsh::to_vec(&msg).unwrap();
                                drop(dashs);
                                println!("sending: {v:?}");
                                socket.send(Message::Binary(v.into())).await.unwrap();
                            },
                            _ => {
                                eprintln!("Unexpected msg");
                            }
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
