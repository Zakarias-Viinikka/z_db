// Small, cheap-to-persist UI state that doesn't belong in Settings
// (which is user config, not a scratch pad).

use crate::edit::Position;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UiState {
    pub selection_name: Option<String>,
    pub paste: String,
    pub settings_open: bool,
}

fn path() -> PathBuf {
    crate::settings::config_dir().join("ui_state.json")
}

pub fn load() -> UiState {
    let Ok(text) = std::fs::read_to_string(path()) else {
        return UiState::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save(state: &UiState) -> Result<(), String> {
    let dir = crate::settings::config_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("couldn't create config dir {}: {}", dir.display(), e))?;
    let text =
        serde_json::to_string_pretty(state).map_err(|e| format!("couldn't serialise: {}", e))?;
    let p = path();
    std::fs::write(&p, text).map_err(|e| format!("couldn't write {}: {}", p.display(), e))?;
    Ok(())
}

pub fn selection_to_name(sel: &crate::edit::Position) -> Option<String> {
    match sel {
        Position::Bottom => None,
        Position::Named(n) => Some(n.clone()),
    }
}
