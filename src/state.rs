use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppState {
    pub windows: Vec<(u64, i32, i32)>,
    #[serde(default)]
    pub is_hidden: bool,
    #[serde(default)]
    pub is_flipped: bool,
}

pub fn get_cache_dir() -> Result<PathBuf> {
    let mut path = dirs::cache_dir().context("Could not find cache directory")?;
    path.push("niri-sidebar");
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}

pub fn load_state() -> Result<AppState> {
    let mut path = get_cache_dir()?;
    path.push("state.json");
    if path.exists() {
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content).unwrap_or_default())
    } else {
        Ok(AppState::default())
    }
}

pub fn save_state(state: &AppState) -> Result<()> {
    let mut path = get_cache_dir()?;
    path.push("state.json");
    let content = serde_json::to_string_pretty(state)?;
    fs::write(path, content)?;
    Ok(())
}
