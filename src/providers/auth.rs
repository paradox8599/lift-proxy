use crate::{
    app_state::AppState,
    db::auth::{db_get_all_auth, db_update_auth, ProviderAuth},
    providers::ProviderFn as _,
};
use eyre::Result;
use reqwest::StatusCode;
use std::{
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

const COOLDOWN_SECONDS: u64 = 30 * 60;

// Keep ProviderAuthVec here as it relates to the provider's in-memory state
pub type ProviderAuthVec = Arc<RwLock<Vec<Arc<Mutex<ProviderAuth>>>>>;

/// Initializes the in-memory auth state by fetching from the database.
pub async fn init_auth(app: &Arc<AppState>) {
    // Fetch auth data using the db module function
    match db_get_all_auth(app).await {
        Ok(all_auth) => {
            tracing::info!("[Auth] {} auths initialized", all_auth.len());
            let providers = app.providers.lock().await;
            for auth in all_auth {
                if let Some(provider) = providers.get(&auth.provider) {
                    let provider_auth_vec = provider.get_auth();
                    let mut provider_auth_vec_locked = provider_auth_vec.write().unwrap();
                    provider_auth_vec_locked.push(Arc::new(Mutex::new(auth)));
                } else {
                    tracing::warn!("Mismatched auth provider found during init: {:?}", auth);
                }
            }
        }
        Err(e) => {
            // Consider how to handle DB errors during startup. Panic might be okay
            // depending on requirements, or perhaps retry logic.
            panic!("Failed to initialize auth from database: {}", e);
        }
    }
}

/// Syncs the in-memory auth state with the database.
pub async fn sync_auth(app: &Arc<AppState>) -> Result<()> {
    let providers = app.providers.lock().await;
    let providers_vec = providers.values().cloned().collect::<Vec<_>>();

    // Collect current state from memory
    let provider_auths_in_memory = providers_vec
        .iter()
        .flat_map(|provider| {
            let auth_vec = provider.get_auth();
            let auth_vec_locked = auth_vec.read().unwrap();
            auth_vec_locked
                .iter()
                .map(|auth_mutex| auth_mutex.lock().unwrap().clone())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // Update the database with the current in-memory state
    let updated_rows = db_update_auth(app, &provider_auths_in_memory).await?;
    tracing::info!("Synced state to DB, updated {} rows", updated_rows);

    // Fetch the latest state from the database again
    let db_auth = db_get_all_auth(app).await?;

    // Find new auth records in the database that are not yet in memory
    let new_auth = db_auth
        .iter()
        .filter(|auth| !provider_auths_in_memory.iter().any(|pa| pa.id == auth.id))
        .cloned()
        .collect::<Vec<_>>();

    if !new_auth.is_empty() {
        tracing::info!("Pulled {} new auths from database", new_auth.len());
        // Add new auth records to the appropriate providers in memory
        for auth in new_auth {
            if let Some(provider) = providers.get(&auth.provider) {
                let provider_auth_vec = provider.get_auth();
                let mut provider_auth_vec_locked = provider_auth_vec.write().unwrap();
                // Avoid adding duplicates if sync runs concurrently somehow (though unlikely with locks)
                if !provider_auth_vec_locked
                    .iter()
                    .any(|pa| pa.lock().unwrap().id == auth.id)
                {
                    provider_auth_vec_locked.push(Arc::new(Mutex::new(auth)));
                }
            } else {
                tracing::warn!("Mismatched auth provider found during sync: {:?}", auth);
            }
        }
    }

    Ok(())
}

/// Updates the state of a specific auth key based on the HTTP response status.
pub fn update_auth_state_on_response(
    app: &Arc<AppState>,
    auth: &Option<Arc<Mutex<ProviderAuth>>>,
    status: &StatusCode,
) {
    if let Some(auth_mutex) = auth {
        let auth_mutex_clone = auth_mutex.clone(); // Clone for potential async task
        let mut auth_locked = auth_mutex.lock().unwrap();
        auth_locked.used_at = chrono::Utc::now(); // Update usage time regardless of status

        match *status {
            StatusCode::OK => {
                auth_locked.sent += 1;
                // Optional: info!() Log success if needed, but may be verbose
                tracing::debug!("[{}] key {} authed", auth_locked.provider, auth_locked.id,);
            }
            StatusCode::UNAUTHORIZED => {
                auth_locked.valid = false;
                tracing::warn!(
                    "Auth key {} for {} marked as invalid due to UNAUTHORIZED",
                    auth_locked.id,
                    auth_locked.provider
                );
            }
            StatusCode::TOO_MANY_REQUESTS => {
                tracing::warn!(
                    "Auth key {} for {} is rate limited (TOO_MANY_REQUESTS)",
                    auth_locked.id,
                    auth_locked.provider
                );
                auth_locked.cooldown = true;
                // Spawn a task to remove the cooldown flag after the duration
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(COOLDOWN_SECONDS)).await;
                    let mut auth_schedule_locked = auth_mutex_clone.lock().unwrap();
                    auth_schedule_locked.cooldown = false;
                    tracing::info!(
                        "Auth key {} for {} cooldown finished",
                        auth_schedule_locked.id,
                        auth_schedule_locked.provider
                    );
                });
            }
            // Handle other potentially relevant error codes if necessary
            // e.g., 403 Forbidden might also indicate an invalid key in some APIs
            _ => tracing::warn!(
                "Received unhandled status code {} for auth key {} on provider {}",
                status,
                auth_locked.id,
                auth_locked.provider
            ),
        };

        // Create a clone of the auth to update the DB
        let auth_for_db = auth_locked.clone();

        // Update the auth in the database in background
        let app = app.clone();
        tokio::spawn(async move {
            if let Err(e) = db_update_auth(&app, &vec![auth_for_db]).await {
                tracing::error!("Failed to update auth in database: {}", e);
            }
        });
    } else {
        // This case might happen if the original request already had an Authorization header
        // or if no suitable key was found (e.g., all keys on cooldown or invalid).
        tracing::debug!(
            "Attempted to update auth state, but no specific key was selected for the request."
        );
    }
}
