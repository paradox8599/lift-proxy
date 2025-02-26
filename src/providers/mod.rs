use axum::{body::Bytes, http::HeaderMap};
use deepinfra::DeepinfraProvider;
use nvidia::NvidiaProvider;
use reqwest::{Body, Url};

pub mod deepinfra;
pub mod nvidia;

pub trait ProviderFn {
    fn models_url(&self) -> Url;
    fn chat_url(&self) -> Url;
    fn get_header_modifier(&self, headers: &mut HeaderMap);
    fn post_header_modifier(&self, headers: &mut HeaderMap);
    fn body_modifier(&self, body: Bytes) -> Body;
}

pub enum Provider {
    Deepinfra(DeepinfraProvider),
    Nvidia(NvidiaProvider),
}

impl ProviderFn for Provider {
    fn models_url(&self) -> Url {
        match self {
            Provider::Deepinfra(p) => p.models_url(),
            Provider::Nvidia(p) => p.models_url(),
        }
    }

    fn chat_url(&self) -> Url {
        match self {
            Provider::Deepinfra(p) => p.chat_url(),
            Provider::Nvidia(p) => p.chat_url(),
        }
    }

    fn get_header_modifier(&self, headers: &mut HeaderMap) {
        match self {
            Provider::Deepinfra(p) => p.get_header_modifier(headers),
            Provider::Nvidia(p) => p.get_header_modifier(headers),
        }
    }

    fn post_header_modifier(&self, headers: &mut HeaderMap) {
        match self {
            Provider::Deepinfra(p) => p.post_header_modifier(headers),
            Provider::Nvidia(p) => p.post_header_modifier(headers),
        }
    }

    fn body_modifier(&self, body: Bytes) -> Body {
        match self {
            Provider::Deepinfra(p) => p.body_modifier(body),
            Provider::Nvidia(p) => p.body_modifier(body),
        }
    }
}

pub fn get_provider(name: &str) -> Option<Provider> {
    match name {
        "deepinfra" => Some(Provider::Deepinfra(DeepinfraProvider {})),
        "nvidia" => Some(Provider::Nvidia(NvidiaProvider {})),
        _ => None,
    }
}
