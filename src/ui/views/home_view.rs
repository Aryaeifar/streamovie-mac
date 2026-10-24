//! Home discovery: horizontal rails.

use gpui_kit::component::{
    IconName, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
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
    ui::components::{
        PosterCardStyle, empty_state, home_skeleton, network_error_state,
        render_poster_card_with_style, skeleton_row,
    },
};

pub fn render_home_view(app: &StreamovieApp, cx: &mut Context<StreamovieApp>) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let home = app.home_state();

    if home.loading
        && home.latest_movies.is_empty()
        && home.latest_series.is_empty()
        && home.imdb_movies.is_empty()
        && home.imdb_series.is_empty()
    {
        return home_skeleton(rtl);
    }

    v_flex()
        .w_full()
        .min_w_0()
        .max_w_full()
        .overflow_hidden()
        .gap(px(36.))
        .pt(px(8.))
        .pb(px(56.))
        .child(rail(
            app,
            0,
            "latest-series",
            dict.home.series_row,
            &home.latest_series,
            home.latest_series_error.as_deref(),
            dict.common.series_failed,
            home.loading,
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
            home.latest_movies_error.as_deref(),
            dict.common.movies_failed,
            home.loading,
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
            home.imdb_movies_error.as_deref(),
            dict.common.movies_failed,
            home.loading,
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
            home.imdb_series_error.as_deref(),
            dict.common.series_failed,
            home.loading,
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
    fetch_error: Option<&str>,
    failed_title: &str,
    loading: bool,
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
    let card_gap = px(20.);
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
        .px(px(32.))
        .gap(px(12.))
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
                                .secondary()
                                .rounded(px(12.))
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
                                .secondary()
                                .rounded(px(12.))
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
                                .ghost()
                                .label(dict.actions.see_more)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.navigate_to_catalog_with_sort(target_nav, target_sort, cx);
                                })),
                        ),
                ),
        )
        .child(if cards.is_empty() && loading {
            let count = if card_style == PosterCardStyle::Widescreen {
                4
            } else {
                6
            };
            skeleton_row(card_style, count, rtl)
        } else if cards.is_empty() && fetch_error.is_some() {
            network_error_state(
                dict.common.network_title,
                failed_title,
                dict.common.network_body,
                fetch_error.map(str::to_string),
                Some(
                    Button::new(SharedString::from(format!("rail-retry-{section_key}")))
                        .primary()
                        .rounded(px(13.))
                        .icon(IconName::RefreshCw)
                        .label(dict.actions.retry)
                        .on_click(cx.listener(|this, _, _, cx| this.reload_home_feeds(cx)))
                        .into_any_element(),
                ),
                cx,
            )
        } else if cards.is_empty() {
            empty_state(IconName::Inbox, dict.home.empty, "", None, cx)
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
                        .py(px(16.))
                        .when(rtl, |row| row.flex_row_reverse())
                        .children(cards),
                )
                .into_any_element()
        })
        .into_any_element()
}
