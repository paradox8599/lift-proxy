use crate::app_state::AppState;

use super::{ProviderAuthVec, ProviderFn};
use axum::{body::Bytes, http::HeaderMap};
use chrono::{DateTime, Utc};
use reqwest::{self as r, Url};
use std::sync::{Arc, Mutex};

const GOOGLE_MODELS_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai/models";
const GOOGLE_CHAT_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions";

const RESET_TIME: chrono::NaiveTime = chrono::NaiveTime::from_hms_opt(7, 0, 0).unwrap();

pub struct GoogleProvider {
    pub app: Arc<AppState>,
    pub auth_vec: ProviderAuthVec,
    pub last_authed_at: Arc<Mutex<DateTime<Utc>>>,
}

impl GoogleProvider {
    pub fn new(app: Arc<AppState>) -> Self {
        Self {
            app,
            auth_vec: ProviderAuthVec::default(),
            last_authed_at: Arc::new(Mutex::new(Utc::now())),
        }
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
        let mut last_authed_at = self.last_authed_at.lock().unwrap();
        super::Provider::handle_auth_reset(
            self.app.clone(),
            self.auth_vec.clone(),
            super::AuthProviderName::Google,
            *last_authed_at,
            RESET_TIME,
        );
        *last_authed_at = Utc::now();
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
