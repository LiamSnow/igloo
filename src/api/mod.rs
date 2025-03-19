use crate::cli::model::Cli;
use crate::state::IglooState;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{AppendHeaders, IntoResponse},
    routing::{any, post},
    Json, Router,
};
use std::{error::Error, sync::Arc};
use tokio::net::TcpListener;

use crate::api::ws::ws_handler;

pub mod ws;

pub async fn init(state: Arc<IglooState>) -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        .route("/", post(post_cmd))
        .route("/ws", any(ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn post_cmd(State(state): State<Arc<IglooState>>, cmd_str: String) -> impl IntoResponse {
    //TODO single-time login
    let uid = *state.auth.uid_lut.get("liams").unwrap(); //FIXME

    let cmd = match Cli::parse(&cmd_str) {
        Ok(r) => r,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(e.render().to_string())).into_response(),
    };

    match cmd.dispatch(&state, uid, true).await {
        Ok(Some(body)) => (
            StatusCode::OK,
            AppendHeaders([(header::CONTENT_TYPE, "application/json")]),
            body,
        )
            .into_response(),
        Ok(None) => (StatusCode::OK).into_response(),
        //TODO log error
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
    }
}
