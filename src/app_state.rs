use crate::proxy::webshare::Proxy;
use shuttle_runtime::SecretStore;
use tokio::sync::Mutex;

pub struct AppState {
    pub secrets: SecretStore,
    pub last_synced_at: Mutex<tokio::time::Instant>,
    pub rng: Mutex<rand::rngs::SmallRng>,
    pub proxies: Mutex<Vec<Proxy>>,
}
