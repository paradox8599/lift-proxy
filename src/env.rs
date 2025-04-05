#[derive(Debug)]
pub struct Env {
    pub database_url: String,
    pub webshare_token: String,
    pub auth_secret: String,
}

impl Env {
    pub fn new() -> Self {
        let env = Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL not set"),
            webshare_token: std::env::var("WEBSHARE_TOKEN").expect("WEBSHARE_TOKEN not set"),
            auth_secret: std::env::var("AUTH_SECRET").expect("AUTH_SECRET not set"),
        };
        tracing::info!("Environment Loaded");
        env
    }
}
