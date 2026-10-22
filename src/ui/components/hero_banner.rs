//! Infuse 8 + Netflix + Apple TV inspired full-bleed Spotlight Hero stage.
//!
//! Features:
//! - `450px` tall widescreen stage with rounded `24px` corners and `1px` subtle inner border
//! - Full-bleed high-resolution backdrop loaded via [`render_remote_image`] (with automatic
//!   fallback to the catalog's own `hero_image` / `poster_image`)
//! - Dual GPUI `linear_gradient` directional + bottom vignette scrims
//! - Oversized `40px` display title, gold IMDb score badge, year & genre pills, and synopsis
//! - Primary `Watch Now` CTA + `View Details` + `Watchlist` + `Like` actions
//! - Interactive widescreen **Spotlight Thumbnail Strip** (`120x68`) with live preview images

use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, black, div, hsla, linear_color_stop,
    linear_gradient, prelude::FluentBuilder as _, px, relative, white,
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Selectable as _, Sizable as _, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};

use crate::{
    app::StreamovieApp,
    i18n::{Locale, directional_row, is_persian_text},
    models::HeroSpotlightItem,
    ui::image_cache::render_remote_image,
};

pub fn render_hero_banner(
    app: &StreamovieApp,
    items: &[HeroSpotlightItem],
    active_index: usize,
    locale: Locale,
    bookmarked: bool,
    liked: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = locale.dict();
    let rtl = locale.is_rtl();

    let Some(item) = items.get(active_index).or_else(|| items.first()) else {
        return div().into_any_element();
    };

    let media = &item.media;
    let category = item.category;
    let media_id = media.id;

    let title = if rtl {
        item.title_fa
            .clone()
            .unwrap_or_else(|| media.display_title())
    } else {
        media.display_title()
    };

    let overview = if rtl {
        item.description_fa
            .clone()
            .or_else(|| media.parsed_description().overview)
    } else {
        media
            .parsed_description()
            .overview
            .or_else(|| item.description_fa.clone())
    };

    let primary_backdrop = item.artwork.clone().or_else(|| media.hero_image());
    let fallback_backdrop = media
        .hero_image()
        .or_else(|| media.cover_image())
        .or_else(|| media.poster_image());

    let rating = media.rating_value();
    let year = media.release_year();
    let genres: Vec<String> = media.genre_list().into_iter().take(3).collect();

    let media_for_watch = media.clone();
    let media_for_details = media.clone();
    let media_for_bookmark = media.clone();
    let title_rtl = is_persian_text(&title) || rtl;
    let overview_rtl = overview.as_deref().map(is_persian_text).unwrap_or(rtl);

    let hero_radius = px(24.);
    let primary_accent = cx.theme().primary;
    let ink_dark = hsla(240.0 / 360.0, 0.25, 0.035, 1.0);
    let gold_text = hsla(45.0 / 360.0, 0.96, 0.64, 1.0);
    let gold_bg = hsla(45.0 / 360.0, 0.95, 0.50, 0.18);
    let gold_border = hsla(45.0 / 360.0, 0.95, 0.55, 0.45);
    let glass_bg = hsla(0.0, 0.0, 1.0, 0.10);
    let glass_border = hsla(0.0, 0.0, 1.0, 0.18);

    let backdrop_element = render_remote_image(
        app,
        primary_backdrop.as_deref(),
        fallback_backdrop.as_deref(),
        hero_radius,
        IconName::Frame,
        Some(&title),
        cx,
    );

    // Build interactive widescreen Spotlight Thumbnail Strip (Infuse / Netflix style)
    let mut thumb_cards = Vec::with_capacity(items.len());
    for (ix, entry) in items.iter().take(6).enumerate() {
        let is_active = ix == active_index;
        let thumb_label = if rtl {
            entry
                .title_fa
                .clone()
                .unwrap_or_else(|| entry.media.display_title())
        } else {
            entry.media.display_title()
        };
        let thumb_primary = entry
            .media
            .cover_image()
            .or_else(|| entry.media.poster_image())
            .or_else(|| entry.artwork.clone());
        let thumb_fallback = entry.media.poster_image().or_else(|| entry.artwork.clone());
        let thumb_img = render_remote_image(
            app,
            thumb_primary.as_deref(),
            thumb_fallback.as_deref(),
            px(10.),
            IconName::Play,
            None,
            cx,
        );

        let thumb_id = SharedString::from(format!("hero-thumb-card-{ix}"));
        thumb_cards.push(
            v_flex()
                .id(thumb_id)
                .gap_1p5()
                .items_center()
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_spotlight_index(ix, cx);
                }))
                .child(
                    div()
                        .relative()
                        .flex()
                        .w(px(118.))
                        .h(px(66.))
                        .rounded(px(10.))
                        .overflow_hidden()
                        .border_2()
                        .border_color(if is_active {
                            primary_accent
                        } else {
                            hsla(0.0, 0.0, 1.0, 0.16)
                        })
                        .when(!is_active, |el| el.opacity(0.68))
                        .hover(|s| s.opacity(1.0).border_color(white().opacity(0.55)))
                        .child(thumb_img)
                        .child(
                            div()
                                .absolute()
                                .inset_0()
                                .bg(linear_gradient(
                                    0.,
                                    linear_color_stop(black().opacity(0.85), 0.0),
                                    linear_color_stop(black().opacity(0.05), 0.75),
                                )),
                        )
                        .child(
                            div()
                                .absolute()
                                .left_2()
                                .right_2()
                                .bottom_1p5()
                                .text_size(px(11.))
                                .font_semibold()
                                .text_color(white())
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(thumb_label),
                        ),
                )
                .child(
                    div()
                        .w(if is_active { px(36.) } else { px(12.) })
                        .h(px(3.))
                        .rounded(px(99.))
                        .bg(if is_active {
                            primary_accent
                        } else {
                            hsla(0.0, 0.0, 1.0, 0.18)
                        }),
                ),
        );
    }

    div()
        .relative()
        .flex()
        .w_full()
        .h(px(450.))
        .rounded(hero_radius)
        .border_1()
        .border_color(hsla(0.0, 0.0, 1.0, 0.12))
        .bg(ink_dark)
        .shadow_xl()
        .overflow_hidden()
        // 1. Full-bleed Backdrop Image
        .child(backdrop_element)
        // 2. Horizontal Directional Gradient Scrim (90deg LTR / 270deg RTL)
        .child(
            div()
                .absolute()
                .inset_0()
                .rounded(hero_radius)
                .bg(linear_gradient(
                    if rtl { 270. } else { 90. },
                    linear_color_stop(ink_dark.opacity(0.96), 0.0),
                    linear_color_stop(ink_dark.opacity(0.18), 0.85),
                )),
        )
        // 3. Bottom-to-Top Vignette Gradient Scrim
        .child(
            div()
                .absolute()
                .inset_0()
                .rounded(hero_radius)
                .bg(linear_gradient(
                    0.,
                    linear_color_stop(ink_dark.opacity(0.97), 0.0),
                    linear_color_stop(ink_dark.opacity(0.05), 0.65),
                )),
        )
        // 4. Foreground Content Layer
        .child(
            v_flex()
                .relative()
                .size_full()
                .p_8()
                .justify_between()
                .when(rtl, |col| col.items_end())
                // Upper / Middle Hero Copy & Actions
                .child(
                    v_flex()
                        .max_w(px(680.))
                        .gap_3p5()
                        .when(rtl, |col| col.items_end().text_right())
                        // Eyebrow Row: Featured Pill + Gold IMDb + Year + Genres
                        .child(
                            directional_row(h_flex(), rtl)
                                .flex_wrap()
                                .gap_2()
                                .child(
                                    directional_row(h_flex(), rtl)
                                        .gap_1p5()
                                        .px_3()
                                        .py_1()
                                        .rounded(px(99.))
                                        .bg(primary_accent.opacity(0.22))
                                        .border_1()
                                        .border_color(primary_accent.opacity(0.55))
                                        .text_xs()
                                        .font_bold()
                                        .text_color(white())
                                        .child(
                                            div()
                                                .size(px(6.))
                                                .rounded(px(99.))
                                                .bg(primary_accent),
                                        )
                                        .child(dict.home.featured),
                                )
                                .when(rating > 0.0, |row| {
                                    row.child(
                                        h_flex()
                                            .gap_1()
                                            .items_center()
                                            .px_2p5()
                                            .py_1()
                                            .rounded(px(99.))
                                            .bg(gold_bg)
                                            .border_1()
                                            .border_color(gold_border)
                                            .text_xs()
                                            .font_bold()
                                            .text_color(gold_text)
                                            .child(Icon::new(IconName::StarFill).xsmall())
                                            .child(format!("{rating:.1} IMDb")),
                                    )
                                })
                                .when_some(year, |row, yr| {
                                    row.child(
                                        div()
                                            .px_2p5()
                                            .py_1()
                                            .rounded(px(99.))
                                            .bg(glass_bg)
                                            .border_1()
                                            .border_color(glass_border)
                                            .text_xs()
                                            .font_semibold()
                                            .text_color(white().opacity(0.90))
                                            .child(yr),
                                    )
                                })
                                .children(genres.into_iter().map(move |g| {
                                    div()
                                        .px_2p5()
                                        .py_1()
                                        .rounded(px(99.))
                                        .bg(glass_bg)
                                        .border_1()
                                        .border_color(glass_border)
                                        .text_xs()
                                        .text_color(white().opacity(0.85))
                                        .child(g)
                                })),
                        )
                        // Large Cinematic Display Title
                        .child(
                            div()
                                .text_size(px(40.))
                                .font_bold()
                                .line_height(relative(1.08))
                                .text_color(white())
                                .when(title_rtl, |el| el.text_right())
                                .child(title),
                        )
                        // Synopsis
                        .when_some(overview, |col, text| {
                            col.child(
                                div()
                                    .max_w(px(620.))
                                    .text_sm()
                                    .line_height(relative(1.6))
                                    .text_color(white().opacity(0.84))
                                    .when(overview_rtl, |el| el.text_right())
                                    .child(text),
                            )
                        })
                        // Primary & Secondary CTA Buttons
                        .child(
                            directional_row(h_flex(), rtl)
                                .pt_1()
                                .gap_3()
                                .child(
                                    Button::new("hero-watch-now")
                                        .primary()
                                        .large()
                                        .icon(IconName::Play)
                                        .label(dict.actions.watch_now)
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.open_details(
                                                media_for_watch.clone(),
                                                category,
                                                cx,
                                            );
                                        })),
                                )
                                .child(
                                    Button::new("hero-details-btn")
                                        .large()
                                        .outline()
                                        .icon(IconName::Info)
                                        .label(dict.actions.details)
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.open_details(
                                                media_for_details.clone(),
                                                category,
                                                cx,
                                            );
                                        })),
                                )
                                .child(
                                    Button::new("hero-bookmark-btn")
                                        .large()
                                        .outline()
                                        .selected(bookmarked)
                                        .icon(if bookmarked {
                                            IconName::StarFill
                                        } else {
                                            IconName::BookOpen
                                        })
                                        .label(if bookmarked {
                                            dict.bookmarks.remove
                                        } else {
                                            dict.bookmarks.add
                                        })
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            let added = this.toggle_bookmark(
                                                media_for_bookmark.clone(),
                                                category,
                                                cx,
                                            );
                                            let notice = if added {
                                                this.locale().dict().bookmarks.add
                                            } else {
                                                this.locale().dict().bookmarks.remove
                                            };
                                            window.push_notification(notice, cx);
                                        })),
                                )
                                .child(
                                    Button::new("hero-like-btn")
                                        .large()
                                        .ghost()
                                        .selected(liked)
                                        .icon(IconName::Heart)
                                        .tooltip(if liked {
                                            dict.actions.unlike
                                        } else {
                                            dict.actions.like
                                        })
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.toggle_like(media_id, cx);
                                        })),
                                ),
                        ),
                )
                // Bottom Strip: Interactive Spotlight Thumbnail Cards + Prev/Next Controls
                .child(
                    directional_row(h_flex(), rtl)
                        .w_full()
                        .justify_between()
                        .items_end()
                        .gap_4()
                        .child(
                            directional_row(h_flex(), rtl)
                                .gap_2p5()
                                .flex_wrap()
                                .children(thumb_cards),
                        )
                        .child(
                            directional_row(h_flex(), rtl)
                                .gap_2()
                                .child(
                                    Button::new("hero-prev")
                                        .small()
                                        .outline()
                                        .icon(if rtl {
                                            IconName::ChevronRight
                                        } else {
                                            IconName::ChevronLeft
                                        })
                                        .tooltip(dict.actions.previous)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.step_spotlight(-1, cx);
                                        })),
                                )
                                .child(
                                    Button::new("hero-next")
                                        .small()
                                        .outline()
                                        .icon(if rtl {
                                            IconName::ChevronLeft
                                        } else {
                                            IconName::ChevronRight
                                        })
                                        .tooltip(dict.actions.next)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.step_spotlight(1, cx);
                                        })),
                                ),
                        ),
                ),
        )
        .into_any_element()
}
