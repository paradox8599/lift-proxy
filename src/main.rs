use axum::{
    body::Body,
    extract::Path,
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use reqwest::{self as r, Error};
use tracing::info;

fn get_client(proxy_addr: Option<String>, proxy_auth: Option<String>) -> r::Client {
    let mut client = r::Client::builder();
    if let Some(proxy_str) = proxy_addr {
        let mut proxy = r::Proxy::all(format!("socks5://{}", proxy_str)).unwrap();
        if let Some(proxy_auth) = proxy_auth {
            let mut s = proxy_auth.split(':');
            if let (Some(u), Some(p)) = (s.next(), s.next()) {
                proxy = proxy.basic_auth(u, p);
            }
        }
        client = client.proxy(proxy)
    }
    client.build().unwrap()
}

async fn get_response_stream(resp: Result<r::Response, Error>) -> Response<Body> {
    let res = match resp {
        Ok(res) => res,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response();
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
    Response::builder().status(status).body(body).unwrap()
}

async fn proxied_get(
    Path((proxy_addr, proxy_auth, addr)): Path<(String, String, String)>,
) -> Response<Body> {
    let addr = match addr.rfind("/v1") {
        Some(index) => addr[..index].to_string() + &addr[index + 3..],
        None => addr,
    };
    info!("[POST] {} -> {}", proxy_addr, addr);
    let client = get_client(Some(proxy_addr), Some(proxy_auth));
    let url = r::Url::parse(&format!("https://{}", addr)).unwrap();
    let res = client.get(url).send().await;
    get_response_stream(res).await
}

async fn proxied_post(
    Path((proxy_addr, proxy_auth, addr)): Path<(String, String, String)>,
    body: String,
) -> Response<Body> {
    let addr = match addr.rfind("/v1") {
        Some(index) => addr[..index].to_string() + &addr[index + 3..],
        None => addr,
    };
    info!("[GET]  {} -> {}", proxy_addr, addr);
    let client = get_client(Some(proxy_addr), Some(proxy_auth));
    let url = r::Url::parse(&format!("https://{}", addr)).unwrap();
    let res = client
        .post(url)
        .body(body)
        .header("Content-Type", "application/json")
        .send()
        .await;
    get_response_stream(res).await
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/{proxy_addr}/{proxy_auth}/{*addr}", get(proxied_get))
        .route("/{proxy_addr}/{proxy_auth}/{*addr}", post(proxied_post));

    Ok(router.into())
}
