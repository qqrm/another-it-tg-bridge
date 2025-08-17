mod state;
mod telegram;

use anyhow::{Context, Result};
use html_escape::encode_safe;
use log::info;
use regex::Regex;
use state::{load_state, save_state};
use telegram::TelegramBot;

const HOME_URL: &str = "https://another-it.ru/";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let mut state = load_state().await?;
    let client = reqwest::Client::new();

    let html = client
        .get(HOME_URL)
        .send()
        .await
        .context("failed to fetch homepage")?
        .text()
        .await?;

    let re = Regex::new(r#"<h2 class=\"entry-title[^\"]*\"><a href=\"([^\"]+)\""#)?;
    let caps = match re.captures(&html) {
        Some(c) => c,
        None => {
            info!("no articles found");
            return Ok(());
        }
    };
    let url = caps.get(1).unwrap().as_str().to_string();

    if state.last_url.as_deref() == Some(&url) {
        info!("no new posts");
        return Ok(());
    }

    let article_html = client
        .get(&url)
        .send()
        .await
        .context("failed to fetch article")?
        .text()
        .await?;
    let title_re = Regex::new(r#"<h1[^>]*>(.*?)</h1>"#)?;
    let title_caps = title_re
        .captures(&article_html)
        .context("title not found")?;
    let title_raw = title_caps.get(1).unwrap().as_str();
    let title = encode_safe(title_raw);

    let bot = TelegramBot::from_env()?;
    let message = format!("{title}\n{url}");
    bot.send_message(&message).await?;

    state.last_url = Some(url);
    save_state(&state).await?;

    Ok(())
}
