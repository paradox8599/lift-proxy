use crate::{
    app_state::AppState,
    db::auth::{db_get_all_auth, db_update_auth, ProviderAuth}, // Import from db module
    providers::ProviderFn as _,
};
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

// Keep ProviderAuthVec here as it relates to the provider's in-memory state
pub type ProviderAuthVec = Arc<Mutex<Vec<Arc<Mutex<ProviderAuth>>>>>;

/// Gets all auth objects currently held in memory within the AppState.
async fn get_all_auth_from_memory(app: &Arc<AppState>) -> Vec<Arc<Mutex<ProviderAuth>>> {
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

/// Initializes the in-memory auth state by fetching from the database.
pub async fn init_auth(app: &Arc<AppState>) {
    // Fetch auth data using the db module function
    match db_get_all_auth(app).await {
        Ok(all_auth) => {
            tracing::info!("Initialized with {} auths from database", all_auth.len());
            let providers = app.providers.lock().await;
            for auth in all_auth {
                if let Some(provider) = providers.get(&auth.provider) {
                    let provider_auth_vec = provider.get_auth();
                    let mut provider_auth_vec_locked = provider_auth_vec.lock().unwrap();
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
            let auth_vec_locked = auth_vec.lock().unwrap();
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
                let mut provider_auth_vec_locked = provider_auth_vec.lock().unwrap();
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

    // Update the last synced timestamp
    *app.auth_last_synced_at.lock().await = Instant::now();

    Ok(())
}

/// Updates the state of a specific auth key based on the HTTP response status.
pub fn update_auth_state_on_response(auth: &Option<Arc<Mutex<ProviderAuth>>>, status: &StatusCode) {
    if let Some(auth_mutex) = auth {
        let auth_mutex_clone = auth_mutex.clone(); // Clone for potential async task
        let mut auth_locked = auth_mutex.lock().unwrap();
        auth_locked.used_at = chrono::Utc::now(); // Update usage time regardless of status

        match *status {
            StatusCode::OK => {
                auth_locked.sent += 1;
                // Optional: Log success if needed, but can be verbose
                // tracing::info!("Successfully used auth key {} for {}", auth_locked.id, auth_locked.provider);
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
    } else {
        // This case might happen if the original request already had an Authorization header
        // or if no suitable key was found (e.g., all keys on cooldown or invalid).
        tracing::debug!(
            "Attempted to update auth state, but no specific key was selected for the request."
        );
    }
}

/// Spawns a background task for regularly syncing the auth state with the database.
pub fn regular_auth_state_update(app: &Arc<AppState>) {
    let app = app.clone();
    tokio::spawn(async move {
        // Initial delay before first check? Optional.
        // tokio::time::sleep(Duration::from_secs(10)).await;

        loop {
            // Determine if a sync is needed based on time elapsed or forced interval
            let needs_sync = {
                let last_synced_at_locked = app.auth_last_synced_at.lock().await;
                let elapsed_since_sync = last_synced_at_locked.elapsed().as_secs();

                if elapsed_since_sync > FORCE_SYNC_INTERVAL_SECONS {
                    tracing::info!("Forcing auth sync due to interval.");
                    true
                } else if elapsed_since_sync > SYNC_INTERVAL_SECONDS {
                    // Check if any key was used recently enough to warrant a sync
                    let all_auth_in_memory = get_all_auth_from_memory(&app).await;
                    // Find the most recently used key
                    let most_recent_use = all_auth_in_memory
                        .iter()
                        .max_by_key(|auth_mutex| auth_mutex.lock().unwrap().used_at);

                    if let Some(auth_mutex) = most_recent_use {
                        let auth_locked = auth_mutex.lock().unwrap();
                        let elapsed_since_last_use = chrono::Utc::now()
                            .signed_duration_since(auth_locked.used_at)
                            .num_seconds();

                        if elapsed_since_last_use < SYNC_INTERVAL_SECONDS as i64 {
                            tracing::info!("Initiating auth sync due to recent activity.");
                            true
                        } else {
                            // Sync interval passed, but no recent activity
                            false
                        }
                    } else {
                        // No keys in memory, or none have been used yet
                        false
                    }
                } else {
                    // Sync interval not yet passed
                    false
                }
            }; // Lock scope ends here

            if needs_sync {
                if let Err(e) = sync_auth(&app).await {
                    tracing::error!("Regular background auth sync failed: {}", e);
                    // Consider adding retry logic or specific error handling here
                }
            }

            // Wait before the next check
            tokio::time::sleep(Duration::from_secs(SYNC_INTERVAL_SECONDS)).await;
        }
    });
}
