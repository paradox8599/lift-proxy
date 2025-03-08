use super::{auth::ProviderAuthVec, wait_until, ProviderFn};
use axum::{body::Bytes, http::HeaderMap};
use chrono::NaiveTime;
use reqwest::{Body, Url};
use std::sync::{Arc, Mutex};

const DZMM_MODELS_URL: &str = "https://www.gpt4novel.com/api/xiaoshuoai/ext/v1/models";
const DZMM_CHAT_URL: &str = "https://www.gpt4novel.com/api/xiaoshuoai/ext/v1/chat/completions";

// TODO: handle response body when stream == false

// DZMM Resets free quota at 11:00AM UTC
const RESET_TIME: NaiveTime = NaiveTime::from_hms_opt(11, 0, 0).unwrap();

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
                tracing::info!("Scheduled next auth reset for DZMM at {}", RESET_TIME);
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
}
