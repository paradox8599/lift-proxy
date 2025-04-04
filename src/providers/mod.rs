pub mod auth;

mod chutes_api;
mod deepinfra;
mod dzmm;
mod google;
mod nvidia;
mod openrouter;

use crate::{
    app_state::AppState,
    db::auth::{db_reset_auth, ProviderAuth},
};
use auth::ProviderAuthVec;
use axum::{body::Bytes, http::HeaderMap};
use chrono::{DateTime, Utc};
use chutes_api::ChutesAPIProvider;
use deepinfra::DeepinfraProvider;
use dzmm::DzmmProvider;
use google::GoogleProvider;
use nvidia::NvidiaProvider;
use openrouter::OpenRouterProvider;
use reqwest::{Body, Url};
use std::sync::{Arc, Mutex};

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
        let mut auth_vec = auth.write().unwrap();

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
        let picked_auth = self.pick_auth();
        if let Some(auth) = &picked_auth {
            let auth = auth.lock().unwrap();
            let value = format!("Bearer {}", auth.api_key);
            headers.insert("authorization", value.parse().unwrap());
            tracing::info!(
                "[Auth] {}: {} - {}/{} # {}",
                auth.provider,
                auth.id,
                auth.sent + 1,
                auth.max,
                auth.comments.clone().unwrap_or("".to_owned())
            );
        }
        picked_auth
    }

    pub fn handle_auth_reset(
        app: Arc<AppState>,
        auth_vec: ProviderAuthVec,
        provider: AuthProviderName,
        last_authed_at: DateTime<Utc>,
        reset_time: chrono::NaiveTime,
    ) {
        let provider = provider.to_string();
        let now = Utc::now();
        let current_time = now.time();

        // If current_time is before reset_time,
        // do not reset yet.
        if current_time < reset_time {
            tracing::debug!("Auth reset for {} skipped: not reset yet", provider);
            return;
        }

        // If the last auth was on the same day and after the reset time,
        // reset already performed, skip
        if last_authed_at.date_naive() == now.date_naive() && last_authed_at.time() > reset_time {
            tracing::debug!("Auth reset for {} skipped: already done today", provider);
            return;
        }

        tokio::spawn(async move {
            {
                let auths = auth_vec.read().unwrap();
                for auth_mutex in auths.iter() {
                    let mut auth = auth_mutex.lock().unwrap();
                    auth.sent = 0;
                }
            }

            match db_reset_auth(&app, &provider).await {
                Ok(rows) => tracing::info!("Auth reset for {}, {} rows updated", provider, rows),
                Err(e) => tracing::error!("Error resetting auth: {}", e),
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
                Arc::new(Provider::$name($provider::new(app.clone()))),
            );)*
        }

        // define providers
        pub enum Provider {
            $($name($provider),)*
        }

        pub enum AuthProviderName {
            $($name),*
        }

        impl AuthProviderName {
            pub fn to_string(&self) -> String {
                match self {
                    $(Self::$name => stringify!($name).to_lowercase(),)*
                }
            }
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
    ChutesAPI => ChutesAPIProvider,
    Deepinfra => DeepinfraProvider,
    Dzmm => DzmmProvider,
    Google => GoogleProvider,
    Nvidia => NvidiaProvider,
    OpenRouter => OpenRouterProvider
);
