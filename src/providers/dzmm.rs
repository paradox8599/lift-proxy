use super::{auth::ProviderAuthVec, ProviderFn};
use crate::{
    app_state::AppState,
    utils::data_types::{ChatBody, ChatResponse, Choice, Delta, StreamChunk},
};
use axum::{body::Bytes, http::HeaderMap, response::IntoResponse as _};
use chrono::{DateTime, Utc};
use reqwest::{Body, Url};
use std::sync::{Arc, Mutex};

const DZMM_MODELS_URL: &str = "https://www.gpt4novel.com/api/xiaoshuoai/ext/v1/models";
const DZMM_CHAT_URL: &str = "https://www.gpt4novel.com/api/xiaoshuoai/ext/v1/chat/completions";

// DZMM Resets free quota at 11:00AM UTC
const RESET_TIME: chrono::NaiveTime = chrono::NaiveTime::from_hms_opt(11, 0, 0).unwrap();

pub struct DzmmProvider {
    pub app: Arc<AppState>,
    pub auth_vec: ProviderAuthVec,
    pub last_authed_at: Arc<Mutex<DateTime<Utc>>>,
}
impl DzmmProvider {
    pub fn new(app: Arc<AppState>) -> Self {
        Self {
            app,
            auth_vec: ProviderAuthVec::default(),
            last_authed_at: Arc::new(Mutex::new(Utc::now())),
        }
    }
}

impl ProviderFn for DzmmProvider {
    fn models_url(&self) -> Url {
        Url::parse(DZMM_MODELS_URL).unwrap()
    }

    fn chat_url(&self) -> Url {
        Url::parse(DZMM_CHAT_URL).unwrap()
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
        let mut last_authed_at = self.last_authed_at.lock().unwrap();
        super::Provider::handle_auth_reset(
            self.app.clone(),
            self.auth_vec.clone(),
            super::AuthProviderName::Dzmm,
            *last_authed_at,
            RESET_TIME,
        );
        *last_authed_at = Utc::now();
        self.auth_vec.clone()
    }

    async fn get_response(
        &self,
        body: axum::body::Bytes,
        resp: reqwest::Response,
    ) -> axum::http::Response<axum::body::Body> {
        let body_text = String::from_utf8_lossy(&body);
        let body = match serde_json::from_str::<ChatBody>(&body_text) {
            Ok(body) => body,
            Err(e) => {
                tracing::warn!("Error parsing body: {}", e);
                return (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response();
            }
        };

        if body.stream.unwrap_or(false) {
            crate::utils::stream_body::get_response_stream(resp).await
        } else {
            tracing::info!("Parsing DZMM non-streaming response");
            let mut resp = resp;
            let mut resp_body = String::new();
            while let Ok(Some(chunk)) = resp.chunk().await {
                let resp_text = String::from_utf8_lossy(&chunk);
                let Some(resp_text) = resp_text.strip_prefix("data: ") else {
                    continue;
                };
                if resp_text == "[DONE]" {
                    break;
                }
                let Ok(chunk) = serde_json::from_str::<StreamChunk>(resp_text) else {
                    continue;
                };
                let Some(choice) = chunk.choices.first() else {
                    continue;
                };
                resp_body.push_str(&choice.delta.as_ref().unwrap().content);
            }

            (
                axum::http::StatusCode::OK,
                serde_json::to_string(&ChatResponse {
                    id: None,
                    choices: vec![Choice {
                        delta: None,
                        index: Some(0),
                        message: Some(Delta {
                            role: Some("assistant".to_owned()),
                            content: resp_body,
                            match_stop: None,
                            finish_reason: Some("stop".to_owned()),
                        }),
                    }],
                    model: None,
                    object: None,
                    created: None,
                })
                .unwrap(),
            )
                .into_response()
        }
    }
}
