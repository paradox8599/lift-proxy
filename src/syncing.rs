use axum::http::HeaderMap;
use native_tls::{Certificate, TlsConnector};
use postgres_native_tls::MakeTlsConnector;
use reqwest as r;
use shuttle_runtime::SecretStore;
use tokio_postgres::types::ToSql;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(serde::Deserialize, Debug)]
pub struct Proxy {
    pub proxy_address: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct ProxyList {
    // pub count: u16,
    // pub next: Option<String>,
    // pub previous: Option<String>,
    pub results: Vec<Proxy>,
}

async fn get_proxies(secrets: &SecretStore) -> Result<Vec<Proxy>> {
    let url = r::Url::parse("https://proxy.webshare.io/api/v2/proxy/list")?;
    let query = vec![
        ("mode", "direct"),
        ("page", "1"),
        ("page_size", "10"),
        ("valid", "true"),
    ];

    let client = r::Client::new();

    let mut headers = HeaderMap::new();
    let webshare_token = secrets
        .get("WEBSHARE_TOKEN")
        .expect("WEBSHARE_TOKEN missing");
    headers.insert(
        "Authorization",
        format!("Token {}", webshare_token).parse().unwrap(),
    );

    let res = client
        .get(url)
        .query(&query)
        .headers(headers)
        .send()
        .await?;

    let json: ProxyList = res.json().await?;
    Ok(json.results)
}

async fn get_pg_client(secrets: &SecretStore) -> Result<tokio_postgres::Client> {
    let cert = std::fs::read("cacert.pem")?;
    let cert = Certificate::from_pem(&cert)?;
    let connector = TlsConnector::builder().add_root_certificate(cert).build()?;
    let connector = MakeTlsConnector::new(connector);

    let (client, conn) = tokio_postgres::connect(
        secrets
            .get("DATABASE_URL")
            .expect("DATABASE_URL missing")
            .as_str(),
        connector,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = conn.await {
            tracing::error!("connection error: {}", e);
        }
    });
    Ok(client)
}

pub async fn sync(secrets: &SecretStore) -> Result<()> {
    let proxies: Vec<Proxy> = get_proxies(secrets).await?;
    let client: tokio_postgres::Client = get_pg_client(secrets).await?;

    let sql = proxies
        .iter()
        .enumerate()
        .map(|(i, _)| format!("({}, ${}, ${})", i + 1, 2 * i + 1, 2 * i + 2))
        .collect::<Vec<String>>()
        .join(",");

    let values = proxies
        .iter()
        .flat_map(|proxy| {
            let name = format!("{}:{}", proxy.proxy_address, proxy.port);
            let base_url = format!(
                "https://lift-proxy-eyo5.shuttle.app/{}/{}:{}/deepinfra",
                name, proxy.username, proxy.password
            );
            vec![name, base_url]
        })
        .collect::<Vec<String>>();

    let params: Vec<&(dyn ToSql + Sync)> =
        values.iter().map(|s| s as &(dyn ToSql + Sync)).collect();

    let sql = format!("UPDATE channels SET name = data.name, base_url = data.base_url FROM (VALUES {}) as data(id, name, base_url) WHERE channels.id = data.id", sql);

    client.execute(&sql, &params).await?;

    tracing::info!("update ok");

    Ok(())
}
