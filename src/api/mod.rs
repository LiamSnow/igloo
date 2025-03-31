use crate::cli::model::Cli;
use crate::state::IglooState;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{AppendHeaders, IntoResponse},
    routing::{any, get_service, post},
    Form, Json, Router,
};
use axum_extra::{
    headers::{authorization::Basic, Authorization, Cookie},
    TypedHeader,
};
use chrono::{Duration, Utc};
use reqwest::header::{LOCATION, SET_COOKIE};
use serde::Deserialize;
use std::{error::Error, sync::Arc};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

use crate::api::ws::ws_handler;

pub mod ws;

pub async fn init(state: Arc<IglooState>) -> Result<(), Box<dyn Error>> {
    let solid = ServeFile::new("./web/dist/index.html");
    let app = Router::new()
        .route("/", get_service(solid.clone()).post(post_cmd))
        .route("/login", get_service(solid).post(post_login))
        .route("/logout", post(post_logout))
        .nest_service("/assets", ServeDir::new("./web/dist/assets"))
        .route("/ws", any(ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn post_cmd(
    TypedHeader(Authorization(header)): TypedHeader<Authorization<Basic>>,
    State(state): State<Arc<IglooState>>,
    cmd_str: String,
) -> impl IntoResponse {
    // use one-time login
    let username = header.username();
    let password = header.password();
    let uid = match state.auth.validate_login(username, password) {
        Ok(Some(uid)) => uid,
        Ok(None) => {
            return (StatusCode::UNAUTHORIZED, "Invalid username or password").into_response()
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {e}")).into_response()
        }
    };

    let cmd = match Cli::parse(&cmd_str) {
        Ok(r) => r,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(e.render().to_string())).into_response(),
    };

    match cmd.dispatch(&state, Some(uid), true).await {
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

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn post_login(
    State(state): State<Arc<IglooState>>,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    match state.auth.validate_login(&form.username, &form.password) {
        Ok(Some(uid)) => {
            let token = match state.auth.token_db.add(uid).await {
                Ok(t) => t,
                Err(e) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {e}"))
                        .into_response()
                }
            };

            let expiry = Utc::now() + Duration::days(30);
            let cookie = format!(
                "auth_token={}; Path=/; Expires={}; HttpOnly; SameSite=Strict",
                token,
                expiry.format("%a, %d %b %Y %H:%M:%S GMT")
            );

            (
                StatusCode::SEE_OTHER,
                AppendHeaders([(SET_COOKIE, cookie), (LOCATION, "/".to_string())]),
            )
                .into_response()
        }
        Ok(None) => (StatusCode::UNAUTHORIZED, "Invalid username or password").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {e}")).into_response(),
    }
}

async fn post_logout(
    State(state): State<Arc<IglooState>>,
    cookies: Option<TypedHeader<Cookie>>,
) -> impl IntoResponse {
    let token = match cookies
        .as_ref()
        .and_then(|cookies| cookies.get("auth_token"))
    {
        Some(token) => token.to_string(),
        None => return (StatusCode::UNAUTHORIZED, "No token exists").into_response(),
    };

    let headers = AppendHeaders([
        (
            SET_COOKIE,
            "auth_token=; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT; HttpOnly; SameSite=Strict",
        ),
        (LOCATION, "/"),
    ]);

    match state.auth.token_db.remove(token).await {
        Ok(_) => (StatusCode::SEE_OTHER, headers).into_response(),
        Err(e) => (StatusCode::SEE_OTHER, headers, format!("Error: {e}")).into_response(),
    }
}
