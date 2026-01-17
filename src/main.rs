mod telegram;

use anyhow::{Context, Result};
use chrono::{Duration as ChronoDuration, FixedOffset, NaiveDate, Utc};
use log::{info, warn};
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use telegram::TelegramBot;

const GOOGLE_CSE_ENDPOINT: &str = "https://www.googleapis.com/customsearch/v1";
const GOOGLE_QUERY: &str = "site:another-it.ru";
const STATE_PATH: &str = "state/sent_urls.txt";
const DATE_WINDOW_DAYS: i64 = 30;
const GOOGLE_RESULTS: u8 = 10;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let client = Client::new();
    let api_key = std::env::var("GOOGLE_CSE_API_KEY").context("GOOGLE_CSE_API_KEY missing")?;
    let cx = std::env::var("GOOGLE_CSE_CX").context("GOOGLE_CSE_CX missing")?;

    let tz = FixedOffset::east_opt(3 * 3600).unwrap();
    let today_msk = Utc::now().with_timezone(&tz).date_naive();
    let window_start = today_msk - ChronoDuration::days(DATE_WINDOW_DAYS);
    let article_re = Regex::new(
        r"^https://another-it\.ru/(?P<year>\d{4})/(?P<month>\d{2})/(?P<day>\d{2})/[^/]+/?$",
    )
    .unwrap();

    let links = fetch_links(&client, &api_key, &cx).await?;
    info!("google cse returned {} links", links.len());

    let articles = filter_links(links, &article_re, window_start, today_msk);
    info!("links after filtering: {}", articles.len());

    let mut sent_urls = load_state(STATE_PATH)?;
    let mut new_articles: Vec<Article> = articles
        .into_iter()
        .filter(|article| !sent_urls.contains(&article.url))
        .collect();

    new_articles.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.url.cmp(&b.url)));

    info!("new links to send: {}", new_articles.len());

    if new_articles.is_empty() {
        retain_recent(&mut sent_urls, &article_re, window_start, today_msk);
        persist_state(STATE_PATH, &sent_urls)?;
        info!("no new posts");
        return Ok(());
    }

    let bot = TelegramBot::from_env()?;
    let mut sent = 0usize;
    let mut failed = 0usize;

    for article in &new_articles {
        if let Err(e) = bot.send_message(&article.url).await {
            warn!("telegram error for {}: {e}", article.url);
            failed += 1;
            continue;
        }
        sent_urls.insert(article.url.clone());
        sent += 1;
    }

    info!("sent {} new links, failed {}", sent, failed);

    retain_recent(&mut sent_urls, &article_re, window_start, today_msk);
    persist_state(STATE_PATH, &sent_urls)?;

    if failed > 0 {
        anyhow::bail!("telegram delivery failed for {} links", failed);
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct CseResponse {
    items: Option<Vec<CseItem>>,
}

#[derive(Debug, Deserialize)]
struct CseItem {
    link: String,
}

#[derive(Debug)]
struct Article {
    url: String,
    date: NaiveDate,
}

async fn fetch_links(client: &Client, api_key: &str, cx: &str) -> Result<Vec<String>> {
    let params = [
        ("key", api_key.to_string()),
        ("cx", cx.to_string()),
        ("q", GOOGLE_QUERY.to_string()),
        ("num", GOOGLE_RESULTS.to_string()),
        ("dateRestrict", format!("d{DATE_WINDOW_DAYS}")),
    ];

    let resp = client
        .get(GOOGLE_CSE_ENDPOINT)
        .query(&params)
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("failed to query Google CSE"))?;

    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("google cse returned HTTP {}", status);
    }

    let payload: CseResponse = resp
        .json()
        .await
        .map_err(|_| anyhow::anyhow!("failed to parse Google CSE response"))?;

    Ok(payload
        .items
        .unwrap_or_default()
        .into_iter()
        .map(|item| item.link)
        .collect())
}

fn filter_links(
    links: Vec<String>,
    article_re: &Regex,
    window_start: NaiveDate,
    today: NaiveDate,
) -> Vec<Article> {
    links
        .into_iter()
        .filter_map(|link| {
            let date = extract_date(&link, article_re)?;
            if date < window_start || date > today {
                return None;
            }
            Some(Article { url: link, date })
        })
        .collect()
}

fn extract_date(url: &str, article_re: &Regex) -> Option<NaiveDate> {
    let caps = article_re.captures(url)?;
    let year = caps.name("year")?.as_str().parse().ok()?;
    let month = caps.name("month")?.as_str().parse().ok()?;
    let day = caps.name("day")?.as_str().parse().ok()?;
    NaiveDate::from_ymd_opt(year, month, day)
}

fn load_state(path: &str) -> Result<HashSet<String>> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(HashSet::new()),
        Err(err) => Err(err).context("failed to read state file"),
    }
}

fn retain_recent(
    sent_urls: &mut HashSet<String>,
    article_re: &Regex,
    window_start: NaiveDate,
    today: NaiveDate,
) {
    sent_urls.retain(|url| {
        extract_date(url, article_re).is_some_and(|date| date >= window_start && date <= today)
    });
}

fn persist_state(path: &str, sent_urls: &HashSet<String>) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).context("failed to create state directory")?;
    }
    let mut urls: Vec<String> = sent_urls.iter().cloned().collect();
    urls.sort();
    let payload = urls.join("\n");
    fs::write(path, payload).context("failed to write state file")
}
