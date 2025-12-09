// TODO glacier should register Extension #0 As internal
// We should be able to get:
//  - Current users name, uid, etc.
//  - Current time, date, datetime, weekday
// For things like Weather, etc they should be separate providers
// What should dummies be?

use crate::core::{IglooRequest, IglooResponse};
use igloo_interface::{
    ComponentType,
    id::{GenerationalID, GroupID},
    query::{
        ComponentAction, ComponentQuery, DeviceFilter, DeviceGroupFilter, EntityFilter, Query,
        TypeFilter,
    },
    types::agg::AggregationOp,
};
use std::time::Duration;
use tokio::time::sleep;

mod core;
mod ext;
mod query;
mod tree;

#[tokio::main]
async fn main() {
    let (handle, req_tx) = core::spawn().await.unwrap();

    // sleep(Duration::from_secs(5)).await;

    let (res_tx, res_rx) = kanal::bounded(200);
    req_tx.send(IglooRequest::RegisterClient(res_tx)).unwrap();
    let res_rx = res_rx.to_async();

    let IglooResponse::Registered { client_id } = res_rx.recv().await.unwrap() else {
        panic!()
    };

    let task = tokio::spawn(async move {
        while let Ok(res) = res_rx.recv().await {
            println!("got {res:?}");
        }
    });

    let query = Query::Component(ComponentQuery {
        device_filter: DeviceFilter {
            group: DeviceGroupFilter::In(GroupID::from_comb(1)),
            ..Default::default()
        },
        entity_filter: EntityFilter {
            type_filter: Some(TypeFilter::With(ComponentType::Light)),
            ..Default::default()
        },
        action: ComponentAction::ObserveValue,
        component: ComponentType::Dimmer,
        post_op: Some(AggregationOp::Mean),
        include_parents: false,
        limit: None,
    });

    req_tx
        .send(IglooRequest::EvalQuery {
            client_id,
            query_id: 0,
            query: query.clone(),
        })
        .unwrap();

    handle.join().unwrap();
    task.await.unwrap();

    println!("SHUTTING DOWN");

    req_tx.send(IglooRequest::Shutdown).unwrap();
}
