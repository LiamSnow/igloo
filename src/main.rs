use std::{error::Error, sync::Arc};

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use cli::model::Cli;
use config::IglooConfig;
use map::IglooStack;
use serde::Serialize;

pub mod config;
pub mod device;
pub mod cli;
pub mod map;
pub mod providers;

pub const VERSION: f32 = 0.1;
pub const CONFIG_VERSION: f32 = 0.1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = IglooConfig::from_file("./config.ron").unwrap();
    if cfg.version != CONFIG_VERSION {
        panic!("Wrong config version. Got {}, expected {}.", cfg.version, CONFIG_VERSION);
    }

    let table = Arc::new(IglooStack::init(cfg).await?);
    println!("all connected!");

    let app = Router::new()
        .route("/", post(post_cmd))
        .with_state(table);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String
}

async fn post_cmd(State(table): State<Arc<IglooStack>>, cmd_str: String) -> impl IntoResponse {
    let cmd = Cli::parse(&cmd_str);

    if let Err(e) = cmd {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.render().to_string()
            }),
        ).into_response()
    }

    match cmd.unwrap().dispatch(table).await {
        Ok(v) => (
            StatusCode::OK,
            v,
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(e),
        ).into_response()
    }
}

