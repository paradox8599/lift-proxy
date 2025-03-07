use super::{ProviderAuthVec, ProviderFn};
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{Body, Url};
use std::sync::{Arc, Mutex};

const DZMM_MODELS_URL: &str = "https://www.gpt4novel.com/api/xiaoshuoai/ext/v1/models";
const DZMM_CHAT_URL: &str = "https://www.gpt4novel.com/api/xiaoshuoai/ext/v1/chat/completions";

#[derive(Clone, Debug)]
pub struct DzmmProvider {
    pub auth_vec: ProviderAuthVec,
}

impl Default for DzmmProvider {
    fn default() -> Self {
        Self {
            auth_vec: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl ProviderFn for DzmmProvider {
    fn models_url(&self) -> Url {
        Url::parse(DZMM_MODELS_URL).expect("dzmm chat url")
    }

    fn chat_url(&self) -> Url {
        Url::parse(DZMM_CHAT_URL).expect("dzmm chat url")
    }

    fn get_header_modifier(&self, headers: &mut HeaderMap) {
        headers.clear();
    }

    fn post_header_modifier(&self, headers: &mut HeaderMap) {
        headers.remove("host");
        headers.remove("user-agent");
        headers.insert("content-type", "application/json".parse().expect(""));
    }

    fn body_modifier(&self, body: Bytes) -> Body {
        Body::from(body)
    }

    fn get_auth(&self) -> ProviderAuthVec {
        self.auth_vec.clone()
    }
}
