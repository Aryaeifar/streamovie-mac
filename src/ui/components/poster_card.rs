//! Poster and landscape cards for Streamovie rails and grids.

use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::{
    AnimationExt as _, AnyElement, BoxShadow, Context, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, SpringAnimation, SpringConfig,
    StatefulInteractiveElement as _, Styled as _, black, deferred, div, hsla, linear_color_stop,
    linear_gradient, point, prelude::FluentBuilder as _, px, relative, white,
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

impl PosterCardStyle {
    pub fn frame(self) -> (gpui_kit::Pixels, gpui_kit::Pixels) {
        match self {
            Self::Poster => (px(168.), px(252.)),
            Self::Widescreen => (px(272.), px(153.)),
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
    let meta = match &year {
        Some(year) => format!("{year}  ·  {kind_label}"),
        None => kind_label.to_string(),
    };
    let (card_w, frame_h) = card_style.frame();

    let card_id = SharedString::from(format!("card-{id_scope}-{}-{media_id}", category.as_str()));
    let bookmark_btn_id =
        SharedString::from(format!("bm-{id_scope}-{}-{media_id}", category.as_str()));
    let media_for_open = media.clone();
    let media_for_bookmark = media.clone();
    let hovered = app.card_is_hovered(&card_id);
    let hover_id = card_id.clone();
    let base_h = frame_h;

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

    let card = v_flex()
        .id(card_id.clone())
        .w(card_w)
        .h(base_h)
        .flex_shrink_0()
        .cursor_pointer()
        .on_hover(cx.listener(move |this, hovered, _, cx| {
            this.set_card_hovered(&hover_id, *hovered, cx);
        }))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.open_details(media_for_open.clone(), category, cx);
        }))
        .child(
            div()
                .relative()
                .size_full()
                .rounded(px(10.))
                .overflow_hidden()
                .border_1()
                .border_color(white().opacity(0.10))
                .bg(cx.theme().muted)
                .shadow(vec![BoxShadow {
                    offset: point(px(0.), px(2.)),
                    blur_radius: px(10.),
                    spread_radius: px(0.),
                    inset: false,
                    color: hsla(0.0, 0.0, 0.0, 0.28),
                }])
                .child(artwork)
                .child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .h(px(108.))
                        .bg(linear_gradient(
                            0.,
                            linear_color_stop(black().opacity(0.88), 0.0),
                            linear_color_stop(black().opacity(0.0), 1.0),
                        )),
                )
                .child(
                    directional_row(h_flex(), rtl)
                        .absolute()
                        .top(px(8.))
                        .left(px(8.))
                        .right(px(8.))
                        .justify_between()
                        .items_center()
                        .child(if rating > 0.0 {
                            div()
                                .px(px(6.))
                                .h(px(22.))
                                .rounded(px(6.))
                                .items_center()
                                .bg(black().opacity(0.55))
                                .text_size(px(11.))
                                .font_semibold()
                                .text_color(white())
                                .child(format!("{rating:.1}"))
                                .into_any_element()
                        } else {
                            div().into_any_element()
                        })
                        .child(
                            h_flex()
                                .id(bookmark_btn_id)
                                .size(px(28.))
                                .rounded_full()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .bg(black().opacity(if bookmarked { 0.72 } else { 0.45 }))
                                .child(
                                    Icon::new(if bookmarked {
                                        IconName::Check
                                    } else {
                                        IconName::Plus
                                    })
                                    .small()
                                    .text_color(white()),
                                )
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
                )
                .child(
                    v_flex()
                        .absolute()
                        .left(px(10.))
                        .right(px(10.))
                        .bottom(px(10.))
                        .gap(px(2.))
                        .when(title_is_rtl, |col| col.items_end().text_right())
                        .child(
                            div()
                                .w_full()
                                .text_size(px(13.))
                                .font_semibold()
                                .line_height(relative(1.2))
                                .text_color(white())
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .text_ellipsis()
                                .when(title_is_rtl, |el| el.text_right())
                                .child(title),
                        )
                        .child(
                            div()
                                .w_full()
                                .text_size(px(11.))
                                .text_color(white().opacity(0.72))
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .text_ellipsis()
                                .when(title_is_rtl, |el| el.text_right())
                                .child(meta),
                        ),
                ),
        )
        .absolute()
        .with_spring(
            SharedString::from(format!("scale-{card_id}")),
            SpringAnimation::new(SpringConfig::new(320.0, 38.0, 1.0))
                .to(if hovered { 1.03 } else { 1.0 })
                .from(1.0),
            move |card, scale| {
                let extra_w = card_w * ((scale - 1.0) * 0.5);
                let extra_h = base_h * ((scale - 1.0) * 0.5);
                // Grow from the center. The slot below keeps a constant size,
                // so the row does not reflow while the spring moves.
                card.w(card_w * scale)
                    .h(base_h * scale)
                    .left(-extra_w)
                    .top(-extra_h)
            },
        );
    // The slot is the flex item. Scaling stays on the absolute card.
    // The spacer stays in the row so the carousel can measure a real card width.
    // The artwork is absolute and may scale without changing that width.
    let slot = div()
        .relative()
        .w(card_w)
        .h(base_h)
        .flex_none()
        .child(div().w(card_w).h(base_h))
        .child(if hovered {
            deferred(card).priority(1).into_any_element()
        } else {
            card.into_any_element()
        });
    slot.into_any_element()
}
