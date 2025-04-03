use crate::app_state::AppState;
use crate::db::proxy::{db_load_proxies, db_save_proxies};
use axum::http::HeaderMap;
use eyre::Result;
use rand::Rng;
use reqwest as r;
use shuttle_runtime::SecretStore;
use std::time::Duration;
use std::{fmt::Display, sync::Arc};
use tokio::time::Instant;

const PROXY_UPDATE_DEBOUNCE: Duration = Duration::from_secs(5 * 60);

#[allow(dead_code)]
#[derive(serde::Deserialize, Debug, Clone, Default)]
pub struct Proxy {
    pub proxy_address: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

impl Display for Proxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "socks5://{}:{}@{}:{}",
            self.username, self.password, self.proxy_address, self.port
        )
    }
}

#[allow(dead_code)]
#[derive(serde::Deserialize, Debug, Clone)]
pub struct ProxyList {
    pub count: u16,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<Proxy>,
}

async fn get_proxies(secrets: &SecretStore) -> eyre::Result<Vec<Arc<Proxy>>> {
    let client = r::Client::new();

    let mut headers = HeaderMap::new();
    let webshare_token = secrets
        .get(crate::constants::WEBSHARE_TOKEN)
        .ok_or(eyre::eyre!("Missing webshare token"))?;
    headers.insert(
        "Authorization",
        format!("Token {}", webshare_token).parse()?,
    );

    let init_url = format!(
        "https://proxy.webshare.io/api/v2/proxy/list?{}&{}&{}&{}",
        "mode=direct", "page=1", "page_size=4", "valid=true"
    );

    let mut next: Option<String> = Some(init_url);
    let mut proxies: Vec<Proxy> = vec![];

    while let Some(url) = next {
        let res = client.get(url).headers(headers.clone()).send().await?;
        let proxy_list: ProxyList = res.json().await?;
        let mut local_proxies = proxy_list.results;
        proxies.append(&mut local_proxies);
        next = proxy_list.next;
    }

    let proxies = proxies.iter().map(|p| Arc::new(p.clone())).collect();
    Ok(proxies)
}

pub async fn update_proxies(app: &Arc<AppState>) -> Result<()> {
    let new_proxies = get_proxies(&app.secrets).await?;
    db_save_proxies(&app.pool, &new_proxies).await?;

    let mut proxies = app.proxies.lock().await;
    *proxies = new_proxies;
    tracing::info!("[Proxy] Updated and saved to database");
    Ok(())
}

pub async fn init_proxies(app: &Arc<AppState>) {
    match db_load_proxies(&app.pool).await {
        Ok(db_proxies) if !db_proxies.is_empty() => {
            let mut proxies = app.proxies.lock().await;
            *proxies = db_proxies;
            tracing::info!("[Proxy] Loaded {} proxies from database", proxies.len());
        }
        Err(e) => tracing::warn!("Failed to load proxies from DB: {}", e),
        _ => (),
    }

    if app.proxies.lock().await.is_empty() {
        if let Err(e) = update_proxies(app).await {
            panic!("Error init proxies: {}", e);
        }
    }
}

pub fn update_proxies_debounced(app: &Arc<AppState>) {
    let app = app.clone();
    tracing::info!("[Proxy] Updating proxies");
    tokio::spawn(async move {
        let mut last_synced_at = app.proxies_last_synced_at.lock().await;
        let elapsed = last_synced_at.elapsed();
        if elapsed > PROXY_UPDATE_DEBOUNCE {
            match update_proxies(&app).await {
                Ok(_) => *last_synced_at = Instant::now(),
                Err(e) => tracing::error!("[Sync] Error: {}", e),
            }
        }
    });
}

pub async fn pick_proxy(app: &Arc<AppState>) -> Option<Arc<Proxy>> {
    let proxies = app.proxies.lock().await;
    if proxies.is_empty() {
        return None;
    }
    let size = proxies.len();
    let mut rng = app.rng.lock().await;
    let i = rng.random_range(0..size);
    proxies.get(i).cloned()
}

pub async fn create_proxied_client(app: &Arc<AppState>) -> Result<(r::Client, Option<Arc<Proxy>>)> {
    update_proxies_debounced(app);
    match pick_proxy(app).await {
        Some(proxy) => {
            let client = r::Client::builder();
            let req_proxy = r::Proxy::all(proxy.to_string())?;
            Ok((client.proxy(req_proxy.clone()).build()?, Some(proxy)))
        }
        None => Err(eyre::eyre!("No available proxy")),
    }
}

pub async fn disable_failed_proxy(app: &Arc<AppState>, proxy: &Option<Arc<Proxy>>) {
    if let Some(proxy) = &proxy {
        let mut proxies = app.proxies.lock().await;
        let index = proxies
            .iter()
            .position(|p| p.proxy_address == proxy.proxy_address);
        if let Some(index) = index {
            let proxy = &proxies[index];
            tracing::info!(
                "Disabling failed proxy: {}:{}",
                proxy.proxy_address,
                proxy.port
            );
            proxies.remove(index);
        }
    }
}
