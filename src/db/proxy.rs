use crate::proxy::webshare::Proxy;
use eyre::Result;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbProxy {
    pub proxy_address: String,
    pub port: i32,
    pub username: String,
    pub password: String,
}

pub async fn db_save_proxies(pool: &PgPool, proxies: &[Arc<Proxy>]) -> Result<()> {
    sqlx::query("DELETE FROM proxies").execute(pool).await?;

    for proxy in proxies {
        sqlx::query(
            "INSERT INTO proxies (proxy_address, port, username, password) 
             VALUES ($1, $2, $3, $4)",
        )
        .bind(&proxy.proxy_address)
        .bind(proxy.port as i32)
        .bind(&proxy.username)
        .bind(&proxy.password)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn db_load_proxies(pool: &PgPool) -> Result<Vec<Arc<Proxy>>> {
    let db_proxies: Vec<DbProxy> = sqlx::query_as("SELECT * FROM proxies")
        .fetch_all(pool)
        .await?;

    Ok(db_proxies
        .into_iter()
        .map(|p| {
            Arc::new(Proxy {
                proxy_address: p.proxy_address,
                port: p.port as u16,
                username: p.username,
                password: p.password,
            })
        })
        .collect())
}
