use crate::{
    app_state::AppState,
    providers::{ProviderAuth, ProviderFn as _},
};
use eyre::Result;
use std::sync::{Arc, Mutex};

pub async fn init_auth(app: &Arc<AppState>) {
    let all_auth: Vec<ProviderAuth> = sqlx::query_as("SELECT * FROM auth")
        .fetch_all(&app.pool)
        .await
        .unwrap();

    let providers = app.providers.lock().await;

    for auth in all_auth {
        if let Some(provider) = providers.get(&auth.provider) {
            let provider_auth = provider.get_auth();
            let mut provider_auth = provider_auth.lock().expect("");
            provider_auth.push(Arc::new(Mutex::new(auth)));
        } else {
            tracing::warn!("Mismatched auth provider: {:?}", auth);
        }
    }
}

pub async fn update_auth(app: &Arc<AppState>) -> Result<()> {
    let providers = app.providers.lock().await;
    let providers_vec = providers.values().cloned().collect::<Vec<_>>();
    let provider_auths = providers_vec
        .iter()
        .flat_map(|provider| {
            let auth = provider.get_auth();
            let auth = auth.lock().expect("");
            auth.iter()
                .map(|auth| auth.lock().expect("").clone())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

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
    for pa in &provider_auths {
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

    println!("Updated {} rows", result.rows_affected());

    let db_auth: Vec<ProviderAuth> = sqlx::query_as("SELECT * FROM auth")
        .fetch_all(&app.pool)
        .await?;

    // find new auth in db_auth that does not exist in provider_auths
    let new_auth = db_auth
        .iter()
        .filter(|auth| !provider_auths.iter().any(|pa| pa.id == auth.id))
        .cloned()
        .collect::<Vec<_>>();

    println!("Found {} new auths", new_auth.len());

    // add new auth to providers
    for auth in new_auth {
        if let Some(provider) = providers.get(&auth.provider) {
            let provider_auth = provider.get_auth();
            let mut provider_auth = provider_auth.lock().expect("");
            provider_auth.push(Arc::new(Mutex::new(auth)));
        } else {
            tracing::warn!("Mismatched auth provider: {:?}", auth);
        }
    }

    Ok(())
}
