use crate::app_state::AppState;
use chrono::{DateTime, Utc};
use eyre::Result;
use std::sync::Arc;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProviderAuth {
    pub id: i32,
    pub provider: String,
    pub api_key: String,
    pub sent: i32,
    pub max: i32,
    pub valid: bool,
    pub used_at: DateTime<Utc>,
    pub cooldown: bool,
    pub comments: Option<String>,
}

/// Fetches all authentication records from the database.
pub async fn db_get_all_auth(app: &Arc<AppState>) -> Result<Vec<ProviderAuth>> {
    let all_auth: Vec<ProviderAuth> = sqlx::query_as("SELECT * FROM auth")
        .fetch_all(&app.pool)
        .await?;
    Ok(all_auth)
}

/// Updates multiple authentication records in the database based on their IDs.
pub async fn db_update_auth(
    app: &Arc<AppState>,
    provider_auths: &Vec<ProviderAuth>,
) -> Result<u64> {
    let query = r#"
        UPDATE auth
        SET sent = u.sent,
            valid = u.valid,
            used_at = u.used_at,
            cooldown = u.cooldown
        FROM UNNEST($1::int[], $2::int[], $3::bool[], $4::timestamptz[], $5::bool[])
        AS u(id, sent, valid, used_at, cooldown)
        WHERE auth.id = u.id
    "#;

    let mut ids = Vec::with_capacity(provider_auths.len());
    let mut sents = Vec::with_capacity(provider_auths.len());
    let mut valids = Vec::with_capacity(provider_auths.len());
    let mut used_ats = Vec::with_capacity(provider_auths.len());
    let mut cooldowns = Vec::with_capacity(provider_auths.len());
    for pa in provider_auths {
        ids.push(pa.id);
        sents.push(pa.sent);
        valids.push(pa.valid);
        used_ats.push(pa.used_at);
        cooldowns.push(pa.cooldown);
    }

    let result = sqlx::query(query)
        .bind(&ids)
        .bind(&sents)
        .bind(&valids)
        .bind(&used_ats)
        .bind(&cooldowns)
        .execute(&app.pool)
        .await?;

    Ok(result.rows_affected())
}
