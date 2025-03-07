mod proxied_chat;
mod proxied_models;
mod health;
pub mod auths;

pub use proxied_chat::proxied_chat;
pub use proxied_models::proxied_models;
pub use health::health;

use crate::{
    app_state::AppState,
    proxy::webshare::{create_proxied_client, Proxy},
};
use eyre::Result;
use reqwest::Client;
use std::sync::Arc;

pub async fn handle_proxy_flag(
    app: &Arc<AppState>,
    flag: &str,
) -> Result<(Client, Option<Arc<Proxy>>)> {
    let (client, proxy) = match flag {
        "x" => (Client::builder().build().expect(""), None),
        "o" => match create_proxied_client(app).await {
            Ok(client) => client,
            Err(e) => return Err(e),
        },
        _ => return Err(eyre::eyre!("Invalid flag")),
    };
    Ok((client, proxy))
}
