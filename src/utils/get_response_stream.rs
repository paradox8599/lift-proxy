use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use reqwest as r;

pub async fn get_response_stream(resp: r::Response) -> Response<Body> {
    let status = resp.status();
    let data_stream = futures::stream::try_unfold(resp, |mut resp| async move {
        match resp.chunk().await {
            Ok(Some(chunk)) => Ok(Some((chunk, resp))),
            Ok(None) => Ok(None),
            Err(err) => Err(std::io::Error::other(err)),
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
