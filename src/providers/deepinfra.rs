use super::ProviderFn;
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{Body, Url};

pub struct DeepinfraProvider;

impl ProviderFn for DeepinfraProvider {
    fn models_url(&self) -> Url {
        Url::parse("https://api.deepinfra.com/v1/openai/models").expect("DeepInfra models url")
    }

    fn chat_url(&self) -> Url {
        Url::parse("https://api.deepinfra.com/v1/openai/chat/completions")
            .expect("DeepInfra chat url")
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
}
