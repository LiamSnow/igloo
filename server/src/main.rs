mod auth;
mod components;
mod dashboard;
mod glacier;
mod penguin;

#[tokio::main]
async fn main() {
    // load
    let auth = auth::Auth::load().await.unwrap();
    let state = glacier::GlacierState::load().await.unwrap();

    // make communication channels

    // spawn glacier
    glacier::spawn(state).await.unwrap();

    // spawn penguin executer

    // spawn axum server

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Shutting down Igloo");
        }
    }
}
