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
use providers::{auth::init_auth, init_providers};
use proxy::webshare::init_proxies;
use routes::{
    auths::{pull_auth_route, update_auth_route},
    health, proxied_chat, proxied_models,
};
use shuttle_runtime::{SecretStore, Secrets};
use sqlx::PgPool;
use std::sync::Arc;

#[shuttle_runtime::main]
async fn main(
    #[Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let app = Arc::new(AppState::new(secrets, pool));

    init_providers(&app).await;
    init_proxies(&app).await;
    init_auth(&app).await;

    let router = Router::new()
        .route(
            "/{proxy_flag}/{provider_name}/v1/models",
            get(proxied_models),
        )
        .route(
            "/{proxy_flag}/{provider_name}/v1/chat/completions",
            post(proxied_chat),
        )
        .route("/health", get(health))
        .route("/auths", post(update_auth_route).put(pull_auth_route))
        .with_state(app);

    Ok(router.into())
}
