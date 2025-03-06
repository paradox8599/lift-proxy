mod deepinfra;
mod dzmm;
mod nvidia;

use crate::app_state::AppState;
use axum::{body::Bytes, http::HeaderMap};
use deepinfra::DeepinfraProvider;
use dzmm::DzmmProvider;
use nvidia::NvidiaProvider;
use reqwest::{Body, Url};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::time::Instant;

#[derive(Debug)]
pub struct ProviderAuth {
    pub api_key: String,
    pub last_used: Instant,
    pub count: u32,
    pub limit: u32,
    pub valid: bool,
    pub cooldown: bool,
}

impl ProviderAuth {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            last_used: Instant::now(),
            count: 0,
            limit: 5000,
            valid: true,
            cooldown: false,
        }
    }
}

type ProviderAuthVec = Arc<Mutex<Vec<Arc<Mutex<ProviderAuth>>>>>;

pub trait ProviderFn {
    fn models_url(&self) -> Url;
    fn chat_url(&self) -> Url;
    fn get_header_modifier(&self, headers: &mut HeaderMap);
    fn post_header_modifier(&self, headers: &mut HeaderMap);
    fn body_modifier(&self, body: Bytes) -> Body;
    fn get_auth(&self) -> ProviderAuthVec;
}

impl Provider {
    pub fn pick_auth(&self) -> Option<Arc<Mutex<ProviderAuth>>> {
        let auth = self.get_auth();
        let mut auth_vec = auth.lock().expect("");

        // sort by last_used
        auth_vec.sort_by(|a, b| {
            let a = a.lock().expect("");
            let b = b.lock().expect("");
            b.last_used.elapsed().cmp(&a.last_used.elapsed())
        });

        // find a valid auth
        let index = auth_vec
            .iter()
            .position(|auth| auth.lock().expect("").valid);

        match index {
            Some(index) => auth_vec.get(index).cloned(),
            _ => None,
        }
    }

    pub fn apply_auth(&self, headers: &mut HeaderMap) -> Option<Arc<Mutex<ProviderAuth>>> {
        let header = headers.get("authorization");
        if header.is_some() {
            return None;
        }

        let picked_auth = self.pick_auth();
        if let Some(auth) = &picked_auth {
            let auth = auth.lock().expect("");
            let value = format!("Bearer {}", auth.api_key);
            headers.insert("authorization", value.parse().expect(""));
            tracing::debug!("{:?}", auth);
        }
        picked_auth
    }
}

pub async fn init_auth(app: &Arc<AppState>) {
    // TODO: pull auth from db
    todo!()
}

macro_rules! impl_provider {
    ($($name:ident => $provider:ident),*) => {
        // initialize providers
        pub async fn init_providers(app: &Arc<AppState>) {
            let mut providers = app.providers.lock().await;
            $(providers.insert(
                stringify!($name).to_lowercase(),
                Arc::new(Provider::$name($provider::default())),
            );)*
        }

        // define providers
        #[derive(Debug)]
        pub enum Provider {
            $($name($provider),)*
        }

        // wrap provider functions
        impl ProviderFn for Provider {
            fn get_auth(&self) -> ProviderAuthVec {
                match self {
                    $(Provider::$name(p) => p.get_auth(),)*
                }
            }

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
    Dzmm => DzmmProvider,
    Nvidia => NvidiaProvider
);
