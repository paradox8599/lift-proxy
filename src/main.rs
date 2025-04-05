mod app_state;
mod db;
mod env;
mod middlewares;
mod providers;
mod proxy;
mod routes;
mod utils;

use app_state::AppState;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use middlewares::handle_auth;
use providers::{auth::init_auth, init_providers};
use proxy::webshare::init_proxies;
use routes::{
    auth_management::{pull_auth_route, sync_auth_route},
    health, proxied_chat, proxied_models, toggle_show_chat,
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let app = Arc::new(AppState::new().await);

    init_providers(&app).await;
    init_auth(&app).await;
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
        .route("/", get(health))
        .route("/show_chat", post(toggle_show_chat))
        .route("/auths", post(sync_auth_route).put(pull_auth_route))
        .layer(middleware::from_fn_with_state(app.clone(), handle_auth))
        .with_state(app.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
