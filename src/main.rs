mod providers;
mod routes;
mod syncing;
mod utils;

use axum::{
    routing::{get, post},
    Router,
};
use routes::{proxied_chat, proxied_models};
use std::time::Duration;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(60)).await;
        loop {
            tracing::info!("[Sync] Start syncing...");
            match syncing::sync(&secrets).await {
                Ok(_) => tracing::info!("[Sync] Done"),
                Err(e) => tracing::error!("[Sync] Error: {}", e),
            }
            tokio::time::sleep(Duration::from_secs(5 * 60)).await;
        }
    });

    let router = Router::new()
        .route(
            "/{proxy_addr}/{proxy_auth}/{provider_name}/v1/models",
            get(proxied_models),
        )
        .route(
            "/{proxy_addr}/{proxy_auth}/{provider_name}/v1/chat/completions",
            post(proxied_chat),
        );

    Ok(router.into())
}
