use reqwest::{self as r};

pub fn create_client(
    proxy_addr: Option<String>,
    proxy_auth: Option<String>,
) -> r::Result<r::Client> {
    let mut client = r::Client::builder();

    if let Some(proxy_str) = proxy_addr {
        if let Ok(mut proxy) = r::Proxy::all(format!("socks5://{}", proxy_str)) {
            if let Some(proxy_auth) = proxy_auth {
                let mut s = proxy_auth.split(':');
                if let (Some(u), Some(p)) = (s.next(), s.next()) {
                    proxy = proxy.basic_auth(u, p);
                }
            }
            client = client.proxy(proxy)
        }
    }
    client.build()
}
