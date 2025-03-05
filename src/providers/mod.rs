use axum::{body::Bytes, http::HeaderMap};
use deepinfra::DeepinfraProvider;
use dzmm::DzmmProvider;
use nvidia::NvidiaProvider;
use reqwest::{Body, Url};

pub mod deepinfra;
pub mod dzmm;
pub mod nvidia;

pub trait ProviderFn {
    fn models_url(&self) -> Url;
    fn chat_url(&self) -> Url;
    fn get_header_modifier(&self, headers: &mut HeaderMap);
    fn post_header_modifier(&self, headers: &mut HeaderMap);
    fn body_modifier(&self, body: Bytes) -> Body;
}

macro_rules! impl_provider {
    ($($name:ident => $provider:ident),*) => {
        pub fn get_provider(name: &str) -> Option<Provider> {
            $(
                if name == stringify!($name).to_lowercase() {
                    return Some(Provider::$name($provider {}));
                }
            )*
            None
        }

        pub enum Provider {
        $($name($provider),)*
        }

        impl ProviderFn for Provider {
            fn models_url(&self) -> Url {
                match self {
                    $(Provider::$name(p) => p.models_url(),)*
                }
            }

            fn chat_url(&self) -> Url {
                match self {
                    $(Provider::$name(p) => p.chat_url(),)*
                }
            }

            fn get_header_modifier(&self, headers: &mut HeaderMap) {
                match self {
                    $(Provider::$name(p) => p.get_header_modifier(headers),)*
                }
            }

            fn post_header_modifier(&self, headers: &mut HeaderMap) {
                match self {
                    $(Provider::$name(p) => p.post_header_modifier(headers),)*
                }
            }

            fn body_modifier(&self, body: Bytes) -> Body {
                match self {
                    $(Provider::$name(p) => p.body_modifier(body),)*
                }
            }
        }
    };

}

impl_provider!(
    Deepinfra => DeepinfraProvider,
    Nvidia => NvidiaProvider,
    Dzmm => DzmmProvider
);
