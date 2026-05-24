use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSession {
    pub active_tab: usize,
    pub tabs: Vec<SavedTab>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedTab {
    pub name: Option<String>,
    /// Index into `pane_cwds` that was the active pane (DFS leaf order).
    pub active_pane: usize,
    /// CWD per pane in DFS leaf order; empty = fall back to $HOME.
    pub pane_cwds: Vec<PathBuf>,
    pub layout: SavedNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SavedNode {
    Leaf {
        slot: usize,
    },
    Split {
        dir: SavedSplitDir,
        ratio: f32,
        a: Box<SavedNode>,
        b: Box<SavedNode>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SavedSplitDir {
    H,
    V,
}

pub fn session_path() -> PathBuf {
    dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mmterm")
        .join("session.toml")
}

pub fn save(session: &SavedSession) -> anyhow::Result<()> {
    let path = session_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let content =
        toml::to_string_pretty(session).map_err(|e| anyhow::anyhow!("serialize session: {e}"))?;
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, &content)?;
    std::fs::rename(&tmp, &path)?;
    log::info!("Session saved to {}", path.display());
    Ok(())
}

pub fn load() -> Option<SavedSession> {
    let path = session_path();
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return None,
    };
    match toml::from_str::<SavedSession>(&raw) {
        Ok(session) => {
            log::info!("Session loaded from {}", path.display());
            Some(session)
        }
        Err(e) => {
            log::warn!("Failed to parse session {}: {e}", path.display());
            None
        }
    }
}

#[cfg(test)]
#[path = "session_test.rs"]
mod tests;
