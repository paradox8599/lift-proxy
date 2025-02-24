use super::ProviderFn;
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{Body, Url};

pub struct ChutesProvider;

impl ProviderFn for ChutesProvider {
    fn models_url(&self) -> Url {
        Url::parse("https://llm.chutes.ai/v1/models").unwrap()
    }

    fn chat_url(&self) -> Url {
        Url::parse("https://chutes.ai/app/api/chat").unwrap()
    }

    fn get_header_modifier(&self, headers: &mut HeaderMap) {
        headers.clear();
    }

    fn post_header_modifier(&self, headers: &mut HeaderMap) {
        headers.insert("origin", "https://chutes.ai".parse().unwrap());
        headers.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36".parse().unwrap());
        headers.insert("content-type", "application/json".parse().unwrap());
    }

    fn body_modifier(&self, _body: Bytes) -> Body {
        todo!()
    }
}
