mod telegram;

use anyhow::{Context, Result};
use html_escape::encode_safe;
use log::info;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use telegram::TelegramBot;
use tokio::fs;

const HOME_URL: &str = "https://another-it.ru/";

#[derive(Debug, Serialize, Deserialize, Default)]
struct State {
    #[serde(default)]
    sent_urls: Vec<String>,
}

async fn load_state(path: &Path) -> Result<State> {
    if path.exists() {
        let data = fs::read_to_string(path).await?;
        let state: State = serde_json::from_str(&data)?;
        Ok(state)
    } else {
        Ok(State::default())
    }
}

async fn save_state(path: &Path, state: &State) -> Result<()> {
    let data = serde_json::to_string(state)?;
    fs::write(path, data).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let state_path = std::env::var("SENT_ARTICLES_PATH").unwrap_or_else(|_| "state.json".into());
    let mut state = load_state(Path::new(&state_path)).await?;
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
    urls.truncate(10);

    let mut to_send: Vec<String> = urls
        .into_iter()
        .filter(|u| !state.sent_urls.contains(u))
        .collect();

    if to_send.is_empty() {
        info!("no new posts");
        return Ok(());
    }

    // Send oldest messages first
    to_send.reverse();

    let bot = TelegramBot::from_env()?
    let title_re = Regex::new(r#"<h1[^>]*>(.*?)</h1>"#)?;
    for url in to_send.iter() {
        let article_html = client
            .get(url)
            .send()
            .await
            .context("failed to fetch article")?
            .text()
            .await?;

        let title_caps = title_re
            .captures(&article_html)
            .context("title not found")?;
        let title_raw = title_caps.get(1).unwrap().as_str();
        let title = encode_safe(title_raw);

        let message = format!("{title}\n{url}");
        bot.send_message(&message).await?;
    }

    state.sent_urls.extend(to_send);
    save_state(Path::new(&state_path), &state).await?;

    Ok(())
}
