//! Large artwork card for the home watchlist row.
//!
//! Type sizes match the Apple TV “Six Degrees” cards in the reference
//! screenshot: 22px title, 12px genre line, 13px description.

use gpui_kit::component::{IconName, StyledExt as _, h_flex, v_flex};
use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, hsla, linear_color_stop, linear_gradient,
    prelude::FluentBuilder as _, px, white,
};

use crate::{
    app::StreamovieApp,
    i18n::{directional_row, is_persian_text},
    models::{ApiMedia, MediaCategory},
    ui::{apple::ApplePaint, image_cache::render_remote_image},
};

pub const WATCHLIST_CARD_W: f32 = 300.;
pub const WATCHLIST_CARD_H: f32 = 376.;

const CARD_RADIUS: f32 = 18.;
const TITLE_SIZE: f32 = 22.;
const GENRE_SIZE: f32 = 12.;
const DESCRIPTION_SIZE: f32 = 13.;

pub fn render_watchlist_card(
    app: &StreamovieApp,
    media: &ApiMedia,
    category: MediaCategory,
    index: usize,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let rtl = app.locale().is_rtl();
    let paint = ApplePaint::from_cx(cx);
    let radius = px(CARD_RADIUS);

    let title = media.display_title();
    let overview = media
        .parsed_description()
        .overview
        .map(|text| clamp_overview(&text, 96))
        .filter(|text| !text.is_empty());
    let text_rtl = is_persian_text(&title)
        || overview.as_deref().is_some_and(is_persian_text)
        || rtl;

    let kind_label = if category == MediaCategory::Series {
        dict.home.series
    } else {
        dict.home.movie
    };
    let mut genre_parts = vec![kind_label.to_string()];
    genre_parts.extend(media.genre_list().into_iter().take(2));
    let genre_line = genre_parts.join(" · ");

    let poster = media.poster_image();
    let cover = media.hero_image().or_else(|| media.cover_image());
    let (primary, fallback) = (cover.as_deref(), poster.as_deref());

    let card_id = SharedString::from(format!("watch-card-{}-{index}", media.id));
    let hover_id = card_id.clone();
    let hovered = app.control_is_hovered(&card_id);
    let media_for_open = media.clone();

    div()
        .id(card_id)
        .w(px(WATCHLIST_CARD_W))
        .h(px(WATCHLIST_CARD_H))
        .flex_none()
        .relative()
        .rounded(radius)
        .overflow_hidden()
        .cursor_pointer()
        .bg(paint.chip)
        .border_1()
        .border_color(if hovered {
            white().opacity(0.75)
        } else {
            paint.hairline
        })
        .shadow(paint.poster_shadow(hovered))
        .on_hover(cx.listener(move |this, hovered, _, cx| {
            this.set_control_hovered(&hover_id, *hovered, cx);
        }))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.open_details(media_for_open.clone(), category, cx);
        }))
        .child(
            div()
                .absolute()
                .inset_0()
                .rounded(radius)
                .overflow_hidden()
                .bg(paint.chip)
                .child(render_remote_image(
                    app,
                    primary,
                    fallback,
                    radius,
                    IconName::GalleryVerticalEnd,
                    None,
                    cx,
                )),
        )
        .child(
            div()
                .absolute()
                .bottom_0()
                .left_0()
                .right_0()
                .h(px(210.))
                .rounded_b(radius)
                .bg(linear_gradient(
                    180.,
                    linear_color_stop(hsla(0.0, 0.0, 0.0, 0.0), 0.),
                    linear_color_stop(hsla(0.0, 0.0, 0.0, 0.92), 1.),
                )),
        )
        .child(
            v_flex()
                .absolute()
                .bottom(px(16.))
                .left(px(16.))
                .right(px(16.))
                .gap(px(5.))
                .when(text_rtl, |col| col.items_end())
                .child(
                    div()
                        .w_full()
                        .text_size(px(TITLE_SIZE))
                        .font_bold()
                        .line_height(px(26.))
                        .text_color(white())
                        .text_ellipsis()
                        .when(text_rtl, |label| label.text_right())
                        .child(title),
                )
                .child(
                    directional_row(h_flex(), text_rtl)
                        .w_full()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .size(px(10.))
                                .flex_none()
                                .rounded(px(2.))
                                .bg(white().opacity(0.92)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_size(px(GENRE_SIZE))
                                .line_height(px(16.))
                                .text_color(white().opacity(0.92))
                                .text_ellipsis()
                                .when(text_rtl, |label| label.text_right())
                                .child(genre_line),
                        ),
                )
                .when_some(overview, |col, overview| {
                    col.child(
                        div()
                            .w_full()
                            .max_h(px(36.))
                            .overflow_hidden()
                            .text_size(px(DESCRIPTION_SIZE))
                            .line_height(px(18.))
                            .text_color(white().opacity(0.94))
                            .when(text_rtl, |label| label.text_right())
                            .child(overview),
                    )
                }),
        )
        .into_any_element()
}

fn clamp_overview(text: &str, max_chars: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max_chars {
        return flat;
    }
    let cut: String = flat.chars().take(max_chars).collect();
    let trimmed = cut
        .rsplit_once(' ')
        .map(|(head, _)| head)
        .unwrap_or(cut.as_str())
        .trim();
    format!("{trimmed}…")
}
