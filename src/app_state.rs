use crate::{providers::Provider, proxy::webshare::Proxy};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::Mutex, time::Instant};

pub struct AppState {
    pub secrets: SecretStore,
    pub rng: Arc<Mutex<SmallRng>>,
    pub proxies_last_synced_at: Arc<Mutex<Instant>>,
    pub proxies: Arc<Mutex<Vec<Arc<Proxy>>>>,
    pub auth_last_synced_at: Arc<Mutex<Instant>>,
    pub providers: Arc<Mutex<HashMap<String, Arc<Provider>>>>,
    pub pool: PgPool,
}

impl AppState {
    pub fn new(secrets: SecretStore, pool: PgPool) -> Self {
        Self {
            secrets,
            rng: Arc::new(Mutex::new(SmallRng::from_os_rng())),
            proxies_last_synced_at: Arc::new(Mutex::new(tokio::time::Instant::now())),
            proxies: Arc::new(Mutex::new(vec![])),
            auth_last_synced_at: Arc::new(Mutex::new(tokio::time::Instant::now())),
            providers: Arc::new(Mutex::new(HashMap::new())),
            pool,
        }
    }

    pub async fn get_provider(&self, name: &str) -> Option<Arc<Provider>> {
        self.providers.lock().await.get(name).cloned()
    }
}
