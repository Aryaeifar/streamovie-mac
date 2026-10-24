//! Persistent JSON storage for user preferences (`settings.json`): locale and
//! accent palette. The interface is always the light Apple canvas.
#![allow(dead_code)]

use std::{fs, path::PathBuf};

use anyhow::{Context as _, Result};
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{App, Hsla, Window, hsla, px};
use serde::{Deserialize, Serialize};

use crate::{
    i18n::{Dictionary, Locale},
    storage::app_data_dir,
};

/// Streamovie brand accent palettes ported from `Streamovie/src/lib/themes.ts`
/// and `Streamovie/src/app/globals.css`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccentPalette {
    #[default]
    Apple,
    Paramount,
    Netflix,
    Midnight,
    Aurora,
    Ocean,
    Sunset,
    Candy,
    Noir,
}

impl AccentPalette {
    pub const ALL: [Self; 9] = [
        Self::Apple,
        Self::Paramount,
        Self::Netflix,
        Self::Midnight,
        Self::Aurora,
        Self::Ocean,
        Self::Sunset,
        Self::Candy,
        Self::Noir,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::Apple => "apple",
            Self::Paramount => "paramount",
            Self::Netflix => "netflix",
            Self::Midnight => "midnight",
            Self::Aurora => "aurora",
            Self::Ocean => "ocean",
            Self::Sunset => "sunset",
            Self::Candy => "candy",
            Self::Noir => "noir",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "apple" => Self::Apple,
            "paramount" => Self::Paramount,
            "netflix" => Self::Netflix,
            "midnight" => Self::Midnight,
            "aurora" => Self::Aurora,
            "ocean" => Self::Ocean,
            "sunset" => Self::Sunset,
            "candy" => Self::Candy,
            "noir" => Self::Noir,
            _ => Self::Apple,
        }
    }

    pub fn label(self, dict: &Dictionary) -> &'static str {
        match self {
            Self::Apple => dict.settings.themes.apple,
            Self::Paramount => dict.settings.themes.paramount,
            Self::Netflix => dict.settings.themes.netflix,
            Self::Midnight => dict.settings.themes.midnight,
            Self::Aurora => dict.settings.themes.aurora,
            Self::Ocean => dict.settings.themes.ocean,
            Self::Sunset => dict.settings.themes.sunset,
            Self::Candy => dict.settings.themes.candy,
            Self::Noir => dict.settings.themes.noir,
        }
    }

    /// Primary accent color for the Streamovie cinema theme.
    pub fn primary_hsla(self) -> Hsla {
        match self {
            Self::Apple => hsla(211.0 / 360.0, 1.0, 0.52, 1.0), // macOS system blue
            Self::Paramount => hsla(211.0 / 360.0, 1.0, 0.50, 1.0),
            Self::Netflix => hsla(357.0 / 360.0, 0.92, 0.47, 1.0), // #e50914
            Self::Midnight => hsla(258.0 / 360.0, 0.90, 0.66, 1.0), // #8b5cf6
            Self::Aurora => hsla(160.0 / 360.0, 0.84, 0.39, 1.0),  // #10b981
            Self::Ocean => hsla(199.0 / 360.0, 0.89, 0.48, 1.0),   // #0ea5e9
            Self::Sunset => hsla(25.0 / 360.0, 0.95, 0.53, 1.0),   // #f97316
            Self::Candy => hsla(330.0 / 360.0, 0.81, 0.60, 1.0),   // #ec4899
            Self::Noir => hsla(240.0 / 360.0, 0.05, 0.65, 1.0),    // #a1a1aa
        }
    }

}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedSettings {
    #[serde(default)]
    pub locale: Locale,
    #[serde(default)]
    pub accent_palette: AccentPalette,
}

impl PersistedSettings {
    /// Applies the light canvas, brand accent, and locale font to `uni-kit`'s global `Theme`.
    pub fn apply_to_uni_kit(&self, window: Option<&mut Window>, cx: &mut App) {
        Theme::change(ThemeMode::Light, window, cx);

        let primary = self.accent_palette.primary_hsla();
        let font_family = self.locale.font_family();

        Theme::update(cx, |theme| {
            let ink = hsla(240.0 / 360.0, 0.03, 0.118, 1.0);
            let canvas = hsla(240.0 / 360.0, 0.11, 0.965, 1.0);
            theme.primary = primary;
            theme.ring = primary;
            theme.sidebar_primary = primary;
            theme.radius = px(13.);
            theme.radius_lg = px(16.);
            theme.font_family = font_family;
            theme.background = canvas;
            theme.foreground = ink;
            theme.popover = hsla(0.0, 0.0, 1.0, 0.96);
            theme.popover_foreground = ink;
            theme.sidebar = hsla(0.0, 0.0, 1.0, 0.86);
            theme.sidebar_foreground = ink;
            theme.sidebar_border = hsla(0.0, 0.0, 0.0, 0.08);
            theme.sidebar_accent = hsla(0.0, 0.0, 0.0, 0.05);
            theme.sidebar_accent_foreground = ink;
            theme.muted = hsla(240.0 / 360.0, 0.08, 0.94, 1.0);
            theme.muted_foreground = hsla(240.0 / 360.0, 0.02, 0.44, 1.0);
            theme.border = hsla(0.0, 0.0, 0.0, 0.08);
            theme.input = hsla(0.0, 0.0, 1.0, 1.0);
            theme.title_bar = canvas;
            theme.title_bar_border = hsla(0.0, 0.0, 0.0, 0.06);
            theme.accent = hsla(0.0, 0.0, 0.0, 0.05);
            theme.accent_foreground = ink;
            theme.list_hover = hsla(0.0, 0.0, 0.0, 0.04);
            theme.list_active = hsla(0.0, 0.0, 0.0, 0.06);
            theme.group_box = hsla(0.0, 0.0, 1.0, 1.0);
            theme.group_box_foreground = ink;
            let secondary = hsla(240.0 / 360.0, 0.08, 0.95, 1.0);
            let secondary_hover = hsla(240.0 / 360.0, 0.06, 0.92, 1.0);
            let secondary_active = hsla(240.0 / 360.0, 0.04, 0.88, 1.0);
            theme.secondary = secondary;
            theme.secondary_hover = secondary_hover;
            theme.secondary_active = secondary_active;
            theme.secondary_foreground = ink;
            theme.button_secondary = secondary;
            theme.button_secondary_hover = secondary_hover;
            theme.button_secondary_active = secondary_active;
            theme.button_secondary_foreground = ink;
            theme.button_primary = ink;
            theme.button_primary_hover = hsla(240.0 / 360.0, 0.02, 0.20, 1.0);
            theme.button_primary_active = hsla(0.0, 0.0, 0.0, 1.0);
            theme.button_primary_foreground = hsla(0.0, 0.0, 1.0, 1.0);
            theme.primary_foreground = hsla(0.0, 0.0, 1.0, 1.0);
            theme.skeleton = hsla(240.0 / 360.0, 0.06, 0.90, 1.0);
            theme.progress_bar = primary;
        });
    }
}

#[derive(Clone, Debug)]
pub struct SettingsStorage {
    file_path: PathBuf,
}

impl Default for SettingsStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsStorage {
    pub fn new() -> Self {
        Self {
            file_path: app_data_dir().join("settings.json"),
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn load(&self) -> PersistedSettings {
        let Ok(content) = fs::read_to_string(&self.file_path) else {
            return PersistedSettings::default();
        };
        serde_json::from_str::<PersistedSettings>(&content).unwrap_or_default()
    }

    pub fn save(&self, settings: &PersistedSettings) -> Result<()> {
        let serialized =
            serde_json::to_string_pretty(settings).context("Failed to serialize settings")?;
        fs::write(&self.file_path, serialized).context("Failed to write settings.json")
    }
}
