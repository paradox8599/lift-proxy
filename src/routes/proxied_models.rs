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

pub async fn proxied_models(
    Path((proxy_addr, proxy_auth, provider_name)): Path<(String, String, String)>,
    mut headers: HeaderMap,
) -> Response<Body> {
    let proxy_addr = (proxy_addr != "_").then_some(proxy_addr);
    let proxy_auth = (proxy_auth != "_").then_some(proxy_auth);

    let client = match create_client(proxy_addr, proxy_auth) {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("{}", e);
            return (StatusCode::BAD_REQUEST, "Failed creating reqwest client").into_response();
        }
    };

    let provider = match get_provider(&provider_name) {
        Some(provider) => provider,
        None => {
            return (StatusCode::NOT_FOUND, "Provider not found").into_response();
        }
    };

    provider.get_header_modifier(&mut headers);

    let res = client
        .get(provider.models_url())
        .headers(headers)
        .send()
        .await;

    get_response_stream(res).await
}
