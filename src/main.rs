mod telegram;

use anyhow::{Context, Result};
use html_escape::encode_safe;
use log::{info, warn};
use regex::Regex;
use telegram::TelegramBot;

const HOME_URL: &str = "https://another-it.ru/";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let client = reqwest::Client::new();

    let html = client
        .get(HOME_URL)
        .send()
        .await
        .context("failed to fetch homepage")?
        .text()
        .await?;

    let re = Regex::new(r#"<h2 class="entry-title[^"]*"><a href="([^"]+)""#)?;
    let mut urls: Vec<String> = re
        .captures_iter(&html)
        .map(|c| c.get(1).unwrap().as_str().to_string())
        .collect();
    urls.truncate(10);

    if urls.is_empty() {
        info!("no posts");
        return Ok(());
    }

    // Send oldest messages first
    urls.reverse();

    let bot = TelegramBot::from_env()?;
    let title_re = Regex::new(r#"<h1[^>]*>(.*?)</h1>"#)?;
    for url in urls.iter() {
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

        if let Err(err) = bot
            .send_message(&format!(
                "{title}
{url}"
            ))
            .await
        {
            warn!("failed to send message for {url}: {err}");
        }
    }

    Ok(())
}
