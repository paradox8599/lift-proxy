pub mod auth;

mod deepinfra;
mod dzmm;
mod nvidia;

use crate::app_state::AppState;
use auth::ProviderAuth;
use auth::ProviderAuthVec;
use axum::{body::Bytes, http::HeaderMap};
use deepinfra::DeepinfraProvider;
use dzmm::DzmmProvider;
use nvidia::NvidiaProvider;
use reqwest::{Body, Url};
use std::sync::Arc;
use std::sync::Mutex;

use chrono::{Duration as ChronoDuration, Local, NaiveTime};
use tokio::time::{sleep, Duration};

async fn wait_until(target_time: NaiveTime) {
    let now = Local::now();
    let today_target = now
        .date_naive()
        .and_time(target_time)
        .and_local_timezone(now.timezone())
        .unwrap();

    // Determine whether the target time is today or tomorrow
    let next_target = if today_target > now {
        today_target
    } else {
        (now + ChronoDuration::days(1))
            .date_naive()
            .and_time(target_time)
            .and_local_timezone(now.timezone())
            .unwrap()
    };

    // Calculate the duration to wait
    let duration = (next_target - now)
        .to_std()
        .unwrap_or(Duration::from_secs(0));

    sleep(duration).await;
}

pub trait ProviderFn {
    fn models_url(&self) -> Url;
    fn chat_url(&self) -> Url;
    fn get_header_modifier(&self, headers: &mut HeaderMap);
    fn post_header_modifier(&self, headers: &mut HeaderMap);
    fn body_modifier(&self, body: Bytes) -> Body;
    fn get_auth(&self) -> ProviderAuthVec;
    async fn get_response(
        &self,
        body: axum::body::Bytes,
        resp: reqwest::Response,
    ) -> axum::http::Response<axum::body::Body>;
}

impl Provider {
    pub fn pick_auth(&self) -> Option<Arc<Mutex<ProviderAuth>>> {
        let auth = self.get_auth();
        let mut auth_vec = auth.lock().unwrap();

        // sort by last_used
        auth_vec.sort_by(|a, b| {
            let a = a.lock().unwrap();
            let b = b.lock().unwrap();
            a.used_at.cmp(&b.used_at)
        });

        // find a valid auth
        let index = auth_vec.iter().position(|auth| {
            let auth = auth.lock().unwrap();
            auth.valid && auth.sent < auth.max
        });

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
            let auth = auth.lock().unwrap();
            let value = format!("Bearer {}", auth.api_key);
            headers.insert("authorization", value.parse().unwrap());
            tracing::info!(
                "[Auth] {}: {} - {}/{} # {}",
                auth.provider,
                auth.id,
                auth.sent,
                auth.max,
                auth.comments.clone().unwrap_or("".to_owned())
            );
        }
        picked_auth
    }
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


            async fn get_response(
                &self,
                body: axum::body::Bytes,
                resp: reqwest::Response,
            ) -> axum::http::Response<axum::body::Body>
            {
                match self {
                    $(Provider::$name(p) => p.get_response(body, resp).await,)*
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
