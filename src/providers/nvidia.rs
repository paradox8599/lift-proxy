use super::ProviderFn;
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{Body, Url};

pub struct NvidiaProvider;

impl ProviderFn for NvidiaProvider {
    fn models_url(&self) -> Url {
        Url::parse("https://integrate.api.nvidia.com/v1/models").unwrap()
    }

    fn chat_url(&self) -> Url {
        Url::parse("https://integrate.api.nvidia.com/v1/chat/completions").unwrap()
    }

    fn get_header_modifier(&self, headers: &mut HeaderMap) {
        headers.clear();
    }

    fn post_header_modifier(&self, _headers: &mut HeaderMap) {}

    fn body_modifier(&self, body: Bytes) -> Body {
        Body::from(body)
    }
}
