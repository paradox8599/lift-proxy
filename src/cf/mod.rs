use reqwest as r;
use scraper::{Html, Selector};

pub async fn ppp() -> Result<(), Box<dyn std::error::Error>> {
    let client = r::Client::builder().build()?;
    let res = client
        .post(r::Url::parse("https://chutes.ai/app/api/chat")?)
        .send()
        .await?;
    let doc = Html::parse_document(&res.text().await?);
    let selector = Selector::parse("script").unwrap();
    let script = doc.select(&selector).next().unwrap();
    let js = script.inner_html();
    println!("{}", js);
    Ok(())
}
