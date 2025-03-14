mod app_state;
mod constants;
mod providers;
mod proxy;
mod routes;
mod utils;

use app_state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    routing::{get, post},
    Router,
};
use providers::{
    auth::{init_auth, regular_auth_state_update},
    init_providers,
};
use proxy::webshare::init_proxies;
use routes::{
    auth_management::{pull_auth_route, sync_auth_route},
    health, proxied_chat, proxied_models,
};
use shuttle_runtime::{SecretStore, Secrets};
use sqlx::PgPool;
use std::sync::Arc;

async fn handle_auth(
    State(app): State<Arc<AppState>>,
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    match req.headers().get(axum::http::header::AUTHORIZATION) {
        Some(auth_header) => match auth_header.to_str() {
            Ok(token) if token.starts_with("Bearer ") => {
                let token = token.trim_start_matches("Bearer ");
                let auth_secret = app.secrets.get(constants::AUTH_SECRET).unwrap();
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

#[shuttle_runtime::main]
async fn main(
    #[Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let app = Arc::new(AppState::new(secrets, pool));

    init_providers(&app).await;
    init_auth(&app).await;
    init_proxies(&app).await;

    regular_auth_state_update(&app);

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
        .route("/auths", post(sync_auth_route).put(pull_auth_route))
        .layer(middleware::from_fn_with_state(app.clone(), handle_auth))
        .with_state(app.clone());

    Ok(router.into())
}
