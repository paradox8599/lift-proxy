use super::{ProviderAuth, ProviderFn};
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{Body, Url};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct NvidiaProvider {
    pub auth: Arc<Mutex<Vec<ProviderAuth>>>,
}

impl Default for NvidiaProvider {
    fn default() -> Self {
        Self {
            auth: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl ProviderFn for NvidiaProvider {
    fn models_url(&self) -> Url {
        Url::parse("https://integrate.api.nvidia.com/v1/models").expect("Nvidia chat url")
    }

    fn chat_url(&self) -> Url {
        Url::parse("https://integrate.api.nvidia.com/v1/chat/completions").expect("Nvidia chat url")
    }

    fn get_header_modifier(&self, headers: &mut HeaderMap) {
        headers.clear();
    }

    fn post_header_modifier(&self, headers: &mut HeaderMap) {
        headers.remove("host");
        headers.remove("user-agent");
        headers.insert("content-type", "application/json".parse().expect(""));

        // TODO: apply auth
    }

    fn body_modifier(&self, body: Bytes) -> Body {
        Body::from(body)
    }
}
