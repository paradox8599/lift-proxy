pub mod auth;

mod deepinfra;
mod dzmm;
mod google;
mod nvidia;
mod openrouter;

use crate::{app_state::AppState, db::auth::ProviderAuth};
use auth::ProviderAuthVec;
use axum::{body::Bytes, http::HeaderMap};
use chrono::{NaiveTime, Utc};
use deepinfra::DeepinfraProvider;
use dzmm::DzmmProvider;
use google::GoogleProvider;
use nvidia::NvidiaProvider;
use openrouter::OpenRouterProvider;
use reqwest::{Body, Url};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

async fn wait_until(target_time: NaiveTime) {
    let now = Utc::now().time();
    let mut wait_duration = target_time - now;

    if wait_duration.num_seconds() < 0 {
        wait_duration += chrono::Duration::days(1);
    }

    let wait_seconds = wait_duration.num_seconds() as u64;
    tracing::debug!("[wait_until] Waiting for {} seconds", wait_seconds);
    sleep(Duration::from_secs(wait_seconds)).await;
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
            // check if auth is valid, if it has available quota
            // if max is 0, then it is unlimited
            auth.valid && (auth.max == 0 || auth.sent < auth.max)
        });

        match index {
            Some(index) => auth_vec.get(index).cloned(),
            _ => None,
        }
    }

    pub fn apply_auth(&self, headers: &mut HeaderMap) -> Option<Arc<Mutex<ProviderAuth>>> {
        let header = headers.get(axum::http::header::AUTHORIZATION);
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

    pub fn scheduled_auth_reset(
        auth_vec: ProviderAuthVec,
        name: &str,
        reset_time: Option<chrono::NaiveTime>,
    ) {
        const RESET_TIME: chrono::NaiveTime = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let reset_time = reset_time.unwrap_or(RESET_TIME);
        let name = name.to_owned();

        // TODO: check auth reset on every requrest

        tokio::spawn(async move {
            loop {
                tracing::info!(
                    "Scheduled next auth reset for {} at {} UTC, now: {} UTC",
                    name,
                    reset_time,
                    chrono::Utc::now().time()
                );
                crate::providers::wait_until(reset_time).await;

                {
                    let auths = auth_vec.lock().unwrap();
                    for auth_mutex in auths.iter() {
                        let mut auth = auth_mutex.lock().unwrap();
                        auth.sent = 0;
                    }
                }

                tracing::info!("Auth reset for {} done", name);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
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
    Nvidia => NvidiaProvider,
    OpenRouter => OpenRouterProvider,
    Google => GoogleProvider
);
