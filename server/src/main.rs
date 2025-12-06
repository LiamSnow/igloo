// TODO glacier should register Floe #0 As internal
// We should be able to get:
//  - Current users name, uid, etc.
//  - Current time, date, datetime, weekday
// For things like Weather, etc they should be separate providers
// What should dummies be?

use crate::core::{IglooRequest, IglooResponse};
use igloo_interface::query::{DeviceAction, DeviceFilter, DeviceQuery, Query};
use std::time::Duration;
use tokio::time::sleep;

mod core;
mod floe;
mod query;
mod tree;

#[tokio::main]
async fn main() {
    let (handle, req_tx) = core::spawn().await.unwrap();

    sleep(Duration::from_secs(5)).await;

    let (res_tx, res_rx) = kanal::bounded(200);
    req_tx.send(IglooRequest::RegisterClient(res_tx)).unwrap();
    let res_rx = res_rx.to_async();

    let IglooResponse::Registered { client_id } = res_rx.recv().await.unwrap() else {
        panic!()
    };

    tokio::spawn(async move {});

    let query = Query::Device(DeviceQuery {
        filter: DeviceFilter::default(),
        action: DeviceAction::Snapshot(true),
        limit: Some(1),
    });

    req_tx
        .send(IglooRequest::EvalQuery {
            client_id,
            query_id: 0,
            query: query.clone(),
        })
        .unwrap();

    let resp = res_rx.recv().await;
    println!("got {resp:?}");

    println!("SHUTTING DOWN");

    req_tx.send(IglooRequest::Shutdown).unwrap();

    handle.join().unwrap();
}
