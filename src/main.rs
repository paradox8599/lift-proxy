mod providers;
mod routes;
mod utils;

use axum::{
    routing::{get, post},
    Router,
};
use routes::{proxied_chat, proxied_models};

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
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
