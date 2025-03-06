mod deepinfra;
mod dzmm;
mod nvidia;

use crate::app_state::AppState;
use axum::{body::Bytes, http::HeaderMap};
use chrono::DateTime;
use chrono::Utc;
use deepinfra::DeepinfraProvider;
use dzmm::DzmmProvider;
use nvidia::NvidiaProvider;
use reqwest::{Body, Url};
use std::sync::Arc;
use std::sync::Mutex;

#[allow(dead_code)]
#[derive(Debug, sqlx::FromRow)]
pub struct ProviderAuth {
    pub id: i32,
    pub provider: String,
    pub api_key: String,
    pub sent: i32,
    pub max: i32,
    pub valid: bool,
    pub used_at: DateTime<Utc>,
    pub cooldown: bool,
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
            a.used_at.cmp(&b.used_at)
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
    let all_auth: Vec<ProviderAuth> = sqlx::query_as("SELECT * FROM auth")
        .fetch_all(&app.pool)
        .await
        .unwrap();

    let providers = app.providers.lock().await;

    for auth in all_auth {
        if let Some(provider) = providers.get(&auth.provider) {
            let provider_auth = provider.get_auth();
            let mut provider_auth = provider_auth.lock().expect("");
            provider_auth.push(Arc::new(Mutex::new(auth)));
        } else {
            tracing::warn!("Mismatched auth provider:{:?}", auth);
        }
    }

    // macro_rules! insert_auth {
    //     ($($p:expr, $k:expr),* $(,)?) => {{
    //         let values = vec![
    //             $(format!("('{}','{}')", $p, $k),)*
    //         ];
    //         format!("INSERT INTO auth(provider, api_key) VALUES {}", values.join(", "))
    //     }};
    // }
    //
    // if r.is_empty() {
    //     let r = sqlx::query(&insert_auth!(
    //         "dzmm",
    //         "",
    //         "nvidia",
    //         ""
    //     ))
    //     .execute(&app.pool)
    //     .await
    //     .unwrap();
    //     tracing::debug!("{:?}", r);
    // }
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
