use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use reqwest as r;

pub async fn get_body_stream(resp: r::Response) -> Body {
    let data_stream = futures::stream::try_unfold(resp, |mut resp| async move {
        match resp.chunk().await {
            Ok(Some(chunk)) => Ok(Some((chunk, resp))),
            Ok(None) => Ok(None),
            Err(err) => Err(std::io::Error::other(err)),
        }
    });

    Body::from_stream(data_stream)
}

pub async fn get_response_stream(resp: r::Response) -> Response<Body> {
    let status = resp.status();
    let headers = resp.headers().clone();
    let body = get_body_stream(resp).await;
    let mut res = match Response::builder().status(status).body(body) {
        Ok(res) => res,
        Err(err) => {
            tracing::error!("Error creating response stream: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
        }
    };
    let resp_headers = res.headers_mut();
    resp_headers.extend(headers.iter().map(|(k, v)| (k.clone(), v.clone())));
    res
}
