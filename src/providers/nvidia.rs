use super::{ProviderAuthVec, ProviderFn};
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{Body, Url};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct NvidiaProvider {
    pub auth_vec: ProviderAuthVec,
}

impl Default for NvidiaProvider {
    fn default() -> Self {
        Self {
            auth_vec: Arc::new(Mutex::new(vec![])),
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
    }

    fn body_modifier(&self, body: Bytes) -> Body {
        Body::from(body)
    }

    fn get_auth(&self) -> ProviderAuthVec {
        self.auth_vec.clone()
    }
}
