use super::ProviderFn;
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{Body, Url};

pub struct DzmmProvider;

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
    }

    fn body_modifier(&self, body: Bytes) -> Body {
        Body::from(body)
    }
}
