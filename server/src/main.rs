// TODO glacier should register Floe #0 As internal
// We should be able to get:
//  - Current users name, uid, etc.
//  - Current time, date, datetime, weekday
// For things like Weather, etc they should be separate providers
// What should dummies be?

use crate::core::{IglooRequest, IglooResponse};
use igloo_interface::{
    ComponentType,
    query::{ComponentAction, ComponentQuery, DeviceFilter, EntityFilter, Query},
    types::IglooValue,
};
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
    req_tx.send(IglooRequest::Register(res_tx)).unwrap();
    let res_rx = res_rx.to_async();

    let IglooResponse::Registered { client_id } = res_rx.recv().await.unwrap() else {
        panic!()
    };

    tokio::spawn(async move {
        while let Ok(_resp) = res_rx.recv().await {
            // println!("got {resp:?}");
        }
    });

    let mut bri = 0.;
    loop {
        let query = Query::Component(ComponentQuery {
            device_filter: DeviceFilter::default(),
            entity_filter: EntityFilter::default(),
            action: ComponentAction::Set(IglooValue::Real(bri)),
            component: ComponentType::Dimmer,
            post_op: None,
            include_parents: false,
            limit: None,
        });

        req_tx
            .send(IglooRequest::Eval {
                client_id,
                query_id: 0,
                query: query.clone(),
            })
            .unwrap();

        bri += 0.5;

        if bri > 1.0 {
            break;
        }

        sleep(Duration::from_millis(200)).await;
    }

    println!("DONE");

    sleep(Duration::from_secs(5)).await;

    println!("SHUTTING DOWN");

    req_tx.send(IglooRequest::Shutdown).unwrap();

    handle.join().unwrap();
}
