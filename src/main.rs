mod app_state;
mod providers;
mod proxy;
mod routes;
mod utils;

use app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use providers::init_providers;
use proxy::webshare::init_proxies;
use routes::{proxied_chat, proxied_models};
use shuttle_runtime::{SecretStore, Secrets};
use std::sync::Arc;

#[shuttle_runtime::main]
async fn main(#[Secrets] secrets: SecretStore) -> shuttle_axum::ShuttleAxum {
    let app = Arc::new(AppState::new(secrets));
    init_providers(&app).await;
    init_proxies(&app).await;

    let router = Router::new()
        .route(
            "/{proxy_flag}/{provider_name}/v1/models",
            get(proxied_models),
        )
        .route(
            "/{proxy_flag}/{provider_name}/v1/chat/completions",
            post(proxied_chat),
        )
        .with_state(app);

    Ok(router.into())
}
