use axum::http::HeaderMap;
use chutes::ChutesProvider;
use deepinfra::DeepinfraProvider;
use reqwest::Url;

pub mod chutes;
pub mod deepinfra;

pub trait ProviderFn {
    fn models_url(&self) -> Url;
    fn chat_url(&self) -> Url;
    fn get_header_modifier(&self, headers: &mut HeaderMap);
    fn post_header_modifier(&self, headers: &mut HeaderMap);
    fn body_modifier(&self, body: &str) -> String;
}

pub enum Provider {
    Chutes(ChutesProvider),
    Deepinfra(DeepinfraProvider),
}

impl ProviderFn for Provider {
    fn models_url(&self) -> Url {
        match self {
            Provider::Chutes(p) => p.models_url(),
            Provider::Deepinfra(p) => p.models_url(),
        }
    }

    fn chat_url(&self) -> Url {
        match self {
            Provider::Chutes(p) => p.chat_url(),
            Provider::Deepinfra(p) => p.chat_url(),
        }
    }

    fn get_header_modifier(&self, headers: &mut HeaderMap) {
        match self {
            Provider::Chutes(p) => p.get_header_modifier(headers),
            Provider::Deepinfra(p) => p.get_header_modifier(headers),
        }
    }

    fn post_header_modifier(&self, headers: &mut HeaderMap) {
        match self {
            Provider::Chutes(p) => p.post_header_modifier(headers),
            Provider::Deepinfra(p) => p.post_header_modifier(headers),
        }
    }

    fn body_modifier(&self, body: &str) -> String {
        match self {
            Provider::Chutes(p) => p.body_modifier(body),
            Provider::Deepinfra(p) => p.body_modifier(body),
        }
    }
}

pub fn get_provider(name: &str) -> Option<Provider> {
    match name {
        "chutes" => Some(Provider::Chutes(ChutesProvider {})),
        "deepinfra" => Some(Provider::Deepinfra(DeepinfraProvider {})),
        _ => None,
    }
}
