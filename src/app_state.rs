use crate::{providers::Provider, proxy::webshare::Proxy};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::Mutex, time::Instant};

pub struct AppState {
    pub pool: PgPool,
    pub secrets: SecretStore,
    pub rng: Arc<Mutex<SmallRng>>,
    pub proxies: Arc<Mutex<Vec<Arc<Proxy>>>>,
    pub proxies_last_synced_at: Arc<Mutex<Instant>>,
    pub providers: Arc<Mutex<HashMap<String, Arc<Provider>>>>,
    pub show_chat: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new(pool: PgPool, secrets: SecretStore) -> Self {
        Self {
            pool,
            secrets,
            rng: Arc::new(Mutex::new(SmallRng::from_os_rng())),
            proxies: Arc::new(Mutex::new(vec![])),
            proxies_last_synced_at: Arc::new(Mutex::new(tokio::time::Instant::now())),
            providers: Arc::new(Mutex::new(HashMap::new())),
            show_chat: Arc::new(Mutex::new(true)),
        }
    }

    pub async fn get_provider(&self, name: &str) -> Option<Arc<Provider>> {
        self.providers.lock().await.get(name).cloned()
    }
}
