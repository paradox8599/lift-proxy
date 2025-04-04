use std::sync::Arc;

use crate::app_state::AppState;

use super::{ProviderAuthVec, ProviderFn};
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{Body, Url};

const DEEPINFRA_MODELS_URL: &str = "https://api.deepinfra.com/v1/openai/models";
const DEEPINFRA_CHAT_URL: &str = "https://api.deepinfra.com/v1/openai/chat/completions";

pub struct DeepinfraProvider;

impl DeepinfraProvider {
    pub fn new(_app: Arc<AppState>) -> Self {
        tracing::debug!("{}", super::AuthProviderName::Deepinfra.to_string());
        Self {}
    }
}

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
        ProviderAuthVec::default()
    }

    async fn get_response(
        &self,
        _body: axum::body::Bytes,
        resp: reqwest::Response,
    ) -> axum::http::Response<axum::body::Body> {
        crate::utils::stream_body::get_response_stream(resp).await
    }
}
