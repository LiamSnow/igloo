use crate::{auth::Auth, glacier::GlacierSupervisor};

mod auth;
mod components;
mod dashboard;
mod glacier;
mod penguin;

#[tokio::main]
async fn main() {
    // load
    let _auth = Auth::load().await.unwrap();

    // make communication channels

    // spawn glacier
    let glacier = GlacierSupervisor::new().await.unwrap();

    // spawn penguin executer

    // spawn axum server

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Shutting down Igloo");
            glacier.shutdown().await;
        }
    }
}
