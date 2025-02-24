use axum::{
    body::Body,
    extract::Path,
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
};

use crate::{
    providers::{get_provider, ProviderFn},
    utils::{create_client, get_response_stream},
};

pub async fn proxied_chat(
    Path((proxy_addr, proxy_auth, provider_name)): Path<(String, String, String)>,
    mut headers: HeaderMap,
    body: String,
) -> Response<Body> {
    tracing::info!("[POST] {} - {}", proxy_addr, provider_name);

    let proxy_addr = (proxy_addr != "_").then_some(proxy_addr);
    let proxy_auth = (proxy_auth != "_").then_some(proxy_auth);

    let info = format!(
        "{} - {}",
        proxy_addr.clone().unwrap_or("_".to_owned()),
        provider_name
    );

    let client = match create_client(proxy_addr, proxy_auth) {
        Ok(client) => client,
        Err(e) => {
            let msg = "Failed creating reqwest client";
            tracing::error!("[{}] {}: {}", info, msg, e);
            return (StatusCode::BAD_REQUEST, msg).into_response();
        }
    };

    let provider = match get_provider(&provider_name) {
        Some(provider) => provider,
        None => {
            let msg = "Provider not found";
            tracing::warn!("[{}] {}", info, msg);
            return (StatusCode::NOT_FOUND, msg).into_response();
        }
    };

    provider.post_header_modifier(&mut headers);

    let res = client
        .post(provider.chat_url())
        .body(provider.body_modifier(&body))
        .headers(headers)
        .send()
        .await;

    get_response_stream(res).await
}
