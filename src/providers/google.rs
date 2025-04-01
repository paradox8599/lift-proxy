use super::{ProviderAuthVec, ProviderFn};
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{self as r, Url};
use std::sync::{Arc, Mutex};

const GOOGLE_MODELS_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai/models";
const GOOGLE_CHAT_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions";

const RESET_TIME: chrono::NaiveTime = chrono::NaiveTime::from_hms_opt(7, 0, 0).unwrap();

#[derive(Clone, Debug)]
pub struct GoogleProvider {
    pub auth_vec: ProviderAuthVec,
}

impl Default for GoogleProvider {
    fn default() -> Self {
        let auth_vec: ProviderAuthVec = Arc::new(Mutex::new(vec![]));
        crate::providers::Provider::scheduled_auth_reset(
            auth_vec.clone(),
            "Google",
            Some(RESET_TIME),
        );
        Self { auth_vec }
    }
}

impl ProviderFn for GoogleProvider {
    fn models_url(&self) -> Url {
        Url::parse(GOOGLE_MODELS_URL).unwrap()
    }

    fn chat_url(&self) -> Url {
        Url::parse(GOOGLE_CHAT_URL).unwrap()
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
