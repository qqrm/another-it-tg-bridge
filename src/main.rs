mod telegram;

use anyhow::{Context, Result};
use chrono::{Duration as ChronoDuration, FixedOffset, NaiveDate, Utc};
use html_escape::encode_safe;
use log::{info, warn};
use reqwest::{Client, StatusCode};
use scraper::{Html, Selector};
use telegram::TelegramBot;

const BASE_URL: &str = "https://another-it.ru";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let date = determine_date()?;
    let page_url = format!("{BASE_URL}/{date}/");
    let client = Client::new();

    let resp = client
        .get(&page_url)
        .send()
        .await
        .context("failed to fetch daily page")?;

    if resp.status() == StatusCode::NOT_FOUND {
        info!("no new posts");
        return Ok(());
    }

    let page_html = resp
        .error_for_status()
        .context("failed to fetch daily page")?
        .text()
        .await
        .context("failed to read daily page")?;

    let doc = Html::parse_document(&page_html);
    let link_selector = Selector::parse("main#site-content h2.entry-title > a").unwrap();
    let links: Vec<String> = doc
        .select(&link_selector)
        .filter_map(|a| a.value().attr("href"))
        .map(|href| href.to_string())
        .filter(|href| href.contains(&date))
        .collect();

    if links.is_empty() {
        info!("no new posts");
        return Ok(());
    }

    let bot = TelegramBot::from_env()?;
    let title_selector = Selector::parse("h1").unwrap();
    let mut sent = 0usize;

    for url in links.iter().rev() {
        let article_html = match client.get(url).send().await {
            Ok(resp) => match resp.error_for_status() {
                Ok(ok) => match ok.text().await {
                    Ok(text) => text,
                    Err(e) => {
                        warn!("failed to read article {url}: {e}");
                        continue;
                    }
                },
                Err(e) => {
                    warn!("article {url} returned error: {e}");
                    continue;
                }
            },
            Err(e) => {
                warn!("failed to fetch article {url}: {e}");
                continue;
            }
        };

        let article_doc = Html::parse_document(&article_html);
        let Some(title_el) = article_doc.select(&title_selector).next() else {
            warn!("title not found for {url}");
            continue;
        };
        let title = encode_safe(&title_el.inner_html()).to_string();

        if let Err(e) = bot.send_message(&format!("{title}\n{url}")).await {
            warn!("telegram error for {url}: {e}");
            continue;
        }
        sent += 1;
    }

    if sent == 0 {
        anyhow::bail!("no messages sent");
    }

    Ok(())
}

fn determine_date() -> Result<String> {
    if let Some(arg) = std::env::args().nth(1) {
        parse_date(&arg)
    } else {
        Ok(yesterday_moscow())
    }
}

fn parse_date(input: &str) -> Result<String> {
    let normalized = input.replace('-', "/");
    let date = NaiveDate::parse_from_str(&normalized, "%Y/%m/%d").context("invalid date")?;
    Ok(date.format("%Y/%m/%d").to_string())
}

fn yesterday_moscow() -> String {
    let tz = FixedOffset::east_opt(3 * 3600).unwrap();
    (Utc::now().with_timezone(&tz) - ChronoDuration::days(1))
        .format("%Y/%m/%d")
        .to_string()
}
