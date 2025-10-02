use igloo_interface::{Component, ComponentType};

use crate::{
    auth::Auth,
    glacier::query::{GlobalQueryRequest, QueryFilter},
};

mod auth;
mod glacier;
mod penguin;

#[tokio::main]
async fn main() {
    // load
    let _auth = Auth::load().await.unwrap();

    // make communication channels

    // spawn glacier
    let mut shared_state = glacier::run().await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    {
        let state = shared_state.lock().await;

        state
            .dispatch_query(GlobalQueryRequest {
                filter: QueryFilter::With(ComponentType::Light),
                area: glacier::query::GlobalArea::All,
                kind: glacier::query::QueryKind::Set(vec![Component::Dimmer(0.5)]),
            })
            .await;
    }

    // spawn penguin executer

    // spawn axum server

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Shutting down Igloo");
        }
    }
}
