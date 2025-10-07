use igloo_interface::ComponentType;
use tokio::sync::oneshot;

use crate::glacier::query::{Query, QueryKind, QueryTarget};

// mod auth;
mod glacier;
mod penguin;
// mod web;

#[tokio::main]
async fn main() {
    // let _auth = Auth::load().await.unwrap();

    let query_tx = glacier::spawn().await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let (tx, rx) = oneshot::channel();

    println!("Sending query");

    query_tx
        .send(Query {
            filter: None,
            target: QueryTarget::All,
            kind: QueryKind::GetOne(tx, ComponentType::Color),
        })
        .await
        .unwrap();

    println!("Query sent");

    let res = rx.await;

    println!("{res:#?}");

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
