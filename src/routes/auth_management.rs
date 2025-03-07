use crate::{
    app_state::AppState,
    providers::{
        auth::{sync_auth, ProviderAuth},
        ProviderFn as _,
    },
};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use std::sync::{Arc, Mutex};

pub async fn sync_auth_route(State(app): State<Arc<AppState>>) -> impl IntoResponse {
    match sync_auth(&app).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("update_auth error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update auth");
        }
    }
    (StatusCode::OK, "OK")
}

pub async fn pull_auth_route(State(app): State<Arc<AppState>>) -> impl IntoResponse {
    let all_auth: Vec<ProviderAuth> = sqlx::query_as("SELECT * FROM auth")
        .fetch_all(&app.pool)
        .await
        .unwrap();

    let providers = app.providers.lock().await;

    for provider in providers.values() {
        let auth = provider.get_auth();
        let mut auth = auth.lock().unwrap();
        auth.clear();
    }

    for auth in &all_auth {
        if let Some(provider) = providers.get(&auth.provider) {
            let provider_auth = provider.get_auth();
            let mut provider_auth = provider_auth.lock().unwrap();
            provider_auth.push(Arc::new(Mutex::new(auth.clone())));
        } else {
            tracing::warn!("Mismatched auth provider: {:?}", auth);
        }
    }

    tracing::info!("Pulled {} auths", all_auth.len());

    (StatusCode::OK, "OK")
}
