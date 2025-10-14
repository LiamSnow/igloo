use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::{any, get},
};
use axum_extra::{TypedHeader, headers::Cookie};
use futures_util::{SinkExt, StreamExt};
use igloo_interface::{Component, ComponentType};
use maud::Markup;
use rustc_hash::FxHashMap;
use smallvec::smallvec;
use std::{collections::HashMap, error::Error, sync::OnceLock};
use tokio::{net::TcpListener, sync::mpsc};

use crate::{
    GlobalState,
    dash::model::{Dashboard, Element},
    glacier::{
        query::{Query, QueryFilter, QueryKind, QueryTarget},
        tree::{DeviceID, GroupID},
    },
    web::page::wrap_page,
};

mod page;

static DASH: OnceLock<Markup> = OnceLock::new();

async fn d() -> Markup {
    DASH.get().unwrap().clone()
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

    let dash_id = 69;
    let (markup, watchers, setters) = dash.render(&dash_id).unwrap();

    let mut s = state.dash_setters.lock().await;
    s.insert(dash_id, setters);
    drop(s);

    DASH.get_or_init(|| wrap_page(markup));

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
            println!("elid={elid}, value={value:#?}");

            let Some(inner) = value.inner_string() else {
                eprintln!("{value:#?} cannot be watched");
                continue;
            };

            let mut ws_txs = gs.ws_txs.lock().await;
            for ws_tx in ws_txs.iter_mut() {
                let res = ws_tx
                    .send(Message::Text(format!("{dash_id},{elid},{inner}",).into()))
                    .await;
                if let Err(e) = res {
                    eprintln!("ws send err: {e}");
                }
            }
        }
    });

    let app = Router::new()
        .route("/", get(d))
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
    socket: WebSocket,
) {
    let (ws_tx, mut ws_rx) = socket.split();

    let mut ws_txs = state.ws_txs.lock().await;
    ws_txs.push(ws_tx);
    drop(ws_txs);

    while let Some(Ok(msg)) = ws_rx.next().await {
        match msg {
            Message::Text(msg) => {
                let parts: Vec<_> = msg.split(',').collect();
                if parts.len() != 3 {
                    eprintln!("invalid msg: {msg}");
                    continue;
                }

                let Ok(dash_id) = parts[0].parse() else {
                    eprintln!("failed to parse dash_id");
                    continue;
                };
                let Ok(elid) = parts[1].parse() else {
                    eprintln!("failed to parse elid");
                    continue;
                };
                let value = parts[2];

                let gmap = state.dash_setters.lock().await;
                let Some(dmap) = gmap.get(&dash_id) else {
                    eprintln!("invalid dash id: {dash_id}");
                    continue;
                };

                let Some(p) = dmap.get(&elid) else {
                    eprintln!("invalid elid: {elid}");
                    continue;
                };

                let (filter, target, comp_type) = p.clone();

                drop(gmap);

                state
                    .query_tx
                    .send(Query {
                        filter,
                        target,
                        kind: QueryKind::Set(smallvec![]),
                    })
                    .await
                    .unwrap();

                // state
                //     .query_tx
                //     .send(Query {
                //         filter: QueryFilter::With(ComponentType::Light),
                //         target: QueryTarget::Group(GroupID::from_parts(1, 0)),
                //         kind: QueryKind::Set(smallvec![Component::Dimmer(bri)]),
                //     })
                //     .await
                //     .unwrap();
            }
            Message::Close(_) => {
                break;
            }
            _ => {}
        }
    }
}
