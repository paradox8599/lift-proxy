use crate::{
    app_state::AppState,
    providers::{get_provider, ProviderFn},
    proxy::webshare::create_proxied_client,
    utils::get_response_stream,
};
use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
};
use reqwest as r;
use std::sync::Arc;

pub async fn proxied_chat(
    State(app): State<Arc<AppState>>,
    Path((proxy_flag, provider_name)): Path<(String, String)>,
    mut headers: HeaderMap,
    body: Bytes,
) -> Response<Body> {
    tracing::info!("[POST] {} {}", proxy_flag, provider_name);

    let client = match proxy_flag.as_str() {
        "_" => r::Client::builder().build().expect(""),
        "r" => match create_proxied_client(&app).await {
            Ok(client) => client,
            Err(e) => {
                let msg = "Failed to create reqwest client";
                tracing::error!("{}: {}", msg, e);
                return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response();
            }
        },
        _ => return (StatusCode::NOT_FOUND).into_response(),
    };

    let provider = match get_provider(&provider_name) {
        Some(provider) => provider,
        None => {
            let msg = "Provider not found";
            return (StatusCode::NOT_FOUND, msg).into_response();
        }
    };

    provider.post_header_modifier(&mut headers);

    let res = client
        .post(provider.chat_url())
        .body(provider.body_modifier(body))
        .headers(headers)
        .send()
        .await;

    get_response_stream(res).await
}
