use anyhow::Result;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::commands::KeyBinding;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub theme: Theme,
    #[serde(skip)]
    pub keybindings: Vec<KeyBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    #[serde(default = "default_fg")]
    pub fg: ColorDef,
    #[serde(default = "default_bg")]
    pub bg: ColorDef,
    #[serde(default = "default_accent")]
    pub accent: ColorDef,
    #[serde(default = "default_selection")]
    pub selection: ColorDef,
    #[serde(default = "default_border")]
    pub border: ColorDef,
    #[serde(default = "default_added")]
    pub added: ColorDef,
    #[serde(default = "default_removed")]
    pub removed: ColorDef,
    #[serde(default = "default_modified")]
    pub modified: ColorDef,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorDef {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb(u8, u8, u8),
}

impl From<ColorDef> for Color {
    fn from(color: ColorDef) -> Self {
        match color {
            ColorDef::Black => Color::Black,
            ColorDef::Red => Color::Red,
            ColorDef::Green => Color::Green,
            ColorDef::Yellow => Color::Yellow,
            ColorDef::Blue => Color::Blue,
            ColorDef::Magenta => Color::Magenta,
            ColorDef::Cyan => Color::Cyan,
            ColorDef::Gray => Color::Gray,
            ColorDef::DarkGray => Color::DarkGray,
            ColorDef::LightRed => Color::LightRed,
            ColorDef::LightGreen => Color::LightGreen,
            ColorDef::LightYellow => Color::LightYellow,
            ColorDef::LightBlue => Color::LightBlue,
            ColorDef::LightMagenta => Color::LightMagenta,
            ColorDef::LightCyan => Color::LightCyan,
            ColorDef::White => Color::White,
            ColorDef::Rgb(r, g, b) => Color::Rgb(r, g, b),
        }
    }
}

fn default_fg() -> ColorDef {
    ColorDef::White
}
fn default_bg() -> ColorDef {
    ColorDef::Black
}
fn default_accent() -> ColorDef {
    ColorDef::Cyan
}
fn default_selection() -> ColorDef {
    ColorDef::DarkGray
}
fn default_border() -> ColorDef {
    ColorDef::Gray
}
fn default_added() -> ColorDef {
    ColorDef::Green
}
fn default_removed() -> ColorDef {
    ColorDef::Red
}
fn default_modified() -> ColorDef {
    ColorDef::Yellow
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            fg: default_fg(),
            bg: default_bg(),
            accent: default_accent(),
            selection: default_selection(),
            border: default_border(),
            added: default_added(),
            removed: default_removed(),
            modified: default_modified(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents = std::fs::read_to_string(config_path)?;
            let mut config: Config = toml::from_str(&contents)?;
            config.keybindings = Self::default_keybindings();
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".config"))
            })
            .unwrap_or_else(|| PathBuf::from("."));

        Ok(config_dir.join("wind").join("tui.toml"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            keybindings: Self::default_keybindings(),
        }
    }
}

mod dirs {
    use std::path::PathBuf;

    pub fn config_dir() -> Option<PathBuf> {
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg_config));
        }
        if let Ok(home) = std::env::var("HOME") {
            return Some(PathBuf::from(home).join(".config"));
        }
        None
    }
}
