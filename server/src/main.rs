mod auth;
mod components;
mod dashboard;
mod glacier;
mod penguin;

#[tokio::main]
async fn main() {
    // load
    let auth = auth::Auth::load().await.unwrap();
    let state = glacier::State::load().await.unwrap();

    // make communication channels

    // spawn glacier

    // spawn penguin executer

    // spawn axum server
}
