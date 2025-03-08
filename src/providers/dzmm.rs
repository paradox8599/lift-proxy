use super::{auth::ProviderAuthVec, wait_until, ProviderFn};
use crate::utils::data_types::{ChatBody, StreamChunk};
use axum::{body::Bytes, http::HeaderMap, response::IntoResponse as _};
use chrono::Utc;
use reqwest::{Body, Url};
use std::sync::{Arc, Mutex};

const DZMM_MODELS_URL: &str = "https://www.gpt4novel.com/api/xiaoshuoai/ext/v1/models";
const DZMM_CHAT_URL: &str = "https://www.gpt4novel.com/api/xiaoshuoai/ext/v1/chat/completions";

// DZMM Resets free quota at 11:00AM UTC
const RESET_TIME: chrono::NaiveTime = chrono::NaiveTime::from_hms_opt(11, 0, 0).unwrap();


#[derive(Clone, Debug)]
pub struct DzmmProvider {
    pub auth_vec: ProviderAuthVec,
}

impl Default for DzmmProvider {
    fn default() -> Self {
        let auth_vec: ProviderAuthVec = Arc::new(Mutex::new(vec![]));
        let auth_vec_clone = auth_vec.clone();
        tokio::spawn(async move {
            loop {
                tracing::info!(
                    "Scheduled next auth reset for DZMM at {} UTC, now: {} UTC",
                    RESET_TIME,
                    Utc::now().time()
                );
                wait_until(RESET_TIME).await;

                let auths = auth_vec_clone.lock().unwrap();
                for auth_mutex in auths.iter() {
                    let mut auth = auth_mutex.lock().unwrap();
                    auth.sent = 0;
                }
                tracing::info!("Auth reset for DZMM done");
            }
        });
        Self { auth_vec }
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
            crate::utils::get_response_stream(resp).await
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
                resp_body.push_str(&choice.delta.content);
            }
            (axum::http::StatusCode::OK, resp_body).into_response()
        }
    }
}
