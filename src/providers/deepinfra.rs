use super::{ProviderAuthVec, ProviderFn};
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{Body, Url};
use std::sync::{Arc, Mutex};

const DEEPINFRA_MODELS_URL: &str = "https://api.deepinfra.com/v1/openai/models";
const DEEPINFRA_CHAT_URL: &str = "https://api.deepinfra.com/v1/openai/chat/completions";

#[derive(Clone, Debug, Default)]
pub struct DeepinfraProvider;

impl ProviderFn for DeepinfraProvider {
    fn models_url(&self) -> Url {
        Url::parse(DEEPINFRA_MODELS_URL).unwrap()
    }

    fn chat_url(&self) -> Url {
        Url::parse(DEEPINFRA_CHAT_URL).unwrap()
    }

    fn get_header_modifier(&self, headers: &mut HeaderMap) {
        headers.clear();
    }

    fn post_header_modifier(&self, headers: &mut HeaderMap) {
        headers.clear();
        headers.insert("content-type", "application/json".parse().unwrap());
    }

    fn body_modifier(&self, body: Bytes) -> Body {
        Body::from(body)
    }

    fn get_auth(&self) -> ProviderAuthVec {
        Arc::new(Mutex::new(vec![]))
    }

    async fn get_response(
        &self,
        _body: axum::body::Bytes,
        resp: reqwest::Response,
    ) -> axum::http::Response<axum::body::Body> {
        crate::utils::get_response_stream(resp).await
    }
}
