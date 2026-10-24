//! Poster and landscape cards for Streamovie rails and grids.

use gpui_kit::component::{
    ActiveTheme as _, IconName, Selectable as _, Sizable as _, StyledExt as _, WindowExt as _,
    button::Button, h_flex, v_flex,
};
use gpui_kit::{
    AnyElement, BoxShadow, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, black, div, hsla,
    linear_color_stop, linear_gradient, point, prelude::FluentBuilder as _, px, relative, white,
};

use crate::{
    app::StreamovieApp,
    i18n::{Locale, directional_row, is_persian_text},
    models::{ApiMedia, MediaCategory},
    ui::image_cache::render_remote_image,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PosterCardStyle {
    /// 2:3 poster used in catalogs and movie rails.
    #[default]
    Poster,
    /// 16:9 artwork used for featured series rails.
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
    let (card_w, frame_h) = match card_style {
        PosterCardStyle::Poster => (px(168.), px(252.)),
        PosterCardStyle::Widescreen => (px(280.), px(158.)),
    };

    let card_id = SharedString::from(format!("card-{id_scope}-{}-{media_id}", category.as_str()));
    let bookmark_btn_id =
        SharedString::from(format!("bm-{id_scope}-{}-{media_id}", category.as_str()));
    let media_for_open = media.clone();
    let media_for_bookmark = media.clone();
    let group = SharedString::from(format!("card-{id_scope}"));
    let hover_group = group.clone();

    let artwork = render_remote_image(
        app,
        primary_img,
        fallback_img,
        px(10.),
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
        .gap_2()
        .cursor_pointer()
        .group(group)
        .on_click(cx.listener(move |this, _, _, cx| {
            this.open_details(media_for_open.clone(), category, cx);
        }))
        .child(
            div()
                .relative()
                .w(card_w)
                .h(frame_h)
                .rounded(px(10.))
                .overflow_hidden()
                .bg(cx.theme().muted)
                .shadow(vec![BoxShadow {
                    offset: point(px(0.), px(8.)),
                    blur_radius: px(18.),
                    spread_radius: px(0.),
                    inset: false,
                    color: hsla(0.0, 0.0, 0.0, 0.35),
                }])
                .group_hover(hover_group, |style| style.opacity(0.92))
                .child(artwork)
                .child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .h(px(72.))
                        .bg(linear_gradient(
                            0.,
                            linear_color_stop(black().opacity(0.72), 0.0),
                            linear_color_stop(black().opacity(0.0), 1.0),
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
                        .child(if rating > 0.0 {
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(black().opacity(0.55))
                                .text_xs()
                                .font_semibold()
                                .text_color(white())
                                .child(format!("{rating:.1}"))
                                .into_any_element()
                        } else {
                            div().into_any_element()
                        })
                        .child(
                            Button::new(bookmark_btn_id)
                                .small()
                                .compact()
                                .outline()
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
                                        this.locale().dict().bookmarks.add
                                    } else {
                                        this.locale().dict().bookmarks.remove
                                    };
                                    window.push_notification(msg, cx);
                                })),
                        ),
                ),
        )
        .child(
            v_flex()
                .w_full()
                .px_0p5()
                .gap_0p5()
                .when(title_is_rtl, |col| col.items_end().text_right())
                .child(
                    div()
                        .w_full()
                        .text_sm()
                        .font_semibold()
                        .line_height(relative(1.25))
                        .text_color(cx.theme().foreground)
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis()
                        .when(title_is_rtl, |el| el.text_right())
                        .child(title),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(year.unwrap_or_else(|| {
                            if category == MediaCategory::Series {
                                dict.home.series.to_string()
                            } else {
                                dict.home.movie.to_string()
                            }
                        })),
                ),
        )
        .into_any_element()
}
