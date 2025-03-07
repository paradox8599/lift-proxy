use crate::{
    app_state::AppState,
    providers::{auth::update_auth_state_on_response, ProviderFn},
    proxy::webshare::disable_failed_proxy,
    routes::handle_proxy_flag,
    utils::get_response_stream,
};
use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn proxied_chat(
    State(app): State<Arc<AppState>>,
    Path((proxy_flag, provider_name)): Path<(String, String)>,
    mut headers: HeaderMap,
    body: Bytes,
) -> Response<Body> {
    tracing::info!("[POST] {} {}", proxy_flag, provider_name);

    let (client, proxy) = match handle_proxy_flag(&app, &proxy_flag).await {
        Ok(result) => result,
        Err(e) => {
            let msg = format!("Failed to create reqwest client: {}", e);
            tracing::error!("{}", msg);
            return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response();
        }
    };

    let provider = match app.get_provider(&provider_name).await {
        Some(provider) => provider,
        None => {
            let msg = "Provider not found";
            return (StatusCode::NOT_FOUND, msg).into_response();
        }
    };

    provider.post_header_modifier(&mut headers);
    let auth = provider.apply_auth(&mut headers);

    let res = client
        .post(provider.chat_url())
        .body(provider.body_modifier(body))
        .headers(headers.clone())
        .send()
        .await;

    let res = match res {
        Ok(res) => res,
        Err(err) => {
            disable_failed_proxy(&app, &proxy).await;
            let msg = "Error sending request";
            tracing::error!("{}: {} - {:?}", msg, err, proxy);
            return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response();
        }
    };

    let status = res.status();
    update_auth_state_on_response(&auth, &status);
    // only disable the proxy if there is no user custom auth header
    if status == StatusCode::TOO_MANY_REQUESTS && headers.get("authorization").is_none() {
        disable_failed_proxy(&app, &proxy).await;
    }

    get_response_stream(res).await
}
