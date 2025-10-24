use std::sync::Arc;

use axum::extract::ws::Message;
use igloo_interface::dash::Dashboard;
use rustc_hash::FxHashMap;
use tokio::sync::{RwLock, broadcast, mpsc};

use crate::glacier::query::Query;

// TODO glacier should register Floe #0 As internal
// We should be able to get:
//  - Current users name, uid, etc.
//  - Current time, date, datetime, weekday
// For things like Weather, etc they should be separate providers
// What should dummies be?

// mod auth;
mod glacier;
#[cfg(test)]
mod test;
mod web;

#[derive(Debug, Clone)]
pub struct GlobalState {
    query_tx: mpsc::Sender<Query>,
    cast: broadcast::Sender<(u16, Message)>,
    /// send msgs to dash tasks
    dash_tx: broadcast::Sender<(u16, DashboardRequest)>,
    dashs: Arc<RwLock<FxHashMap<String, Dashboard>>>,
}

#[derive(Debug, Clone)]
pub enum DashboardRequest {
    Shutdown,
    DumpData,
}

#[tokio::main]
async fn main() {
    // let _auth = Auth::load().await.unwrap();

    let query_tx = glacier::spawn().await.unwrap();

    let gs = GlobalState {
        query_tx,
        cast: broadcast::channel(100).0,
        dash_tx: broadcast::channel(10).0,
        dashs: Arc::new(RwLock::new(FxHashMap::default())),
    };

    let gsc = gs.clone();
    tokio::spawn(async move {
        web::run(gsc).await.unwrap();
    });

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Shutting down Igloo");
        }
    }
}
