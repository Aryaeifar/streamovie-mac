//! Streamovie design tokens shared across shell, views, and media components.
#![allow(dead_code)]

use gpui_kit::{Hsla, hsla, px, Pixels};

/// Default corner radius for Streamovie chrome and media cards.
pub const RADIUS: Pixels = px(8.);

/// Slightly larger radius for hero stages and elevated surfaces.
pub const RADIUS_LG: Pixels = px(12.);

/// Deep cinema canvas (`#040711`) used for full-bleed stages.
pub fn canvas() -> Hsla {
    hsla(222.0 / 360.0, 0.52, 0.032, 1.0)
}

/// Elevated card / glass surface over the cinema canvas.
pub fn elevated_surface(is_dark: bool, popover: Hsla) -> Hsla {
    if is_dark {
        hsla(220.0 / 360.0, 0.38, 0.08, 0.92)
    } else {
        popover
    }
}

/// Quiet hairline border on dark cinema surfaces.
pub fn quiet_border(is_dark: bool, theme_border: Hsla) -> Hsla {
    if is_dark {
        hsla(216.0 / 360.0, 0.40, 0.85, 0.12)
    } else {
        theme_border
    }
}

/// Soft glass pill background for metadata chips.
pub fn glass_pill() -> Hsla {
    hsla(220.0 / 360.0, 0.35, 0.08, 0.70)
}

/// IMDb / rating gold accents.
pub fn rating_gold() -> Hsla {
    hsla(45.0 / 360.0, 0.96, 0.64, 1.0)
}

pub fn rating_gold_bg() -> Hsla {
    hsla(45.0 / 360.0, 0.95, 0.50, 0.20)
}

pub fn rating_gold_border() -> Hsla {
    hsla(45.0 / 360.0, 0.95, 0.55, 0.45)
}

/// Accent cyan used sparingly for status kickers (not Paramount branding).
pub fn status_cyan() -> Hsla {
    hsla(188.0 / 360.0, 1.0, 0.52, 1.0)
}
