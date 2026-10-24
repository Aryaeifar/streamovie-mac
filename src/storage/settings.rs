//! Persistent JSON storage for user preferences (`settings.json`), including
//! theme mode (`Dark`, `Light`, `System`), Streamovie brand accent palette, and `Locale`.
//! Applies Linear + Netflix + Infuse cinema surface tokens in Dark mode while
//! preserving crisp Light and macOS System appearance support.
#![allow(dead_code)]

use std::{fs, path::PathBuf};

use anyhow::{Context as _, Result};
use gpui_kit::{App, Hsla, Window, hsla, px};
use gpui_kit::component::{Theme, ThemeMode};
use serde::{Deserialize, Serialize};

use crate::{
    i18n::{Dictionary, Locale},
    storage::app_data_dir,
};

/// Theme mode preference supporting Dark (default), Light, and macOS System appearance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeModePreference {
    #[default]
    Dark,
    Light,
    System,
}

impl ThemeModePreference {
    pub const ALL: [Self; 3] = [Self::Dark, Self::Light, Self::System];

    pub fn code(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::System => "system",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "light" => Self::Light,
            "system" => Self::System,
            _ => Self::Dark,
        }
    }

    pub fn label(self, dict: &Dictionary) -> &'static str {
        match self {
            Self::Dark => dict.settings.mode_dark,
            Self::Light => dict.settings.mode_light,
            Self::System => dict.settings.mode_system,
        }
    }
}

/// Streamovie brand accent palettes ported from `Streamovie/src/lib/themes.ts`
/// and `Streamovie/src/app/globals.css`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccentPalette {
    #[default]
    #[serde(alias = "paramount")]
    Streamovie,
    Netflix,
    Midnight,
    Aurora,
    Ocean,
    Sunset,
    Candy,
    Noir,
}

impl AccentPalette {
    pub const ALL: [Self; 8] = [
        Self::Streamovie,
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
            Self::Streamovie => "streamovie",
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
            // Migrate persisted "paramount" accent from older installs.
            "streamovie" | "paramount" => Self::Streamovie,
            "netflix" => Self::Netflix,
            "midnight" => Self::Midnight,
            "aurora" => Self::Aurora,
            "ocean" => Self::Ocean,
            "sunset" => Self::Sunset,
            "candy" => Self::Candy,
            "noir" => Self::Noir,
            _ => Self::Streamovie,
        }
    }

    pub fn label(self, dict: &Dictionary) -> &'static str {
        match self {
            Self::Streamovie => dict.settings.themes.streamovie,
            Self::Netflix => dict.settings.themes.netflix,
            Self::Midnight => dict.settings.themes.midnight,
            Self::Aurora => dict.settings.themes.aurora,
            Self::Ocean => dict.settings.themes.ocean,
            Self::Sunset => dict.settings.themes.sunset,
            Self::Candy => dict.settings.themes.candy,
            Self::Noir => dict.settings.themes.noir,
        }
    }

    /// Primary accent color in HSLA matching Streamovie cinema blue (`#0064FF`).
    pub fn primary_hsla(self) -> Hsla {
        match self {
            Self::Streamovie => hsla(216.0 / 360.0, 1.0, 0.50, 1.0), // #0064ff Streamovie cinema blue
            Self::Netflix => hsla(357.0 / 360.0, 0.92, 0.47, 1.0),  // #e50914
            Self::Midnight => hsla(258.0 / 360.0, 0.90, 0.66, 1.0), // #8b5cf6
            Self::Aurora => hsla(160.0 / 360.0, 0.84, 0.39, 1.0),   // #10b981
            Self::Ocean => hsla(199.0 / 360.0, 0.89, 0.48, 1.0),    // #0ea5e9
            Self::Sunset => hsla(25.0 / 360.0, 0.95, 0.53, 1.0),    // #f97316
            Self::Candy => hsla(330.0 / 360.0, 0.81, 0.60, 1.0),    // #ec4899
            Self::Noir => hsla(240.0 / 360.0, 0.05, 0.65, 1.0),     // #a1a1aa
        }
    }

    /// Deep cinema dark canvas & elevated surface tokens for each palette (`globals.css`).
    pub fn dark_surfaces(self) -> (Hsla, Hsla, Hsla) {
        match self {
            // (canvas_bg, card_surface, sidebar_bg)
            Self::Streamovie => (
                hsla(222.0 / 360.0, 0.52, 0.032, 1.0), // #040711 deep cinema canvas
                hsla(220.0 / 360.0, 0.38, 0.075, 1.0), // #0a1120 card surface
                hsla(224.0 / 360.0, 0.58, 0.026, 1.0), // #03050c quiet sidebar
            ),
            Self::Netflix => (
                hsla(222.0 / 360.0, 0.42, 0.038, 1.0),
                hsla(220.0 / 360.0, 0.32, 0.082, 1.0),
                hsla(224.0 / 360.0, 0.38, 0.045, 1.0),
            ),
            Self::Midnight => (
                hsla(240.0 / 360.0, 0.29, 0.035, 1.0), // #06060b
                hsla(240.0 / 360.0, 0.22, 0.082, 1.0), // #10101a
                hsla(240.0 / 360.0, 0.26, 0.048, 1.0), // #090910
            ),
            Self::Aurora => (
                hsla(190.0 / 360.0, 0.43, 0.028, 1.0), // #04090a
                hsla(190.0 / 360.0, 0.30, 0.075, 1.0), // #0d181a
                hsla(190.0 / 360.0, 0.38, 0.042, 1.0), // #070f10
            ),
            Self::Ocean => (
                hsla(222.0 / 360.0, 0.56, 0.037, 1.0), // #04070f
                hsla(222.0 / 360.0, 0.42, 0.086, 1.0), // #0d1320
                hsla(222.0 / 360.0, 0.50, 0.050, 1.0), // #070b14
            ),
            Self::Sunset => (
                hsla(24.0 / 360.0, 0.45, 0.030, 1.0),  // #0b0704
                hsla(24.0 / 360.0, 0.35, 0.078, 1.0),  // #1b110b
                hsla(24.0 / 360.0, 0.40, 0.044, 1.0),  // #100a06
            ),
            Self::Candy => (
                hsla(292.0 / 360.0, 0.40, 0.034, 1.0), // #0b050c
                hsla(292.0 / 360.0, 0.32, 0.082, 1.0), // #1b0e1c
                hsla(292.0 / 360.0, 0.36, 0.048, 1.0), // #110812
            ),
            Self::Noir => (
                hsla(220.0 / 360.0, 0.15, 0.038, 1.0), // #0a0b0e
                hsla(220.0 / 360.0, 0.12, 0.086, 1.0), // #14151a
                hsla(220.0 / 360.0, 0.14, 0.052, 1.0), // #0c0e12
            ),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedSettings {
    #[serde(default)]
    pub locale: Locale,
    #[serde(default)]
    pub theme_mode: ThemeModePreference,
    #[serde(default)]
    pub accent_palette: AccentPalette,
}

impl PersistedSettings {
    /// Applies the configured theme mode (`Dark`, `Light`, `System`), brand accent
    /// color, cinema surface hierarchy, and locale font family to `uni-kit`'s global `Theme`.
    pub fn apply_to_uni_kit(&self, window: Option<&mut Window>, cx: &mut App) {
        match self.theme_mode {
            ThemeModePreference::Dark => Theme::change(ThemeMode::Dark, window, cx),
            ThemeModePreference::Light => Theme::change(ThemeMode::Light, window, cx),
            ThemeModePreference::System => Theme::sync_system_appearance(window, cx),
        }

        let primary = self.accent_palette.primary_hsla();
        let (dark_bg, dark_surface, dark_sidebar) = self.accent_palette.dark_surfaces();
        let font_family = self.locale.font_family();

        Theme::update(cx, |theme| {
            theme.primary = primary;
            theme.primary_foreground = hsla(0.0, 0.0, 1.0, 1.0);
            theme.ring = primary;
            theme.sidebar_primary = primary;
            theme.radius = px(8.);
            theme.radius_lg = px(12.);
            theme.font_family = font_family;

            if theme.is_dark() {
                theme.background = dark_bg;
                theme.foreground = hsla(215.0 / 360.0, 0.20, 0.98, 1.0);
                theme.popover = dark_surface;
                theme.popover_foreground = hsla(215.0 / 360.0, 0.20, 0.98, 1.0);
                theme.sidebar = dark_sidebar;
                theme.sidebar_foreground = hsla(215.0 / 360.0, 0.15, 0.92, 1.0);
                theme.sidebar_border = hsla(216.0 / 360.0, 0.40, 0.85, 0.08);
                theme.muted = hsla(220.0 / 360.0, 0.35, 0.11, 1.0);
                theme.muted_foreground = hsla(215.0 / 360.0, 0.18, 0.65, 1.0);
                theme.border = hsla(216.0 / 360.0, 0.40, 0.85, 0.09);
                theme.input = hsla(220.0 / 360.0, 0.35, 0.08, 0.85);
            }
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
