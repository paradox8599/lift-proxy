use crate::{app_state::AppState, providers::ProviderFn as _};
use chrono::{DateTime, Utc};
use eyre::Result;
use reqwest::StatusCode;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::Instant;

const COOLDOWN_SECONDS: u64 = 30 * 60;
const SYNC_INTERVAL_SECONDS: u64 = 5 * 60;
const FORCE_SYNC_INTERVAL_SECONS: u64 = 8 * 60 * 60;

#[allow(dead_code)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProviderAuth {
    pub id: i32,
    pub provider: String,
    pub api_key: String,
    pub sent: i32,
    pub max: i32,
    pub valid: bool,
    pub used_at: DateTime<Utc>,
    pub cooldown: bool,
    pub comments: Option<String>,
}

pub type ProviderAuthVec = Arc<Mutex<Vec<Arc<Mutex<ProviderAuth>>>>>;

async fn get_all_auth(app: &Arc<AppState>) -> Vec<Arc<Mutex<ProviderAuth>>> {
    let providers = app.providers.lock().await;
    let providers = providers.values().collect::<Vec<_>>();
    providers
        .iter()
        .flat_map(|provider| {
            provider
                .get_auth()
                .lock()
                .unwrap()
                .iter()
                .cloned()
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
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
            let mut provider_auth = provider_auth.lock().unwrap();
            provider_auth.push(Arc::new(Mutex::new(auth)));
        } else {
            tracing::warn!("Mismatched auth provider: {:?}", auth);
        }
    }
}

async fn db_update_auth(app: &Arc<AppState>, provider_auths: &Vec<ProviderAuth>) -> Result<u64> {
    let query = r#"
        UPDATE auth
        SET sent = u.sent,
            valid = u.valid,
            used_at = u.used_at,
            cooldown = u.cooldown
        FROM UNNEST($1::int[], $2::int[], $3::bool[], $4::timestamptz[], $5::bool[])
        AS u(id, sent, valid, used_at, cooldown)
        WHERE auth.id = u.id
    "#;

    let mut ids = Vec::with_capacity(provider_auths.len());
    let mut sents = Vec::with_capacity(provider_auths.len());
    let mut valids = Vec::with_capacity(provider_auths.len());
    let mut used_ats = Vec::with_capacity(provider_auths.len());
    let mut cooldowns = Vec::with_capacity(provider_auths.len());
    for pa in provider_auths {
        ids.push(pa.id);
        sents.push(pa.sent);
        valids.push(pa.valid);
        used_ats.push(pa.used_at);
        cooldowns.push(pa.cooldown);
    }

    let result = sqlx::query(query)
        .bind(&ids)
        .bind(&sents)
        .bind(&valids)
        .bind(&used_ats)
        .bind(&cooldowns)
        .execute(&app.pool)
        .await?;

    Ok(result.rows_affected())
}

pub async fn sync_auth(app: &Arc<AppState>) -> Result<()> {
    let providers = app.providers.lock().await;
    let providers_vec = providers.values().cloned().collect::<Vec<_>>();
    let provider_auths = providers_vec
        .iter()
        .flat_map(|provider| {
            let auth = provider.get_auth();
            let auth = auth.lock().unwrap();
            auth.iter()
                .map(|auth| auth.lock().unwrap().clone())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let result = db_update_auth(app, &provider_auths).await?;

    tracing::info!("Updated {} rows", result);

    let db_auth: Vec<ProviderAuth> = sqlx::query_as("SELECT * FROM auth")
        .fetch_all(&app.pool)
        .await?;

    // find new auth in db_auth that does not exist in provider_auths
    let new_auth = db_auth
        .iter()
        .filter(|auth| !provider_auths.iter().any(|pa| pa.id == auth.id))
        .cloned()
        .collect::<Vec<_>>();

    tracing::info!("Pulled {} new auths", new_auth.len());

    // add new auth to providers
    for auth in new_auth {
        if let Some(provider) = providers.get(&auth.provider) {
            let provider_auth = provider.get_auth();
            let mut provider_auth = provider_auth.lock().unwrap();
            provider_auth.push(Arc::new(Mutex::new(auth)));
        } else {
            tracing::warn!("Mismatched auth provider: {:?}", auth);
        }
    }

    *app.auth_last_synced_at.lock().await = Instant::now();

    Ok(())
}

pub fn update_auth_state_on_response(auth: &Option<Arc<Mutex<ProviderAuth>>>, status: &StatusCode) {
    if let Some(auth_mutex) = auth {
        let auth_mutex = auth_mutex.clone();
        let auth_mutex_schedule = auth_mutex.clone();
        let mut auth = auth_mutex.lock().unwrap();
        auth.used_at = chrono::Utc::now();
        match *status {
            StatusCode::OK => {
                auth.sent += 1;
                tracing::info!("Successfully sent a request to {}", auth.provider);
            }
            // 401
            StatusCode::UNAUTHORIZED => {
                auth.valid = false;
                tracing::warn!("Unauthorized request to {}", auth.provider);
            }
            // 429
            StatusCode::TOO_MANY_REQUESTS => {
                tracing::warn!("One of {}'s auth key is rate limited", auth.provider);
                auth.cooldown = true;
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(COOLDOWN_SECONDS)).await;
                    let mut auth = auth_mutex_schedule.lock().unwrap();
                    auth.cooldown = false;
                    tracing::info!(
                        "Auth key {} of {} is no longer rate limited",
                        auth.id,
                        auth.provider
                    );
                });
            }
            x => tracing::warn!("Unsuccessful StatusCode: {}", x),
        };
    }
}

pub fn regular_auth_state_update(app: &Arc<AppState>) {
    let app = app.clone();
    tokio::spawn(async move {
        loop {
            {
                let mut all_auth = get_all_auth(&app).await;

                // check app.auth_last_synced_at and determine if needs to do db update
                let last_synced_at = {
                    let last_synced_at = app.auth_last_synced_at.lock().await;
                    last_synced_at.elapsed().as_secs()
                };
                if last_synced_at > SYNC_INTERVAL_SECONDS {
                    // check last request time
                    // if last request was more than SYNC_INTERVAL_MINUTES, skip
                    all_auth
                        .sort_by(|a, b| a.lock().unwrap().used_at.cmp(&b.lock().unwrap().used_at));
                    if let Some(auth) = all_auth.last() {
                        let auth = auth.lock().unwrap().clone();
                        let now = Utc::now();
                        let delta = now - auth.used_at;
                        if (delta.num_seconds() as u64) < SYNC_INTERVAL_SECONDS
                            || last_synced_at > FORCE_SYNC_INTERVAL_SECONS
                        {
                            tracing::info!("Start syncing database auth");
                            // call sync_auth
                            if let Err(e) = sync_auth(&app).await {
                                tracing::error!("Regular auth syncing failed: {}", e);
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(SYNC_INTERVAL_SECONDS)).await;
        }
    });
}
