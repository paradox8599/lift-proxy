use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;

use crate::app_state::AppState;
use crate::constants::AUTH_SECRET;

pub async fn handle_auth(
    State(app): State<Arc<AppState>>,
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    match req.headers().get(axum::http::header::AUTHORIZATION) {
        Some(auth_header) => match auth_header.to_str() {
            Ok(token) if token.starts_with("Bearer ") => {
                let token = token.trim_start_matches("Bearer ");
                let auth_secret = app.secrets.get(AUTH_SECRET).unwrap();
                match token {
                    token if token == auth_secret => Ok(next.run(req).await),
                    _ => Err(StatusCode::UNAUTHORIZED),
                }
            }
            _ => Err(StatusCode::UNAUTHORIZED),
        },
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
