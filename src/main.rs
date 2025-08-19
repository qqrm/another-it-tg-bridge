mod telegram;

use anyhow::{Context, Result};
use chrono::{Duration as ChronoDuration, FixedOffset, Utc};
use html_escape::encode_safe;
use log::{debug, info, warn};
use regex::Regex;
use telegram::TelegramBot;
use tokio::time::{sleep, Duration};

const HOME_URL: &str = "https://another-it.ru/";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    // First command line argument overrides TARGET_DATE for debugging
    let target_date = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("TARGET_DATE").ok());

    let run_number: u64 = std::env::var("GITHUB_RUN_NUMBER")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let is_first_run = run_number <= 1 && target_date.is_none();
    debug!("run_number={run_number}, target_date={target_date:?}, is_first_run={is_first_run}");
    let client = reqwest::Client::new();

    let html = client
        .get(HOME_URL)
        .send()
        .await
        .context("failed to fetch homepage")?
        .text()
        .await?;

    let re = Regex::new(r#"<h2 class=\"entry-title[^\"]*\"><a href=\"([^\"]+)\""#)?;
    let mut urls: Vec<String> = re
        .captures_iter(&html)
        .map(|c| c.get(1).unwrap().as_str().to_string())
        .collect();
    debug!("fetched {} urls", urls.len());
    urls.truncate(10);
    debug!("urls after truncate: {urls:?}");

    if !is_first_run {
        let filter_date = target_date.unwrap_or_else(|| {
            (Utc::now().with_timezone(&FixedOffset::east_opt(3 * 3600).unwrap())
                - ChronoDuration::days(1))
            .format("%Y/%m/%d")
            .to_string()
        });
        debug!("filtering by date {filter_date}");
        urls.retain(|u| u.contains(&filter_date));
        debug!("urls after filtering: {urls:?}");
    }

    if urls.is_empty() {
        info!("no new posts");
        return Ok(());
    }

    // Send oldest messages first
    urls.reverse();
    debug!("final url order: {urls:?}");

    let bot = TelegramBot::from_env()?;
    let title_re = Regex::new(r#"<h1[^>]*>(.*?)</h1>"#)?;
    for (idx, url) in urls.iter().enumerate() {
        debug!("processing url {url}");
        let article_html = match client.get(url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => text,
                Err(err) => {
                    warn!("failed to read article {url}: {err}");
                    continue;
                }
            },
            Err(err) => {
                warn!("failed to fetch article {url}: {err}");
                continue;
            }
        };

        let Some(title_caps) = title_re.captures(&article_html) else {
            warn!("title not found for {url}");
            continue;
        };
        let title_raw = title_caps.get(1).unwrap().as_str();
        let title = encode_safe(title_raw);

        bot.send_message(&format!("{title}\n{url}"))
            .await
            .with_context(|| format!("failed to send message for {url}"))?;
        debug!("sent message for {url}");

        if is_first_run && idx + 1 < urls.len() {
            sleep(Duration::from_secs(60)).await;
        }
    }
    Ok(())
}
