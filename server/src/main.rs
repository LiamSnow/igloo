use std::sync::Arc;

use axum::extract::ws::Message;
use rustc_hash::FxHashMap;
use tokio::sync::{Mutex, broadcast, mpsc};

use crate::{dash::model::Dashboard, glacier::query::Query};

// TODO glacier should register Floe #0 As internal
// We should be able to get:
//  - Current users name, uid, etc.
//  - Current time, date, datetime, weekday
// For things like Weather, etc they should be separate providers
// What should dummies be?

// mod auth;
mod dash;
mod glacier;
#[cfg(test)]
mod test;
mod web;

#[derive(Clone)]
pub struct GlobalState {
    query_tx: mpsc::Sender<Query>,
    cast: broadcast::Sender<(u16, Message)>,
    dashs: Arc<Mutex<FxHashMap<u16, Dashboard>>>,
}

#[tokio::main]
async fn main() {
    // let _auth = Auth::load().await.unwrap();

    let query_tx = glacier::spawn().await.unwrap();

    let gs = GlobalState {
        query_tx,
        cast: broadcast::channel(100).0,
        dashs: Arc::new(Mutex::new(FxHashMap::default())),
    };

    let gsc = gs.clone();
    tokio::spawn(async move {
        web::run(gsc).await.unwrap();
    });

    // let (tx, mut rx) = mpsc::channel(10);
    // gs.query_tx
    //     .send(Query {
    //         filter: QueryFilter::With(ComponentType::Light),
    //         target: QueryTarget::Group(GroupID::from_parts(1, 0)),
    //         kind: QueryKind::WatchAll(0, tx, ComponentType::Dimmer),
    //     })
    //     .await
    //     .unwrap();

    // while let Some(update) = rx.recv().await {
    //     // println!("GOT update: {update:?}");
    //     let mut ws_txs = gs.ws_txs.lock().await;
    //     for ws_tx in ws_txs.iter_mut() {
    //         let res = ws_tx
    //             .send(Message::Text(format!("{:?}", update).into()))
    //             .await;
    //         if let Err(e) = res {
    //             eprintln!("ws send err: {e}");
    //         }
    //     }
    // }

    // tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // let (tx, rx) = oneshot::channel();

    // println!("Sending query");

    // query_tx
    //     .send(Query {
    //         filter: None,
    //         target: QueryTarget::All,
    //         kind: QueryKind::GetAll(tx, ComponentType::Color),
    //     })
    //     .await
    //     .unwrap();

    // println!("Query sent");

    // let res = rx.await;

    // println!("{res:#?}");

    // {
    //     let state = shared_state.lock().await;

    //     state
    //         .dispatch_query(Query {
    //             filter: QueryFilter::With(ComponentType::Light),
    //             area: glacier::query::Area::All,
    //             kind: glacier::query::QueryKind::Set(vec![Component::Switch(true)]),
    //             started_at: Instant::now(),
    //         })
    //         .await;
    // }

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Shutting down Igloo");
        }
    }
}
