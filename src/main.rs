mod app_state;
mod providers;
mod proxy;
mod routes;
mod utils;

use std::sync::Arc;

use app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use proxy::webshare::update_proxies;
use routes::{proxied_chat, proxied_models};
use shuttle_runtime::{SecretStore, Secrets};
use tokio::sync::Mutex;

use rand::rngs::SmallRng;
use rand::SeedableRng;

#[shuttle_runtime::main]
async fn main(#[Secrets] secrets: SecretStore) -> shuttle_axum::ShuttleAxum {
    let app = Arc::new(AppState {
        secrets,
        rng: Mutex::new(SmallRng::from_os_rng()),
        last_synced_at: Mutex::new(tokio::time::Instant::now()),
        proxies: Mutex::new(vec![]),
    });

    if let Err(e) = update_proxies(&app).await {
        panic!("Error init proxies: {}", e);
    }

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
