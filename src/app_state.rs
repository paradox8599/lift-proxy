use crate::{providers::Provider, proxy::webshare::Proxy};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use shuttle_runtime::SecretStore;
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::Mutex, time::Instant};

pub struct AppState {
    pub secrets: SecretStore,
    pub rng: Arc<Mutex<SmallRng>>,
    pub last_synced_at: Arc<Mutex<Instant>>,
    pub proxies: Arc<Mutex<Vec<Arc<Proxy>>>>,
    pub providers: Arc<Mutex<HashMap<String, Arc<Provider>>>>,
}

impl AppState {
    pub fn new(secrets: SecretStore) -> Self {
        Self {
            secrets,
            rng: Arc::new(Mutex::new(SmallRng::from_os_rng())),
            last_synced_at: Arc::new(Mutex::new(tokio::time::Instant::now())),
            proxies: Arc::new(Mutex::new(vec![])),
            providers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_provider(&self, name: &str) -> Option<Arc<Provider>> {
        let providers = self.providers.lock().await;
        let provider = providers.get(name).cloned();
        provider
    }
}
