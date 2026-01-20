// TODO glacier should register Extension #0 As internal
// We should be able to get:
//  - Current users name, uid, etc.
//  - Current time, date, datetime, weekday
// For things like Weather, etc they should be separate providers
// What should dummies be?

use igloo_interface::{
    ComponentType,
    id::{GenerationalID, GroupID},
    query::{
        ComponentQuery, DeviceFilter, DeviceGroupFilter, EntityFilter, EntityIDFilter, IDFilter,
        OneShotQuery, TypeFilter, WatchComponentQuery, WatchQuery,
    },
    types::{IglooValue, agg::AggregationOp},
};

use crate::core::{IglooRequest, IglooResponse};

mod core;
mod ext;
mod query;
mod tree;
mod web;

#[tokio::main]
async fn main() {
    let (handle, req_tx) = core::spawn().await.unwrap();

    let (tx, rx) = kanal::bounded(50);
    req_tx.send(IglooRequest::RegisterClient(tx)).unwrap();

    let rx = rx.to_async();

    let Ok(IglooResponse::Registered { client_id }) = rx.recv().await else {
        panic!();
    };

    let query = OneShotQuery::Component(ComponentQuery {
        device_filter: DeviceFilter {
            group: DeviceGroupFilter::In(GroupID::from_comb(1)),
            ..Default::default()
        },
        entity_filter: EntityFilter {
            type_filter: Some(TypeFilter::With(ComponentType::Light)),
            ..Default::default()
        },
        action: igloo_interface::query::ComponentAction::Set(IglooValue::Real(0.7)),
        component: ComponentType::Dimmer,
        post_op: None,
        include_parents: false,
        limit: None,
    });

    // tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // req_tx
    //     .send(IglooRequest::EvalOneShot {
    //         client_id,
    //         query_id: 1,
    //         query,
    //     })
    //     .unwrap();

    let query = WatchQuery::Metadata;

    // let query = WatchQuery::Component(WatchComponentQuery {
    //     device_id: IDFilter::Any,
    //     entity_id: EntityIDFilter::Any,
    //     owner: IDFilter::Any,
    //     group: DeviceGroupFilter::In(GroupID::from_comb(1)),
    //     type_filter: Some(TypeFilter::With(ComponentType::Light)),
    //     component: ComponentType::Dimmer,
    //     post_op: Some(AggregationOp::Mean),
    // });

    req_tx
        .send(IglooRequest::SubWatch {
            client_id,
            query_id: 0,
            query,
        })
        .unwrap();

    while let Ok(res) = rx.recv().await {
        dbg!(res);
    }

    // if let Err(e) = web::run(req_tx.clone()).await {
    //     eprintln!("Error running web: {e}");
    // }

    // tokio::signal::ctrl_c().await.unwrap();
    // println!("SHUTTING DOWN");
    // req_tx.send(IglooRequest::Shutdown).unwrap();
    // handle.join().unwrap();
}
