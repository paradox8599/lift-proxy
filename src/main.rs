mod providers;
mod routes;
mod syncing;
mod utils;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use routes::{proxied_chat, proxied_models};
use shuttle_runtime::{SecretStore, Secrets};
use tokio::{sync::Mutex, time::Instant};
use utils::AppState;

#[shuttle_runtime::main]
async fn main(#[Secrets] secrets: SecretStore) -> shuttle_axum::ShuttleAxum {
    let app = Arc::new(AppState {
        secrets,
        last_synced_at: Mutex::new(Instant::now()),
    });

    let router = Router::new()
        .route(
            "/{proxy_addr}/{proxy_auth}/{provider_name}/v1/models",
            get(proxied_models),
        )
        .route(
            "/{proxy_addr}/{proxy_auth}/{provider_name}/v1/chat/completions",
            post(proxied_chat),
        )
        .with_state(app);

    Ok(router.into())
}
