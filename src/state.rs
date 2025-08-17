use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

const STATE_FILE: &str = "state.json";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct State {
    pub last_url: Option<String>,
}

pub async fn load_state() -> Result<State> {
    if Path::new(STATE_FILE).exists() {
        let data = fs::read_to_string(STATE_FILE).await?;
        let state: State = serde_json::from_str(&data)?;
        Ok(state)
    } else {
        Ok(State::default())
    }
}

pub async fn save_state(state: &State) -> Result<()> {
    let data = serde_json::to_string(state)?;
    fs::write(STATE_FILE, data).await?;
    Ok(())
}
