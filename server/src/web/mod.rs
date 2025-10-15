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
use igloo_interface::{
    Component, ComponentType, DeviceID, QueryFilter, QueryTarget,
    dash::{
        ColorPickerElement, ColorPickerVariant, DashQuery, DashQueryNoType, Dashboard, HAlign,
        SliderElement, VAlign, VStackElement,
    },
    ws::{ClientMessage, ElementUpdate, ServerMessage},
};
use std::{collections::HashMap, error::Error};
use tokio::{net::TcpListener, sync::mpsc};

use crate::{GlobalState, glacier::query::WatchAllQuery, web::watch::GetWatchers};

mod watch;

pub async fn run(state: GlobalState) -> Result<(), Box<dyn Error>> {
    // START TESTING CODE

    let mut targets = HashMap::default();
    targets.insert(
        "surf".to_string(),
        QueryTarget::Device(DeviceID::from_parts(0, 0)),
    );

    let mut dash = Dashboard {
        name: "test".to_string(),
        targets,
        child: VStackElement {
            justify: VAlign::Center,
            align: HAlign::Center,
            scroll: false,
            children: vec![
                SliderElement {
                    watch_id: None,
                    binding: DashQuery {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                        comp_type: ComponentType::Dimmer,
                    },
                    disable_validation: false,
                    min: Some(Component::Float(0.)),
                    max: Some(Component::Float(1.)),
                    step: None,
                }
                .into(),
                ColorPickerElement {
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::Circle,
                }
                .into(),
                ColorPickerElement {
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::HueSlider,
                }
                .into(),
                ColorPickerElement {
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::Hsl,
                }
                .into(),
            ],
        }
        .into(),
    };

    let dash_id = 0;

    let watchers = dash.attach_watchers(dash_id).unwrap();

    let (watch_tx, mut watch_rx) = mpsc::channel(10);
    for watcher in watchers {
        println!("registering query");
        state
            .query_tx
            .send(
                WatchAllQuery {
                    filter: watcher.filter,
                    target: watcher.target,
                    update_tx: watch_tx.clone(),
                    comp: watcher.comp,
                    prefix: watcher.watch_id,
                }
                .into(),
            )
            .await
            .unwrap();
    }
    let gs = state.clone();
    tokio::spawn(async move {
        while let Some((watch_id, _, _, value)) = watch_rx.recv().await {
            // TODO we need some system of collecting all of these,
            // then shipping out all values to new viewers
            let msg: ServerMessage = ElementUpdate { watch_id, value }.into();
            let bytes = borsh::to_vec(&msg).unwrap(); // FIXME unwrap
            let dash_id = (watch_id >> 16) as u16;
            let res = gs.cast.send((dash_id, Message::Binary(bytes.into())));
            if let Err(e) = res {
                eprintln!("failed to broadcast: {e}");
            }
        }
    });

    let mut dashs = state.dashboards.write().await;
    dashs.insert(dash_id, dash);
    drop(dashs);

    // END TESTING CODE

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
    let mut cur_dash = u16::MAX;
    loop {
        tokio::select! {
            Ok((dash_id, msg)) = cast_rx.recv() => {
                if dash_id != cur_dash {
                    println!("Client is on wrong dashboard, skipping.."); // FIXME remove
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
                            &mut cur_dash,
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
    cur_dash: &mut u16,
    bytes: Bytes,
) -> Result<(), Box<dyn Error>> {
    let msg: ClientMessage = borsh::from_slice(&bytes)?;

    match msg {
        ClientMessage::ExecSetQuery(q) => state.query_tx.send(q.into()).await.unwrap(),
        ClientMessage::SetDashboard(dash_id) => {
            *cur_dash = dash_id;
            if dash_id == u16::MAX {
                return Ok(());
            }

            let dashboards = state.dashboards.read().await;

            let dash = dashboards.get(&dash_id).ok_or("invalid dashboard ID")?;

            let msg: ServerMessage = (dash_id, Box::new(dash.clone())).into();
            let bytes = borsh::to_vec(&msg)?;

            drop(dashboards);

            socket.send(Message::Binary(bytes.into())).await?;
        }
    }

    Ok(())
}
