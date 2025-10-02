use crate::auth::Auth;

mod auth;
mod glacier;
mod penguin;

#[tokio::main]
async fn main() {
    // load
    let _auth = Auth::load().await.unwrap();

    // make communication channels

    // spawn glacier
    let mut _state = glacier::run().await.unwrap();

    // spawn penguin executer

    // spawn axum server

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Shutting down Igloo");
        }
    }
}
