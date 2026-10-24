//! Poster and landscape cards for Streamovie rails and grids.

use gpui_kit::component::{
    Icon, IconName, Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::{
    AnimationExt as _,     AnyElement, Context, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, SpringAnimation, SpringConfig,
    StatefulInteractiveElement as _, Styled as _, div, hsla, linear_color_stop,
    linear_gradient, prelude::FluentBuilder as _, px, relative, white,
};

use crate::{
    app::StreamovieApp,
    i18n::{Locale, directional_row, format_number, is_persian_text},
    models::{ApiMedia, MediaCategory},
    ui::{
        apple::{ApplePaint, apple_progress},
        image_cache::render_remote_image,
    },
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PosterCardStyle {
    /// 2:3 poster used in catalogs, movie rails, and recommendation rows.
    #[default]
    Poster,
    /// 16:9 artwork used for the featured series rail.
    Widescreen,
}

impl PosterCardStyle {
    pub fn frame(self) -> (gpui_kit::Pixels, gpui_kit::Pixels) {
        match self {
            Self::Poster => (px(176.), px(264.)),
            Self::Widescreen => (px(300.), px(169.)),
        }
    }
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
    render_media_card(
        app,
        media,
        category,
        bookmarked,
        locale,
        id_scope,
        card_style,
        None,
        None,
        true,
        cx,
    )
}

/// Poster card with an Apple TV rank numeral and an optional continue-watching bar.
#[allow(clippy::too_many_arguments)]
pub fn render_media_card(
    app: &StreamovieApp,
    media: &ApiMedia,
    category: MediaCategory,
    bookmarked: bool,
    locale: Locale,
    id_scope: &str,
    card_style: PosterCardStyle,
    rank: Option<u16>,
    progress: Option<f32>,
    show_bookmark: bool,
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
    let kind_label = if category == MediaCategory::Series {
        dict.home.series
    } else {
        dict.home.movie
    };
    let meta = if rank.is_some() {
        media
            .genre_list()
            .into_iter()
            .next()
            .unwrap_or_else(|| kind_label.to_string())
    } else {
        match &year {
            Some(year) => format!("{year}  ·  {kind_label}"),
            None => kind_label.to_string(),
        }
    };
    let (card_w, frame_h) = card_style.frame();
    let paint = ApplePaint::from_cx(cx);

    let card_id = SharedString::from(format!("card-{id_scope}-{}-{media_id}", category.as_str()));
    let title_id = SharedString::from(format!("title-{card_id}"));
    let bookmark_btn_id =
        SharedString::from(format!("bm-{id_scope}-{}-{media_id}", category.as_str()));
    let media_for_open = media.clone();
    let media_for_title = media.clone();
    let media_for_bookmark = media.clone();
    let hovered = app.card_is_hovered(&card_id);
    let hover_id = card_id.clone();
    let radius = px(14.);

    let artwork = render_remote_image(
        app,
        primary_img,
        fallback_img,
        radius,
        if category == MediaCategory::Series {
            IconName::Frame
        } else {
            IconName::GalleryVerticalEnd
        },
        Some(&title),
        cx,
    );

    let frame = div()
        .id(card_id.clone())
        .w(card_w)
        .h(frame_h)
        .rounded(radius)
        .overflow_hidden()
        .border_1()
        .border_color(paint.hairline)
        .bg(paint.chip)
        .shadow(paint.poster_shadow(hovered))
        .cursor_pointer()
        .on_hover(cx.listener(move |this, hovered, _, cx| {
            this.set_card_hovered(&hover_id, *hovered, cx);
        }))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.open_details(media_for_open.clone(), category, cx);
        }))
        .child(artwork)
        .when(rank.is_some(), |frame| {
            frame.child(
                div()
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .h(px(128.))
                    .rounded_b(radius)
                    .bg(linear_gradient(
                        180.,
                        linear_color_stop(hsla(0.0, 0.0, 0.0, 0.0), 0.),
                        linear_color_stop(hsla(0.0, 0.0, 0.0, 0.82), 1.),
                    )),
            )
        })
        .when_some(rank, |frame, rank| {
            frame.child(
                div()
                    .absolute()
                    .bottom(px(4.))
                    .when(!rtl, |label| label.left(px(12.)))
                    .when(rtl, |label| label.right(px(12.)))
                    .text_size(px(52.))
                    .font_semibold()
                    .line_height(relative(1.0))
                    .text_color(white())
                    .child(format_number(rank as usize, locale)),
            )
        })
        .when(show_bookmark || (rating > 0.0 && rank.is_none()), |frame| {
            frame.child(
            directional_row(h_flex(), rtl)
                .absolute()
                .top(px(8.))
                .left(px(8.))
                .right(px(8.))
                .justify_between()
                .items_center()
                .child(if rating > 0.0 && rank.is_none() {
                    div()
                        .px(px(6.))
                        .h(px(20.))
                        .rounded(px(6.))
                        .items_center()
                        .bg(white().opacity(0.94))
                        .shadow(paint.button_shadow(false))
                        .text_size(px(11.))
                        .font_semibold()
                        .text_color(paint.ink)
                        .child(format!("{rating:.1}"))
                        .into_any_element()
                } else {
                    div().into_any_element()
                })
                .when(show_bookmark, |row| {
                    row.child(
                    h_flex()
                        .id(bookmark_btn_id)
                        .size(px(26.))
                        .rounded_full()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .bg(if bookmarked {
                            paint.primary_fill
                        } else {
                            white().opacity(0.94)
                        })
                        .shadow(paint.button_shadow(false))
                        .child(
                            Icon::new(if bookmarked {
                                IconName::Check
                            } else {
                                IconName::Plus
                            })
                            .small()
                            .text_color(if bookmarked {
                                paint.on_primary
                            } else {
                                paint.ink
                            }),
                        )
                        .on_click(cx.listener(move |this, _, window, cx| {
                            cx.stop_propagation();
                            let added =
                                this.toggle_bookmark(media_for_bookmark.clone(), category, cx);
                            let msg = if added {
                                this.locale().dict().bookmarks.add
                            } else {
                                this.locale().dict().bookmarks.remove
                            };
                            window.push_notification(msg, cx);
                        })),
                    )
                }),
            )
        })
        .when_some(progress, |frame, fraction| {
            frame.child(
                div()
                    .absolute()
                    .bottom(px(0.))
                    .left(px(0.))
                    .right(px(0.))
                    .px(px(8.))
                    .pb(px(8.))
                    .child(apple_progress(
                        fraction,
                        white(),
                        white().opacity(0.35),
                    )),
            )
        })
        .absolute()
        .with_spring(
            SharedString::from(format!("scale-{card_id}")),
            SpringAnimation::new(SpringConfig::new(320.0, 38.0, 1.0))
                .to(if hovered { 1.035 } else { 1.0 })
                .from(1.0),
            move |card, scale| {
                let extra_w = card_w * ((scale - 1.0) * 0.5);
                let extra_h = frame_h * ((scale - 1.0) * 0.5);
                card.w(card_w * scale)
                    .h(frame_h * scale)
                    .left(-extra_w)
                    .top(-extra_h)
            },
        );

    let image_slot = div()
        .relative()
        .w(card_w)
        .h(frame_h)
        .flex_none()
        .rounded(radius)
        .bg(paint.chip)
        .child(div().w(card_w).h(frame_h).rounded(radius).bg(paint.chip))
        .child(frame);

    v_flex()
        .w(card_w)
        .flex_shrink_0()
        .gap(px(8.))
        .when(title_is_rtl, |col| col.items_end())
        .child(image_slot)
        .child(
            div()
                .id(title_id)
                .w_full()
                .cursor_pointer()
                .text_size(px(13.))
                .font_semibold()
                .line_height(relative(1.25))
                .text_color(paint.ink)
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .when(title_is_rtl, |el| el.text_right())
                .child(title.clone())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.open_details(media_for_title.clone(), category, cx);
                })),
        )
        .child(
            div()
                .w_full()
                .text_size(px(12.))
                .text_color(paint.secondary_text)
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .when(title_is_rtl, |el| el.text_right())
                .child(meta),
        )
        .into_any_element()
}
