use std::sync::{Arc, Mutex};

use super::{ProviderAuthVec, ProviderFn};
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{Body, Url};

const DEEPINFRA_MODELS_URL: &str = "https://api.deepinfra.com/v1/openai/models";
const DEEPINFRA_CHAT_URL: &str = "https://api.deepinfra.com/v1/openai/chat/completions";

#[derive(Clone, Debug, Default)]
pub struct DeepinfraProvider;

impl ProviderFn for DeepinfraProvider {
    fn models_url(&self) -> Url {
        Url::parse(DEEPINFRA_MODELS_URL).expect("DeepInfra models url")
    }

    fn chat_url(&self) -> Url {
        Url::parse(DEEPINFRA_CHAT_URL).expect("DeepInfra chat url")
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
        Arc::new(Mutex::new(vec![]))
    }
}
