//! Streamovie media cards built on `uni-kit` (`gpui-kit`).
//!
//! Distinct poster (2:3) and landscape (16:9) styles, quiet hover focus,
//! bookmark action, and open-details — no third-party watermarks.
//! - Below-poster high-contrast title and metadata caption

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
    /// Standard 2:3 vertical movie/series poster card (`192x288`).
    #[default]
    Poster,
    /// Widescreen 16:9 landscape card (`288x162`).
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
        PosterCardStyle::Poster => (px(192.), px(288.)),
        PosterCardStyle::Widescreen => (px(288.), px(162.)),
    };
    let card_radius = crate::ui::theme_tokens::RADIUS;

    let card_id = SharedString::from(format!(
        "streamovie-card-{id_scope}-{}-{media_id}",
        category.as_str()
    ));
    let bookmark_btn_id = SharedString::from(format!(
        "streamovie-bm-btn-{id_scope}-{}-{media_id}",
        category.as_str()
    ));

    let media_for_open = media.clone();
    let media_for_bookmark = media.clone();

    let primary_accent = cx.theme().primary;
    let card_bg = hsla(220.0 / 360.0, 0.38, 0.075, 1.0);
    let subtle_border = hsla(216.0 / 360.0, 0.35, 0.85, 0.10);

    let default_shadow = vec![BoxShadow {
        offset: point(px(0.), px(4.)),
        blur_radius: px(12.),
        spread_radius: px(0.),
        inset: false,
        color: hsla(222.0 / 360.0, 0.50, 0.02, 0.70),
    }];

    // Quiet focus: border accent without heavy glow stack
    let hover_shadow = vec![BoxShadow {
        offset: point(px(0.), px(8.)),
        blur_radius: px(18.),
        spread_radius: px(0.),
        inset: false,
        color: hsla(0.0, 0.0, 0.0, 0.55),
    }];

    let gold_pill_bg = hsla(222.0 / 360.0, 0.50, 0.05, 0.85);
    let gold_pill_border = hsla(45.0 / 360.0, 0.95, 0.55, 0.40);
    let gold_text = hsla(45.0 / 360.0, 0.96, 0.64, 1.0);

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
        .gap_2()
        .cursor_pointer()
        .group("streamovie-card")
        .on_click(cx.listener(move |this, _, _, cx| {
            this.open_details(media_for_open.clone(), category, cx);
        }))
        .child(
            div()
                .relative()
                .flex()
                .w(card_w)
                .h(frame_h)
                .rounded(card_radius)
                .bg(card_bg)
                .border_1()
                .border_color(subtle_border)
                .shadow(default_shadow)
                .overflow_hidden()
                .group_hover("streamovie-card", move |style| {
                    style
                        .mt(-px(4.))
                        .mb(px(4.))
                        .border_color(primary_accent)
                        .shadow(hover_shadow.clone())
                })
                .child(artwork)
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded(card_radius)
                        .bg(linear_gradient(
                            0.,
                            linear_color_stop(black().opacity(0.88), 0.0),
                            linear_color_stop(black().opacity(0.0), 0.65),
                        )),
                )
                .child(
                    directional_row(h_flex(), rtl)
                        .absolute()
                        .top_2()
                        .left_2()
                        .right_2()
                        .justify_between()
                        .items_center()
                        .gap_1p5()
                        .child(if category == MediaCategory::Series {
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded(px(4.))
                                .bg(primary_accent)
                                .text_size(px(9.5))
                                .font_bold()
                                .text_color(white())
                                .child("SERIES")
                                .into_any_element()
                        } else if rating > 0.0 {
                            h_flex()
                                .gap_1()
                                .items_center()
                                .px_1p5()
                                .py_0p5()
                                .rounded(px(4.))
                                .bg(gold_pill_bg)
                                .border_1()
                                .border_color(gold_pill_border)
                                .text_color(gold_text)
                                .text_size(px(10.))
                                .font_bold()
                                .child(Icon::new(IconName::StarFill).xsmall())
                                .child(format!("{rating:.1}"))
                                .into_any_element()
                        } else {
                            div().into_any_element()
                        })
                        .child(
                            Button::new(bookmark_btn_id)
                                .small()
                                .compact()
                                .selected(bookmarked)
                                .icon(if bookmarked {
                                    IconName::Check
                                } else {
                                    IconName::Plus
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
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .opacity(0.0)
                        .group_hover("streamovie-card", |s| s.opacity(1.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size_11()
                                .rounded_full()
                                .bg(primary_accent.opacity(0.95))
                                .text_color(white())
                                .border_2()
                                .border_color(white().opacity(0.50))
                                .child(Icon::new(IconName::Play).small()),
                        ),
                )
                .child(match card_style {
                    PosterCardStyle::Widescreen => v_flex()
                        .absolute()
                        .left_3()
                        .right_3()
                        .bottom_2p5()
                        .gap_1()
                        .when(title_is_rtl, |col| col.items_end().text_right())
                        .child(
                            div()
                                .w_full()
                                .text_sm()
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
                                .text_color(hsla(215.0 / 360.0, 0.20, 0.80, 1.0))
                                .when_some(year.clone(), |row, yr| row.child(yr))
                                .when_some(first_genre.clone(), |row, g| {
                                    row.child("·").child(g)
                                }),
                        )
                        .into_any_element(),
                    PosterCardStyle::Poster => div().into_any_element(),
                }),
        )
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
                            .line_height(relative(1.25))
                            .text_color(cx.theme().foreground)
                            .group_hover("streamovie-card", move |s| s.text_color(primary_accent))
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
                            .child(first_genre.unwrap_or_else(|| fallback_type_label.to_string())),
                    ),
            )
        })
        .into_any_element()
}
