use crate::app_state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

pub async fn toggle_show_chat(State(app): State<Arc<AppState>>) -> impl IntoResponse {
    let mut show_chat = app.show_chat.lock().await;
    *show_chat = !*show_chat;
    tracing::info!("Show chat: {}", show_chat);
    (StatusCode::OK, show_chat.to_string())
}
