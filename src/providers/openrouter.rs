use super::{ProviderAuthVec, ProviderFn};
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{self as r, Url};
use std::sync::{Arc, Mutex};

const OPENROUTER_MODELS_URL: &str = "https://openrouter.ai/api/v1/models";
const OPENROUTER_CHAT_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

#[derive(Clone, Debug)]
pub struct OpenRouterProvider {
    pub auth_vec: ProviderAuthVec,
}

impl Default for OpenRouterProvider {
    fn default() -> Self {
        Self {
            auth_vec: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl ProviderFn for OpenRouterProvider {
    fn models_url(&self) -> Url {
        Url::parse(OPENROUTER_MODELS_URL).unwrap()
    }

    fn chat_url(&self) -> Url {
        Url::parse(OPENROUTER_CHAT_URL).unwrap()
    }

    fn get_header_modifier(&self, headers: &mut HeaderMap) {
        headers.clear();
    }

    fn post_header_modifier(&self, headers: &mut HeaderMap) {
        headers.clear();
        headers.insert("content-type", "application/json".parse().unwrap());
    }

    fn body_modifier(&self, body: Bytes) -> r::Body {
        r::Body::from(body)
    }

    fn get_auth(&self) -> ProviderAuthVec {
        self.auth_vec.clone()
    }

    async fn get_response(
        &self,
        _body: axum::body::Bytes,
        resp: reqwest::Response,
    ) -> axum::http::Response<axum::body::Body> {
        crate::utils::stream_body::get_response_stream(resp).await
    }
}
