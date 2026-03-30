mod telegram;

use anyhow::{Context, Result};
use chrono::{Datelike, Duration as ChronoDuration, FixedOffset, NaiveDate, Utc};
use log::{info, warn};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use telegram::TelegramBot;

const GOOGLE_CSE_ENDPOINT: &str = "https://www.googleapis.com/customsearch/v1";
const GOOGLE_QUERY_PREFIX: &str = "site:another-it.ru/";
const DEFAULT_STATE_PATH: &str = "state/sent_urls.txt";
const STATE_RETENTION_DAYS: i64 = 30;
const AUTO_LOOKBACK_DAYS: i64 = 7;
const GOOGLE_RESULTS: u8 = 10;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let client = Client::new();
    let api_key = std::env::var("GOOGLE_CSE_API_KEY").context("GOOGLE_CSE_API_KEY missing")?;
    let cx = std::env::var("GOOGLE_CSE_CX").context("GOOGLE_CSE_CX missing")?;
    let state_path = std::env::var("STATE_PATH").unwrap_or_else(|_| DEFAULT_STATE_PATH.to_string());

    let tz = FixedOffset::east_opt(3 * 3600).unwrap();
    let today_msk = Utc::now().with_timezone(&tz).date_naive();
    let state_window_start = today_msk - ChronoDuration::days(STATE_RETENTION_DAYS);
    let target_dates = determine_target_dates(today_msk)?;
    info!("checking {} dates via Google CSE", target_dates.len());

    let mut articles = Vec::new();
    for target_date in target_dates {
        let items = fetch_items(&client, &api_key, &cx, &google_query(target_date)).await?;
        info!(
            "google cse returned {} items for {}",
            items.len(),
            target_date.format("%Y-%m-%d")
        );

        let filtered = filter_items(&items, target_date);
        if filtered.is_empty() && !items.is_empty() {
            info!(
                "google candidate links for {}: {}",
                target_date.format("%Y-%m-%d"),
                format_candidate_links(&items)
            );
        }
        info!(
            "links after filtering for {}: {}",
            target_date.format("%Y-%m-%d"),
            filtered.len()
        );
        articles.extend(filtered);
    }

    let mut sent_urls = load_state(&state_path)?;
    let mut new_articles: Vec<Article> = articles
        .into_iter()
        .filter(|article| !sent_urls.contains(&article.url))
        .collect();

    new_articles.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.url.cmp(&b.url)));

    info!("new links to send: {}", new_articles.len());

    if new_articles.is_empty() {
        retain_recent(&mut sent_urls, state_window_start, today_msk);
        persist_state(&state_path, &sent_urls)?;
        info!("no new posts");
        return Ok(());
    }

    let bot = TelegramBot::from_env()?;
    let mut sent = 0usize;
    let mut failed = 0usize;

    for article in &new_articles {
        let text = match article
            .title
            .as_ref()
            .map(|title| title.trim())
            .filter(|title| !title.is_empty())
        {
            Some(title) => format!("{title}\n{}", article.url),
            None => article.url.clone(),
        };

        if let Err(e) = bot.send_message(&text).await {
            warn!("telegram error for {}: {e}", article.url);
            failed += 1;
            continue;
        }
        sent_urls.insert(article.url.clone());
        sent += 1;
    }

    info!("sent {} new links, failed {}", sent, failed);

    retain_recent(&mut sent_urls, state_window_start, today_msk);
    persist_state(&state_path, &sent_urls)?;

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
    title: Option<String>,
}

#[derive(Debug)]
struct Article {
    url: String,
    title: Option<String>,
    date: NaiveDate,
}

async fn fetch_items(
    client: &Client,
    api_key: &str,
    cx: &str,
    query: &str,
) -> Result<Vec<CseItem>> {
    let params = [
        ("key", api_key.to_string()),
        ("cx", cx.to_string()),
        ("q", query.to_string()),
        ("num", GOOGLE_RESULTS.to_string()),
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

    Ok(payload.items.unwrap_or_default())
}

fn filter_items(items: &[CseItem], target_date: NaiveDate) -> Vec<Article> {
    items
        .iter()
        .filter_map(|item| {
            let url = normalize_article_url(&item.link, target_date)?;
            Some(Article {
                url,
                title: item
                    .title
                    .as_deref()
                    .map(str::trim)
                    .filter(|title| !title.is_empty())
                    .map(str::to_string),
                date: target_date,
            })
        })
        .collect()
}

fn google_query(target_date: NaiveDate) -> String {
    format!("{GOOGLE_QUERY_PREFIX}{}/", target_date.format("%Y/%m/%d"))
}

fn determine_target_dates(today_msk: NaiveDate) -> Result<Vec<NaiveDate>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => Ok((1..=AUTO_LOOKBACK_DAYS)
            .map(|days_ago| today_msk - ChronoDuration::days(days_ago))
            .collect()),
        [date] => Ok(vec![parse_date_arg(date)?]),
        [flag, days] if flag == "--days" => trailing_dates(today_msk, parse_days_arg(days)?),
        _ => anyhow::bail!("usage: another-it-tg-bridge [YYYY/MM/DD|YYYY-MM-DD] [--days N]"),
    }
}

fn trailing_dates(today_msk: NaiveDate, days: i64) -> Result<Vec<NaiveDate>> {
    if days <= 0 {
        anyhow::bail!("days must be greater than 0");
    }

    Ok((0..days)
        .map(|days_ago| today_msk - ChronoDuration::days(days_ago))
        .collect())
}

fn parse_date_arg(input: &str) -> Result<NaiveDate> {
    let normalized = input.replace('-', "/");
    NaiveDate::parse_from_str(&normalized, "%Y/%m/%d").context("invalid date")
}

fn parse_days_arg(input: &str) -> Result<i64> {
    input.parse::<i64>().context("invalid days")
}

fn normalize_article_url(raw_url: &str, target_date: NaiveDate) -> Option<String> {
    let mut url = reqwest::Url::parse(raw_url).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }

    match url.host_str()? {
        "another-it.ru" | "www.another-it.ru" => {}
        _ => return None,
    }

    let expected_segments = [
        target_date.year().to_string(),
        format!("{:02}", target_date.month()),
        format!("{:02}", target_date.day()),
    ];

    let path_segments: Vec<String> = url
        .path_segments()?
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect();
    if path_segments.len() != 4 || path_segments[3].is_empty() {
        return None;
    }
    if path_segments[0] != expected_segments[0]
        || path_segments[1] != expected_segments[1]
        || path_segments[2] != expected_segments[2]
    {
        return None;
    }

    url.set_scheme("https").ok()?;
    url.set_host(Some("another-it.ru")).ok()?;
    url.set_query(None);
    url.set_fragment(None);
    url.set_path(&format!(
        "/{}/{}/{}/{}/",
        path_segments[0], path_segments[1], path_segments[2], path_segments[3]
    ));

    Some(url.into())
}

fn format_candidate_links(items: &[CseItem]) -> String {
    items
        .iter()
        .map(|item| item.link.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn extract_date(url: &str) -> Option<NaiveDate> {
    let parsed = reqwest::Url::parse(url).ok()?;
    match parsed.host_str()? {
        "another-it.ru" | "www.another-it.ru" => {}
        _ => return None,
    }

    let segments: Vec<_> = parsed
        .path_segments()?
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.len() != 4 {
        return None;
    }

    let year = segments[0].parse().ok()?;
    let month = segments[1].parse().ok()?;
    let day = segments[2].parse().ok()?;
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

fn retain_recent(sent_urls: &mut HashSet<String>, window_start: NaiveDate, today: NaiveDate) {
    sent_urls
        .retain(|url| extract_date(url).is_some_and(|date| date >= window_start && date <= today));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_debug_date_with_dashes() {
        let date = parse_date_arg("2026-03-28").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 3, 28).unwrap());
    }

    #[test]
    fn parses_days_argument() {
        assert_eq!(parse_days_arg("14").unwrap(), 14);
    }

    #[test]
    fn trailing_dates_include_today() {
        let today = NaiveDate::from_ymd_opt(2026, 3, 30).unwrap();
        let dates = trailing_dates(today, 3).unwrap();

        assert_eq!(
            dates,
            vec![
                NaiveDate::from_ymd_opt(2026, 3, 30).unwrap(),
                NaiveDate::from_ymd_opt(2026, 3, 29).unwrap(),
                NaiveDate::from_ymd_opt(2026, 3, 28).unwrap(),
            ]
        );
    }

    #[test]
    fn normalizes_article_url_and_strips_query() {
        let target_date = NaiveDate::from_ymd_opt(2026, 3, 28).unwrap();
        let normalized = normalize_article_url(
            "http://www.another-it.ru/2026/03/28/test-post/?utm_source=google#frag",
            target_date,
        )
        .unwrap();

        assert_eq!(normalized, "https://another-it.ru/2026/03/28/test-post/");
    }

    #[test]
    fn rejects_archive_page_urls() {
        let target_date = NaiveDate::from_ymd_opt(2026, 3, 28).unwrap();
        assert!(normalize_article_url("https://another-it.ru/2026/03/28/", target_date).is_none());
    }
}
