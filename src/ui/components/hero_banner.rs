//! Paramount+ signature widescreen Spotlight Hero stage.
//!
//! Visual language matches Paramount+ desktop & TV experience:
//! - Full-bleed cinematic backdrop with dual feathered gradient scrims (directional left-to-right
//!   + bottom vignette) so character art on the right stays vibrant while text on the left is razor-sharp
//! - Top-right "Paramount+ ORIGINAL" watermark emblem
//! - Neon cyan status kicker ("NOW STREAMING" / "ALL EPISODES NOW STREAMING")
//! - High-impact display typography for title
//! - Clean metadata row: GENRE · SEASONS · YEAR · TV-MA · 4K UHD · ★ IMDb score
//! - Primary electric Paramount Blue ("#0064FF") CTA button ("▶ WATCH NOW") + Watchlist "+" button + Details button
//! - Interactive widescreen spotlight preview strip with luminous Paramount Blue active highlight and smooth navigation

use gpui_kit::{
    AnyElement, BoxShadow, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, black, div, hsla,
    linear_color_stop, linear_gradient, point, prelude::FluentBuilder as _, px, relative, white,
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
    let duration = media.duration_label();
    let classification = media.classification_label().unwrap_or_else(|| "TV-MA".to_string());

    let media_for_watch = media.clone();
    let media_for_details = media.clone();
    let media_for_bookmark = media.clone();
    let title_rtl = is_persian_text(&title) || rtl;
    let overview_rtl = overview.as_deref().map(is_persian_text).unwrap_or(rtl);

    let hero_radius = px(14.);
    let primary_accent = cx.theme().primary; // #0064FF Paramount Blue
    let ink_dark = hsla(222.0 / 360.0, 0.52, 0.032, 1.0); // #040711
    let cyan_kicker = hsla(188.0 / 360.0, 1.0, 0.52, 1.0); // #00E5FF Paramount cyan
    let gold_text = hsla(45.0 / 360.0, 0.96, 0.64, 1.0);
    let gold_bg = hsla(45.0 / 360.0, 0.95, 0.50, 0.20);
    let gold_border = hsla(45.0 / 360.0, 0.95, 0.55, 0.45);
    let glass_pill_bg = hsla(220.0 / 360.0, 0.35, 0.08, 0.70);

    let backdrop_element = render_remote_image(
        app,
        primary_backdrop.as_deref(),
        fallback_backdrop.as_deref(),
        hero_radius,
        IconName::Frame,
        Some(&title),
        cx,
    );

    // Build interactive Paramount+ Spotlight Thumbnail Strip
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
            px(8.),
            IconName::Play,
            None,
            cx,
        );

        let thumb_id = SharedString::from(format!("paramount-hero-thumb-{ix}"));
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
                        .w(px(128.))
                        .h(px(72.))
                        .rounded(px(8.))
                        .overflow_hidden()
                        .border_2()
                        .border_color(if is_active {
                            primary_accent
                        } else {
                            hsla(0.0, 0.0, 1.0, 0.14)
                        })
                        .shadow(if is_active {
                            vec![BoxShadow {
                                offset: point(px(0.), px(4.)),
                                blur_radius: px(14.),
                                spread_radius: px(1.),
                                inset: false,
                                color: primary_accent.opacity(0.60),
                            }]
                        } else {
                            vec![]
                        })
                        .when(!is_active, |el| el.opacity(0.68))
                        .hover(|s| s.opacity(1.0).border_color(white().opacity(0.70)))
                        .child(thumb_img)
                        .child(
                            div()
                                .absolute()
                                .inset_0()
                                .bg(linear_gradient(
                                    0.,
                                    linear_color_stop(black().opacity(0.85), 0.0),
                                    linear_color_stop(black().opacity(0.0), 0.70),
                                )),
                        )
                        .child(
                            div()
                                .absolute()
                                .left_2()
                                .right_2()
                                .bottom_1p5()
                                .text_size(px(11.))
                                .font_bold()
                                .text_color(white())
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(thumb_label),
                        ),
                )
                .child(
                    div()
                        .w(if is_active { px(36.) } else { px(10.) })
                        .h(px(3.))
                        .rounded_full()
                        .bg(if is_active {
                            primary_accent
                        } else {
                            hsla(0.0, 0.0, 1.0, 0.20)
                        }),
                ),
        );
    }

    div()
        .relative()
        .flex()
        .w_full()
        .h(px(520.))
        .rounded(hero_radius)
        .border_1()
        .border_color(hsla(216.0 / 360.0, 0.40, 0.85, 0.12))
        .bg(ink_dark)
        .shadow_2xl()
        .overflow_hidden()
        // 1. Full-bleed Backdrop Image
        .child(backdrop_element)
        // 2. Horizontal Directional Gradient Scrim (90deg LTR / 270deg RTL)
        // Strong contrast on left for 100% text readability, feathered on right to showcase artwork
        .child(
            div()
                .absolute()
                .inset_0()
                .rounded(hero_radius)
                .bg(linear_gradient(
                    if rtl { 270. } else { 90. },
                    linear_color_stop(ink_dark.opacity(0.98), 0.0),
                    linear_color_stop(ink_dark.opacity(0.08), 0.90),
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
                    linear_color_stop(ink_dark.opacity(0.98), 0.0),
                    linear_color_stop(ink_dark.opacity(0.0), 0.70),
                )),
        )
        // 4. Paramount+ Top-Right Signature Watermark Logo
        .child(
            directional_row(h_flex(), rtl)
                .absolute()
                .top_6()
                .when(!rtl, |el| el.right_8())
                .when(rtl, |el| el.left_8())
                .items_center()
                .gap_2()
                .px_3()
                .py_1p5()
                .rounded(px(6.))
                .bg(hsla(222.0 / 360.0, 0.50, 0.05, 0.65))
                .border_1()
                .border_color(hsla(0.0, 0.0, 1.0, 0.15))
                .child(
                    div()
                        .text_size(px(13.))
                        .font_extrabold()
                        .text_color(white())
                        .child("Paramount+"),
                )
                .child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded(px(3.))
                        .bg(primary_accent)
                        .text_size(px(9.5))
                        .font_bold()
                        .text_color(white())
                        .child("ORIGINAL"),
                ),
        )
        // 5. Foreground Content Layer
        .child(
            v_flex()
                .relative()
                .size_full()
                .p_8()
                .justify_between()
                .when(rtl, |col| col.items_end())
                // Upper / Middle Paramount+ Hero Copy & Actions
                .child(
                    v_flex()
                        .max_w(px(720.))
                        .gap_3p5()
                        .when(rtl, |col| col.items_end().text_right())
                        // Status Kicker (Paramount+ Neon Cyan: "ALL EPISODES NOW STREAMING")
                        .child(
                            directional_row(h_flex(), rtl)
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .size(px(6.))
                                        .rounded_full()
                                        .bg(cyan_kicker),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_extrabold()
                                        .text_color(cyan_kicker)
                                        .child("ALL EPISODES NOW STREAMING"),
                                ),
                        )
                        // Giant Display Title (Paramount+ bold cinema typography)
                        .child(
                            div()
                                .text_size(px(46.))
                                .font_extrabold()
                                .line_height(relative(1.05))
                                .text_color(white())
                                .when(title_rtl, |el| el.text_right())
                                .child(title),
                        )
                        // Paramount+ Metadata Row: GENRE · SEASONS · YEAR · TV-MA · 4K UHD · ★ IMDb
                        .child(
                            directional_row(h_flex(), rtl)
                                .flex_wrap()
                                .items_center()
                                .gap_2()
                                .text_xs()
                                .font_semibold()
                                .text_color(hsla(215.0 / 360.0, 0.20, 0.82, 1.0))
                                // Genre tag
                                .when(!genres.is_empty(), |row| {
                                    row.child(
                                        div()
                                            .font_bold()
                                            .text_color(white())
                                            .child(genres[0].to_uppercase()),
                                    )
                                })
                                // Year
                                .when_some(year, |row, yr| {
                                    row.child(div().child("·")).child(div().child(yr))
                                })
                                // Classification
                                .child(div().child("·"))
                                .child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded(px(4.))
                                        .border_1()
                                        .border_color(hsla(0.0, 0.0, 1.0, 0.25))
                                        .bg(glass_pill_bg)
                                        .text_size(px(10.))
                                        .font_bold()
                                        .text_color(white())
                                        .child(classification),
                                )
                                // Duration / Category
                                .when_some(duration, |row, d| {
                                    row.child(div().child("·")).child(div().child(d))
                                })
                                // 4K UHD Badge
                                .child(div().child("·"))
                                .child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded(px(4.))
                                        .border_1()
                                        .border_color(hsla(0.0, 0.0, 1.0, 0.25))
                                        .bg(glass_pill_bg)
                                        .text_size(px(10.))
                                        .font_bold()
                                        .text_color(white())
                                        .child("4K UHD"),
                                )
                                // Gold IMDb rating
                                .when(rating > 0.0, |row| {
                                    row.child(div().child("·")).child(
                                        h_flex()
                                            .gap_1()
                                            .items_center()
                                            .px_2()
                                            .py_0p5()
                                            .rounded(px(4.))
                                            .bg(gold_bg)
                                            .border_1()
                                            .border_color(gold_border)
                                            .text_xs()
                                            .font_bold()
                                            .text_color(gold_text)
                                            .child(Icon::new(IconName::StarFill).xsmall())
                                            .child(format!("{rating:.1}")),
                                    )
                                }),
                        )
                        // Synopsis / Plot Description
                        .when_some(overview, |col, text| {
                            col.child(
                                div()
                                    .max_w(px(640.))
                                    .text_sm()
                                    .line_height(relative(1.6))
                                    .text_color(hsla(0.0, 0.0, 0.90, 1.0))
                                    .when(overview_rtl, |el| el.text_right())
                                    .child(text),
                            )
                        })
                        // Primary CTA Buttons (Paramount+ solid blue watch button + glass + watchlist)
                        .child(
                            directional_row(h_flex(), rtl)
                                .pt_2()
                                .items_center()
                                .gap_3()
                                // Paramount+ Primary Watch Now CTA
                                .child(
                                    Button::new("paramount-watch-now")
                                        .primary()
                                        .large()
                                        .icon(IconName::Play)
                                        .label(dict.actions.watch_now.to_uppercase())
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.open_details(
                                                media_for_watch.clone(),
                                                category,
                                                cx,
                                            );
                                        })),
                                )
                                // Watchlist "+" Button
                                .child(
                                    Button::new("paramount-watchlist-btn")
                                        .large()
                                        .outline()
                                        .selected(bookmarked)
                                        .icon(if bookmarked {
                                            IconName::Check
                                        } else {
                                            IconName::Plus
                                        })
                                        .label(if bookmarked {
                                            dict.bookmarks.remove.to_uppercase()
                                        } else {
                                            dict.bookmarks.add.to_uppercase()
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
                                // More Details Button
                                .child(
                                    Button::new("paramount-details-btn")
                                        .large()
                                        .outline()
                                        .icon(IconName::Info)
                                        .label(dict.actions.details.to_uppercase())
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.open_details(
                                                media_for_details.clone(),
                                                category,
                                                cx,
                                            );
                                        })),
                                )
                                // Like Button
                                .child(
                                    Button::new("paramount-like-btn")
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
                // Bottom Strip: Paramount+ Spotlight Carousel Thumbnails + Arrows
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
                                    Button::new("paramount-hero-prev")
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
                                    Button::new("paramount-hero-next")
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
