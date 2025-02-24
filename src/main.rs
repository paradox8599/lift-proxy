mod providers;

use axum::{
    body::Body,
    extract::Path,
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use providers::{get_provider, ProviderFn};
use reqwest::{self as r, Error};

fn create_client(proxy_addr: Option<String>, proxy_auth: Option<String>) -> r::Result<r::Client> {
    let mut client = r::Client::builder();

    if let Some(proxy_str) = proxy_addr {
        if let Ok(mut proxy) = r::Proxy::all(format!("socks5://{}", proxy_str)) {
            if let Some(proxy_auth) = proxy_auth {
                let mut s = proxy_auth.split(':');
                if let (Some(u), Some(p)) = (s.next(), s.next()) {
                    proxy = proxy.basic_auth(u, p);
                }
            }
            client = client.proxy(proxy)
        }
    }
    client.build()
}

async fn get_response_stream(resp: Result<r::Response, Error>) -> Response<Body> {
    let res = match resp {
        Ok(res) => res,
        Err(err) => {
            tracing::error!("Error sending request: {}", err);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error sending request").into_response();
        }
    };

    let status = res.status();
    let data_stream = futures::stream::try_unfold(res, |mut resp| async move {
        match resp.chunk().await {
            Ok(Some(chunk)) => Ok(Some((chunk, resp))),
            Ok(None) => Ok(None),
            Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
        }
    });

    let body = Body::from_stream(data_stream);
    match Response::builder().status(status).body(body) {
        Ok(res) => res,
        Err(err) => {
            tracing::error!("Error creating response stream: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
        }
    }
}

async fn proxied_models(
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

async fn proxied_chat(
    Path((proxy_addr, proxy_auth, provider_name)): Path<(String, String, String)>,
    mut headers: HeaderMap,
    body: String,
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

    provider.post_header_modifier(&mut headers);

    let res = client
        .post(provider.chat_url())
        .body(provider.body_modifier(&body))
        .headers(headers)
        .send()
        .await;

    get_response_stream(res).await
}

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
