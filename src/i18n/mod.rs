//! Bilingual internationalization (`en` and `fa`) and RTL layout support.
#![allow(dead_code)]

pub mod dictionary;

pub use dictionary::*;
use gpui_kit::{Div, SharedString, Styled as _, TextAlign};
use gpui_kit::component::Side;
use serde::{Deserialize, Serialize};

/// Supported application locales.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    #[default]
    En,
    Fa,
}

impl Locale {
    /// Language code (`"en"` or `"fa"`).
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Fa => "fa",
        }
    }

    /// Display label in its native script.
    pub fn label(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Fa => "فارسی",
        }
    }

    /// Short uppercase badge label (`"EN"` or `"FA"`).
    pub fn short_badge(self) -> &'static str {
        match self {
            Self::En => "EN",
            Self::Fa => "FA",
        }
    }

    /// Returns `true` when the locale uses Right-to-Left layout (`Locale::Fa`).
    pub fn is_rtl(self) -> bool {
        matches!(self, Self::Fa)
    }

    /// Toggle between English and Persian.
    pub fn toggle(self) -> Self {
        match self {
            Self::En => Self::Fa,
            Self::Fa => Self::En,
        }
    }

    /// Sidebar placement side for the current reading direction.
    pub fn sidebar_side(self) -> Side {
        if self.is_rtl() {
            Side::Right
        } else {
            Side::Left
        }
    }

    /// Text alignment for the current reading direction.
    pub fn text_align(self) -> TextAlign {
        if self.is_rtl() {
            TextAlign::Right
        } else {
            TextAlign::Left
        }
    }

    /// Returns the static dictionary for this locale.
    pub fn dict(self) -> &'static Dictionary {
        match self {
            Self::En => &EN_DICTIONARY,
            Self::Fa => &FA_DICTIONARY,
        }
    }

    /// Preferred font family for this locale on macOS.
    pub fn font_family(self) -> SharedString {
        match self {
            Self::En => ".SystemUIFont".into(),
            Self::Fa => "Geeza Pro".into(),
        }
    }
}

/// Checks whether a string contains Persian/Arabic script characters
/// (`[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF]`), matching Streamovie's `isPersianText`.
pub fn is_persian_text(value: &str) -> bool {
    value.chars().any(|ch| {
        ('\u{0600}'..='\u{06FF}').contains(&ch)
            || ('\u{0750}'..='\u{077F}').contains(&ch)
            || ('\u{08A0}'..='\u{08FF}').contains(&ch)
    })
}

/// Applies directional row layout (`flex_row_reverse` when RTL, `flex_row` when LTR)
/// so horizontal stacks mirror naturally in Persian mode.
pub fn directional_row(div: Div, rtl: bool) -> Div {
    let base = div.flex().items_center();
    if rtl {
        base.flex_row_reverse()
    } else {
        base.flex_row()
    }
}

/// Formats numbers using Persian digits (`۰-۹`) when `Locale::Fa` is active.
pub fn format_number(value: usize, locale: Locale) -> String {
    let raw = value.to_string();
    if !locale.is_rtl() {
        return raw;
    }
    raw.chars()
        .map(|c| match c {
            '0' => '۰',
            '1' => '۱',
            '2' => '۲',
            '3' => '۳',
            '4' => '۴',
            '5' => '۵',
            '6' => '۶',
            '7' => '۷',
            '8' => '۸',
            '9' => '۹',
            other => other,
        })
        .collect()
}

/// Replaces `{count}` placeholder in a localized template with a locale-formatted number.
pub fn format_count(template: &str, count: usize, locale: Locale) -> String {
    template.replace("{count}", &format_number(count, locale))
}
