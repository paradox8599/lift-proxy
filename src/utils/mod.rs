mod create_client;
mod get_response_stream;

pub use create_client::create_client;
pub use get_response_stream::get_response_stream;
use shuttle_runtime::SecretStore;

use tokio::sync::Mutex;

pub struct AppState {
    pub secrets: SecretStore,
    pub last_synced_at: Mutex<tokio::time::Instant>,
}
