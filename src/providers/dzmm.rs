use super::{ProviderAuth, ProviderFn};
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{Body, Url};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct DzmmProvider {
    pub auth: Arc<Mutex<Vec<ProviderAuth>>>,
}

impl Default for DzmmProvider {
    fn default() -> Self {
        Self {
            auth: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl ProviderFn for DzmmProvider {
    fn models_url(&self) -> Url {
        Url::parse("https://www.gpt4novel.com/api/xiaoshuoai/ext/v1/models").expect("dzmm chat url")
    }

    fn chat_url(&self) -> Url {
        Url::parse("https://www.gpt4novel.com/api/xiaoshuoai/ext/v1/chat/completions")
            .expect("dzmm chat url")
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
