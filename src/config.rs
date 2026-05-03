use serde::Deserialize;
use std::path::PathBuf;

use crate::terminal::grid::Color;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub font: FontConfig,

    #[serde(default)]
    pub window: WindowConfig,

    #[serde(default)]
    pub shell: ShellConfig,

    #[serde(default)]
    pub colors: ColorsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    #[serde(default = "default_blink_ms")]
    pub cursor_blink_ms: u32,
}

fn default_blink_ms() -> u32 { 500 }

#[derive(Debug, Clone, Deserialize)]
pub struct ShellConfig {
    pub program: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ColorsConfig {
    pub background: String,
    pub foreground: String,
    pub cursor: String,
    pub selection: String,
    /// 16-color ANSI palette: [black, red, green, yellow, blue, magenta, cyan, white,
    ///                          bright variants of each]
    pub palette: Vec<String>,
}

impl ColorsConfig {
    pub fn bg(&self) -> Color { parse_hex(&self.background) }
    pub fn fg(&self) -> Color { parse_hex(&self.foreground) }
    pub fn cursor(&self) -> Color { parse_hex(&self.cursor) }
    pub fn selection(&self) -> Color { parse_hex(&self.selection) }

    pub fn palette_colors(&self) -> [Color; 16] {
        let mut out = [Color::rgb(0, 0, 0); 16];
        for (i, hex) in self.palette.iter().enumerate().take(16) {
            out[i] = parse_hex(hex);
        }
        out
    }
}

pub fn parse_hex(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    let n = u32::from_str_radix(s, 16).unwrap_or(0);
    Color::rgb(((n >> 16) & 0xFF) as u8, ((n >> 8) & 0xFF) as u8, (n & 0xFF) as u8)
}

impl Default for FontConfig {
    fn default() -> Self {
        Self { family: "Noto Sans Mono".to_string(), size: 16.0 }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self { width: 800, height: 600, title: "mmterm".to_string(), cursor_blink_ms: 500 }
    }
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self { program: None }
    }
}

impl Default for ColorsConfig {
    fn default() -> Self {
        // Hardcore (Monokai) — from your Terminator profile
        Self {
            background: "#121212".into(),
            foreground: "#a0a0a0".into(),
            cursor:     "#bbbbbb".into(),
            selection:  "#3d3d3d".into(),
            palette: vec![
                "#1b1d1e".into(), "#f92672".into(), "#a6e22e".into(), "#fd971f".into(),
                "#66d9ef".into(), "#9e6ffe".into(), "#5e7175".into(), "#ccccc6".into(),
                "#505354".into(), "#ff669d".into(), "#beed5f".into(), "#e6db74".into(),
                "#66d9ef".into(), "#9e6ffe".into(), "#a3babf".into(), "#f8f8f2".into(),
            ],
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font: FontConfig::default(),
            window: WindowConfig::default(),
            shell: ShellConfig::default(),
            colors: ColorsConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        match std::fs::read_to_string(&path) {
            Ok(raw) => match toml::from_str(&raw) {
                Ok(cfg) => { log::info!("Loaded config from {}", path.display()); cfg }
                Err(e) => {
                    log::warn!("Invalid config at {}: {e} — using defaults", path.display());
                    Self::default()
                }
            },
            Err(_) => {
                log::info!("No config at {} — using defaults", path.display());
                Self::default()
            }
        }
    }

    pub fn save(&self) {
        let path = config_path();
        if let Some(dir) = path.parent() { let _ = std::fs::create_dir_all(dir); }
        let palette_toml = self.colors.palette.iter()
            .map(|c| format!("  {:?}", c))
            .collect::<Vec<_>>()
            .join(",\n");
        let content = format!(
r#"# mmterm configuration

[font]
family = {family:?}
size = {size}

[window]
width           = {width}
height          = {height}
title           = {title:?}
cursor_blink_ms = {blink_ms}

[shell]
{shell}

[colors]
background = {bg:?}
foreground = {fg:?}
cursor     = {cursor:?}
selection  = {sel:?}
palette = [
{palette}
]
"#,
            family  = self.font.family,
            size    = self.font.size,
            width    = self.window.width,
            height   = self.window.height,
            title    = self.window.title,
            blink_ms = self.window.cursor_blink_ms,
            shell   = self.shell.program.as_ref()
                .map(|s| format!("program = {s:?}"))
                .unwrap_or_else(|| "# program = \"/bin/zsh\"".into()),
            bg      = self.colors.background,
            fg      = self.colors.foreground,
            cursor  = self.colors.cursor,
            sel     = self.colors.selection,
            palette = palette_toml,
        );
        match std::fs::write(&path, &content) {
            Ok(_) => log::info!("Config saved to {}", path.display()),
            Err(e) => log::error!("Failed to save config: {e}"),
        }
    }

    pub fn write_default_if_missing() {
        let path = config_path();
        if path.exists() { return; }
        Self::default().save();
        log::info!("Created default config at {}", path.display());
    }
}

fn config_path() -> PathBuf {
    dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mmterm")
        .join("config.toml")
}
