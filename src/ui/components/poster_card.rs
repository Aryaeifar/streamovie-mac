//! Infuse 8 + Apple TV + Netflix inspired media cards built on `uni-kit` (`gpui-kit`).
//!
//! Supports two presentation styles:
//! - [`PosterCardStyle::Poster`] (`200x300` `2:3` vertical artwork card, Infuse / Apple TV style)
//! - [`PosterCardStyle::Widescreen`] (`296x168` `16:9` landscape backdrop card, Netflix / Apple TV style)
//!
//! Both styles use [`render_remote_image`] for reliable image loading, GPUI `linear_gradient`
//! bottom vignettes, floating frosted-glass gold IMDb badges, quick-bookmark buttons, and
//! glowing brand-accent hover states.

use gpui_kit::{
    AnyElement, BoxShadow, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, black, div, hsla,
    linear_color_stop, linear_gradient, point, prelude::FluentBuilder as _, px, relative, white,
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Selectable as _, Sizable as _, StyledExt as _, WindowExt as _,
    button::Button,
    h_flex, v_flex,
};

use crate::{
    app::StreamovieApp,
    i18n::{Locale, directional_row, is_persian_text},
    models::{ApiMedia, MediaCategory},
    ui::image_cache::render_remote_image,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PosterCardStyle {
    /// Standard 2:3 vertical movie/series poster card (`200x300`, Paramount+ / cinema style).
    #[default]
    Poster,
    /// Widescreen 16:9 landscape card (`296x168`, Paramount+ featured row style).
    Widescreen,
}

pub fn render_poster_card(
    app: &StreamovieApp,
    media: &ApiMedia,
    category: MediaCategory,
    bookmarked: bool,
    locale: Locale,
    id_scope: &str,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    render_poster_card_with_style(
        app,
        media,
        category,
        bookmarked,
        locale,
        id_scope,
        PosterCardStyle::Poster,
        cx,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_poster_card_with_style(
    app: &StreamovieApp,
    media: &ApiMedia,
    category: MediaCategory,
    bookmarked: bool,
    locale: Locale,
    id_scope: &str,
    card_style: PosterCardStyle,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let media_id = media.id;
    let title = media.display_title();
    let title_is_rtl = is_persian_text(&title) || rtl;

    let poster_url = media.poster_image();
    let cover_url = media.hero_image().or_else(|| media.cover_image());
    let (primary_img, fallback_img) = match card_style {
        PosterCardStyle::Poster => (poster_url.as_deref(), cover_url.as_deref()),
        PosterCardStyle::Widescreen => (cover_url.as_deref(), poster_url.as_deref()),
    };

    let rating = media.rating_value();
    let year = media.release_year();
    let genres = media.genre_list();
    let first_genre = genres.first().cloned();

    let (card_w, frame_h) = match card_style {
        PosterCardStyle::Poster => (px(196.), px(294.)),
        PosterCardStyle::Widescreen => (px(292.), px(166.)),
    };
    let card_radius = px(14.);

    let card_id = SharedString::from(format!(
        "media-card-{id_scope}-{}-{media_id}",
        category.as_str()
    ));
    let bookmark_btn_id = SharedString::from(format!(
        "bookmark-btn-{id_scope}-{}-{media_id}",
        category.as_str()
    ));

    let media_for_open = media.clone();
    let media_for_bookmark = media.clone();

    let primary_accent = cx.theme().primary;
    let subtle_border = if cx.theme().is_dark() {
        hsla(215.0 / 360.0, 0.30, 0.85, 0.12)
    } else {
        cx.theme().border
    };

    let default_shadow = vec![BoxShadow {
        offset: point(px(0.), px(4.)),
        blur_radius: px(14.),
        spread_radius: px(0.),
        inset: false,
        color: hsla(222.0 / 360.0, 0.45, 0.02, 0.65),
    }];

    let hover_shadow = vec![
        BoxShadow {
            offset: point(px(0.), px(12.)),
            blur_radius: px(28.),
            spread_radius: px(1.5),
            inset: false,
            color: hsla(217.0 / 360.0, 0.95, 0.50, 0.40),
        },
        BoxShadow {
            offset: point(px(0.), px(6.)),
            blur_radius: px(14.),
            spread_radius: px(0.),
            inset: false,
            color: hsla(0.0, 0.0, 0.0, 0.85),
        },
    ];

    let gold_pill_bg = hsla(0.0, 0.0, 0.04, 0.82);
    let gold_pill_border = hsla(45.0 / 360.0, 0.95, 0.55, 0.45);
    let gold_text = hsla(45.0 / 360.0, 0.96, 0.64, 1.0);
    let glass_pill_bg = hsla(220.0 / 360.0, 0.35, 0.06, 0.80);
    let glass_pill_border = hsla(0.0, 0.0, 1.0, 0.16);

    let fallback_type_label = if category == MediaCategory::Series {
        dict.home.series
    } else {
        dict.home.movie
    };

    let artwork = render_remote_image(
        app,
        primary_img,
        fallback_img,
        card_radius,
        if category == MediaCategory::Series {
            IconName::Frame
        } else {
            IconName::GalleryVerticalEnd
        },
        Some(&title),
        cx,
    );

    v_flex()
        .id(card_id)
        .w(card_w)
        .flex_shrink_0()
        .pt(px(6.))
        .pb_1()
        .gap_2p5()
        .cursor_pointer()
        .group("poster-card")
        .on_click(cx.listener(move |this, _, _, cx| {
            this.open_details(media_for_open.clone(), category, cx);
        }))
        // Artwork Frame (Paramount+ rounded poster with gentle scale lift + soft glow shadow)
        .child(
            div()
                .relative()
                .flex()
                .w(card_w)
                .h(frame_h)
                .rounded(card_radius)
                .bg(cx.theme().popover)
                .border_1()
                .border_color(subtle_border)
                .shadow(default_shadow)
                .overflow_hidden()
                // Smooth hover animation: gentle scale lift + soft Paramount Blue shadow + crisp border highlight
                .group_hover("poster-card", move |style| {
                    style
                        .mt(-px(6.))
                        .mb(px(6.))
                        .border_color(primary_accent.opacity(0.95))
                        .shadow(hover_shadow.clone())
                })
                // 1. Remote Poster / Backdrop Image
                .child(artwork)
                // 2. Bottom Cinematic Gradient Vignette
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded(card_radius)
                        .bg(linear_gradient(
                            0.,
                            linear_color_stop(black().opacity(0.92), 0.0),
                            linear_color_stop(black().opacity(0.0), 0.62),
                        )),
                )
                // 3. Top Floating Bar: Gold IMDb Rating Pill + Bookmark Action
                .child(
                    directional_row(h_flex(), rtl)
                        .absolute()
                        .top_2p5()
                        .left_2p5()
                        .right_2p5()
                        .justify_between()
                        .items_center()
                        .gap_1p5()
                        .child(if rating > 0.0 {
                            h_flex()
                                .gap_1()
                                .items_center()
                                .px_2()
                                .py_0p5()
                                .rounded(px(7.))
                                .bg(gold_pill_bg)
                                .border_1()
                                .border_color(gold_pill_border)
                                .text_color(gold_text)
                                .text_xs()
                                .font_bold()
                                .child(Icon::new(IconName::StarFill).xsmall())
                                .child(format!("{rating:.1}"))
                                .into_any_element()
                        } else {
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded(px(7.))
                                .bg(glass_pill_bg)
                                .border_1()
                                .border_color(glass_pill_border)
                                .text_color(white().opacity(0.90))
                                .text_xs()
                                .font_semibold()
                                .child(fallback_type_label)
                                .into_any_element()
                        })
                        .child(
                            Button::new(bookmark_btn_id)
                                .small()
                                .compact()
                                .selected(bookmarked)
                                .icon(if bookmarked {
                                    IconName::StarFill
                                } else {
                                    IconName::BookOpen
                                })
                                .tooltip(if bookmarked {
                                    dict.bookmarks.remove
                                } else {
                                    dict.bookmarks.add
                                })
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    cx.stop_propagation();
                                    let added = this.toggle_bookmark(
                                        media_for_bookmark.clone(),
                                        category,
                                        cx,
                                    );
                                    let msg = if added {
                                        format!(
                                            "{}: {}",
                                            this.locale().dict().bookmarks.add,
                                            media_for_bookmark.display_title()
                                        )
                                    } else {
                                        format!(
                                            "{}: {}",
                                            this.locale().dict().bookmarks.remove,
                                            media_for_bookmark.display_title()
                                        )
                                    };
                                    window.push_notification(msg, cx);
                                })),
                        ),
                )
                // 4. Center Play Button overlay on hover
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .opacity(0.0)
                        .group_hover("poster-card", |s| s.opacity(1.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size_11()
                                .rounded_full()
                                .bg(primary_accent.opacity(0.92))
                                .text_color(white())
                                .border_2()
                                .border_color(white().opacity(0.40))
                                .shadow(vec![BoxShadow {
                                    offset: point(px(0.), px(4.)),
                                    blur_radius: px(16.),
                                    spread_radius: px(0.),
                                    inset: false,
                                    color: hsla(0.0, 0.0, 0.0, 0.8),
                                }])
                                .child(Icon::new(IconName::Play).small()),
                        ),
                )
                // 5. Bottom Inside Overlay: For Widescreen cards, embedded title + metadata;
                //    for Tall Poster cards, subtle genre chip
                .child(match card_style {
                    PosterCardStyle::Widescreen => v_flex()
                        .absolute()
                        .left_3()
                        .right_3()
                        .bottom_3()
                        .gap_1()
                        .when(title_is_rtl, |col| col.items_end().text_right())
                        .child(
                            div()
                                .w_full()
                                .text_base()
                                .font_bold()
                                .line_height(relative(1.2))
                                .text_color(white())
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .text_ellipsis()
                                .when(title_is_rtl, |el| el.text_right())
                                .child(title.clone()),
                        )
                        .child(
                            directional_row(h_flex(), rtl)
                                .gap_2()
                                .text_xs()
                                .text_color(white().opacity(0.78))
                                .when_some(year.clone(), |row, yr| row.child(yr))
                                .when_some(first_genre.clone(), |row, g| {
                                    row.child("·").child(g)
                                }),
                        )
                        .into_any_element(),
                    PosterCardStyle::Poster => directional_row(h_flex(), rtl)
                        .absolute()
                        .left_2p5()
                        .right_2p5()
                        .bottom_2p5()
                        .justify_between()
                        .items_center()
                        .when_some(first_genre.clone(), |row, genre| {
                            row.child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded(px(6.))
                                    .bg(glass_pill_bg)
                                    .border_1()
                                    .border_color(glass_pill_border)
                                    .text_size(px(10.))
                                    .font_medium()
                                    .text_color(white().opacity(0.92))
                                    .child(genre),
                            )
                        })
                        .into_any_element(),
                }),
        )
        // Below-Poster Caption (Paramount+ style — for Tall Poster cards)
        .when(card_style == PosterCardStyle::Poster, |card| {
            card.child(
                v_flex()
                    .w_full()
                    .px_1()
                    .gap_0p5()
                    .when(title_is_rtl, |col| col.items_end().text_right())
                    .child(
                        div()
                            .w_full()
                            .text_sm()
                            .font_bold()
                            .line_height(relative(1.28))
                            .text_color(cx.theme().foreground)
                            .group_hover("poster-card", move |s| s.text_color(primary_accent))
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .when(title_is_rtl, |el| el.text_right())
                            .child(title),
                    )
                    .child(
                        directional_row(h_flex(), rtl)
                            .w_full()
                            .justify_between()
                            .items_center()
                            .text_xs()
                            .font_medium()
                            .text_color(cx.theme().muted_foreground)
                            .child(year.unwrap_or_else(|| "—".to_string()))
                            .child(fallback_type_label),
                    ),
            )
        })
        .into_any_element()
}
