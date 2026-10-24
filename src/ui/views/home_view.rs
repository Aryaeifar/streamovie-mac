//! Home discovery: horizontal rails.

use gpui_kit::component::{
    ActiveTheme as _, IconName, Sizable as _, StyledExt as _, alert::Alert, button::Button, h_flex,
    skeleton::Skeleton, v_flex,
};
use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};

use crate::{
    app::StreamovieApp,
    i18n::directional_row,
    models::{ApiMedia, BrowseSort, MediaCategory},
    state::NavSection,
    ui::components::{PosterCardStyle, render_poster_card_with_style},
};

pub fn render_home_view(app: &StreamovieApp, cx: &mut Context<StreamovieApp>) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let home = app.home_state();

    if home.loading && home.latest_movies.is_empty() && home.latest_series.is_empty() {
        return v_flex()
            .w_full()
            .px(px(24.))
            .pt(px(20.))
            .gap_6()
            .child(
                h_flex()
                    .px_8()
                    .gap_3()
                    .child(Skeleton::new().w(px(168.)).h(px(252.)).rounded(px(10.)))
                    .child(Skeleton::new().w(px(168.)).h(px(252.)).rounded(px(10.)))
                    .child(Skeleton::new().w(px(168.)).h(px(252.)).rounded(px(10.))),
            )
            .into_any_element();
    }

    let load_error = home.error.clone().filter(|_| {
        home.latest_movies.is_empty()
            && home.latest_series.is_empty()
            && home.imdb_movies.is_empty()
            && home.imdb_series.is_empty()
    });

    v_flex()
        .w_full()
        .min_w_0()
        .max_w_full()
        .overflow_hidden()
        .gap(px(28.))
        .pt(px(12.))
        .pb_16()
        .when_some(load_error, |col, err| {
            col.child(
                v_flex()
                    .px(px(24.))
                    .gap_3()
                    .child(Alert::error("home-load-error", err).title(dict.home.unavailable_title))
                    .child(
                        Button::new("home-retry")
                            .outline()
                            .icon(IconName::RefreshCw)
                            .label(dict.actions.retry)
                            .on_click(cx.listener(|this, _, _, cx| this.reload_home_feeds(cx))),
                    ),
            )
        })
        .child(rail(
            app,
            0,
            "latest-series",
            dict.home.series_row,
            &home.latest_series,
            MediaCategory::Series,
            NavSection::Series,
            BrowseSort::Created,
            PosterCardStyle::Widescreen,
            rtl,
            cx,
        ))
        .child(rail(
            app,
            1,
            "latest-movies",
            dict.home.movies,
            &home.latest_movies,
            MediaCategory::Movie,
            NavSection::Movies,
            BrowseSort::Created,
            PosterCardStyle::Poster,
            rtl,
            cx,
        ))
        .child(rail(
            app,
            2,
            "imdb-movies",
            dict.home.imdb_movies,
            &home.imdb_movies,
            MediaCategory::Movie,
            NavSection::Movies,
            BrowseSort::Imdb,
            PosterCardStyle::Poster,
            rtl,
            cx,
        ))
        .child(rail(
            app,
            3,
            "imdb-series",
            dict.home.imdb_series,
            &home.imdb_series,
            MediaCategory::Series,
            NavSection::Series,
            BrowseSort::Imdb,
            PosterCardStyle::Poster,
            rtl,
            cx,
        ))
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn rail(
    app: &StreamovieApp,
    row_index: usize,
    section_key: &str,
    title: &str,
    items: &[ApiMedia],
    category: MediaCategory,
    target_nav: NavSection,
    target_sort: BrowseSort,
    card_style: PosterCardStyle,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let locale = app.locale();
    let scroll_handle = app.home_row_scroll_handle(row_index).clone();
    let (card_w, _) = card_style.frame();
    let card_gap = px(16.);
    let cards: Vec<_> = items
        .iter()
        .take(18)
        .map(|media| {
            render_poster_card_with_style(
                app,
                media,
                category,
                app.is_bookmarked(media.id),
                locale,
                section_key,
                card_style,
                cx,
            )
        })
        .collect();
    let count = cards.len() as f32;
    let row_w = if count == 0. {
        px(0.)
    } else {
        card_w * count + card_gap * (count - 1.)
    };

    v_flex()
        .w_full()
        .min_w_0()
        .max_w_full()
        .px(px(24.))
        .gap(px(10.))
        .child(
            directional_row(h_flex(), rtl)
                .w_full()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .text_size(px(22.))
                        .font_semibold()
                        .child(title.to_string()),
                )
                .child(
                    directional_row(h_flex(), rtl)
                        .gap_1()
                        .child(
                            Button::new(SharedString::from(format!("rail-prev-{section_key}")))
                                .small()
                                .outline()
                                .icon(if rtl {
                                    IconName::ChevronRight
                                } else {
                                    IconName::ChevronLeft
                                })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.scroll_home_row(row_index, -1, cx);
                                })),
                        )
                        .child(
                            Button::new(SharedString::from(format!("rail-next-{section_key}")))
                                .small()
                                .outline()
                                .icon(if rtl {
                                    IconName::ChevronLeft
                                } else {
                                    IconName::ChevronRight
                                })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.scroll_home_row(row_index, 1, cx);
                                })),
                        )
                        .child(
                            Button::new(SharedString::from(format!("rail-more-{section_key}")))
                                .small()
                                .outline()
                                .label(dict.actions.see_more)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.navigate_to_catalog_with_sort(target_nav, target_sort, cx);
                                })),
                        ),
                ),
        )
        .child(if cards.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(dict.home.empty)
                .into_any_element()
        } else {
            div()
                .id(SharedString::from(format!("rail-{section_key}")))
                .w_full()
                .min_w_0()
                .overflow_x_scroll()
                .restrict_scroll_to_axis()
                .track_scroll(&scroll_handle)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &MouseDownEvent, _, _| {
                        this.begin_rail_drag(row_index, event.position.x);
                    }),
                )
                .on_mouse_move(cx.listener(move |this, event: &MouseMoveEvent, _, cx| {
                    if event.dragging() {
                        this.pan_rail(row_index, event.position.x, cx);
                    }
                }))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |this, _: &MouseUpEvent, _, _| {
                        this.end_rail_drag();
                    }),
                )
                .child(
                    h_flex()
                        .id(SharedString::from(format!("rail-row-{section_key}")))
                        .flex_none()
                        .w(row_w)
                        .gap(card_gap)
                        .py(px(12.))
                        .when(rtl, |row| row.flex_row_reverse())
                        .children(cards),
                )
                .into_any_element()
        })
        .into_any_element()
}
