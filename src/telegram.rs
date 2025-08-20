use anyhow::{Context, Result};
use reqwest::Client;

pub struct TelegramBot {
    token: String,
    chat_id: String,
    client: Client,
}

impl TelegramBot {
    pub fn new(token: impl Into<String>, chat_id: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            chat_id: chat_id.into(),
            client: Client::new(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let token = std::env::var("TELEGRAM_BOT_TOKEN").context("TELEGRAM_BOT_TOKEN missing")?;
        let chat_id = std::env::var("TELEGRAM_CHAT_ID").context("TELEGRAM_CHAT_ID missing")?;
        Ok(Self::new(token, chat_id))
    }

    pub async fn send_message(&self, text: &str) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        let resp = self
            .client
            .post(&url)
            .form(&[
                ("chat_id", self.chat_id.as_str()),
                ("text", text),
                ("parse_mode", "HTML"),
            ])
            .send()
            .await
            .context("failed to send request")?;
        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("telegram returned HTTP {}", status);
        }
        Ok(())
    }
}
