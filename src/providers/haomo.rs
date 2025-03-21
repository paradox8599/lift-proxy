use super::{ProviderAuthVec, ProviderFn};
use axum::{body::Bytes, http::HeaderMap};
use reqwest::{self as r, Url};

const HAOMO_MODELS_URL: &str = "https://chat.haomo.de/api/models";
const HAOMO_CHAT_URL: &str = "https://chat.haomo.de/api/chat/completions";

#[derive(Clone, Debug, Default)]
pub struct HaomoProvider {
    pub auth_vec: ProviderAuthVec,
}

impl ProviderFn for HaomoProvider {
    fn models_url(&self) -> Url {
        Url::parse(HAOMO_MODELS_URL).unwrap()
    }

    fn chat_url(&self) -> Url {
        Url::parse(HAOMO_CHAT_URL).unwrap()
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
