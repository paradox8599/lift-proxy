use super::{ProviderAuthVec, ProviderFn};
use crate::app_state::AppState;
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{self as r, Url};
use std::sync::Arc;

const NVIDIA_MODELS_URL: &str = "https://integrate.api.nvidia.com/v1/models";
const NVIDIA_CHAT_URL: &str = "https://integrate.api.nvidia.com/v1/chat/completions";

pub struct NvidiaProvider {
    pub auth_vec: ProviderAuthVec,
}

impl NvidiaProvider {
    pub fn new(_app: Arc<AppState>) -> Self {
        tracing::debug!("{}", super::AuthProviderName::Nvidia.to_string());
        Self {
            auth_vec: ProviderAuthVec::default(),
        }
    }
}

impl ProviderFn for NvidiaProvider {
    fn models_url(&self) -> Url {
        Url::parse(NVIDIA_MODELS_URL).unwrap()
    }

    fn chat_url(&self) -> Url {
        Url::parse(NVIDIA_CHAT_URL).unwrap()
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
