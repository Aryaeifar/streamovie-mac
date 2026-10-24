//! Movies and series browsing.

use gpui_kit::component::{
    ActiveTheme as _, IconName, Selectable as _, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};

use crate::{
    api::streamovie::TOP250_GENRE_ID,
    app::StreamovieApp,
    i18n::{Dictionary, directional_row, format_count},
    models::{
        BrowseSort, MediaCategory,
        imdb_top250::chart_rank,
    },
    state::{CatalogGridState, NavSection},
    ui::components::{
        PosterCardStyle, empty_state, network_error_state, render_media_card, skeleton_card,
        skeleton_grid,
    },
};

pub fn render_catalog_view(
    app: &StreamovieApp,
    category: MediaCategory,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let state = app.catalog_state(category).clone();
    let genres = app.genres().to_vec();

    let title = if state.chart {
        match category {
            MediaCategory::Movie => dict.browse.top_movies,
            MediaCategory::Series => dict.browse.top_series,
        }
    } else if state.sort == BrowseSort::Imdb && state.genre_id == 0 {
        dict.browse.popular
    } else {
        match category {
            MediaCategory::Movie => dict.nav.movies,
            MediaCategory::Series => dict.nav.series,
        }
    };
    let subtitle = if state.chart {
        match category {
            MediaCategory::Movie => dict.home.imdb_movies_subtitle,
            MediaCategory::Series => dict.home.imdb_series_subtitle,
        }
    } else {
        match category {
            MediaCategory::Movie => dict.home.movies_subtitle,
            MediaCategory::Series => dict.home.series_subtitle,
        }
    };

    let list_chips = list_filter_chips(app, category, &state, cx);
    let detail_chips = detail_filter_chips(category, &state, &genres, dict, cx);

    let cards: Vec<_> = state
        .items
        .iter()
        .enumerate()
        .map(|(index, media)| {
            let rank = if state.chart {
                chart_rank(media, category).or(Some((index + 1) as u16))
            } else {
                None
            };
            render_media_card(
                app,
                media,
                category,
                app.is_bookmarked(media.id),
                locale,
                &format!("catalog-{}", category.as_str()),
                PosterCardStyle::Poster,
                rank,
                None,
                true,
                cx,
            )
        })
        .collect();

    v_flex()
        .w_full()
        .px(px(32.))
        .pt(px(24.))
        .pb(px(48.))
        .gap_5()
        .child(
            directional_row(h_flex(), rtl)
                .w_full()
                .justify_between()
                .items_end()
                .child(
                    v_flex()
                        .gap_1()
                        .when(rtl, |col| col.items_end().text_right())
                        .child(div().text_size(px(28.)).font_semibold().child(title))
                        .child(
                            div()
                                .text_size(px(13.))
                                .text_color(cx.theme().muted_foreground)
                                .child(subtitle),
                        ),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(format_count(dict.browse.results, state.items.len(), locale)),
                ),
        )
        .child(
            v_flex()
                .w_full()
                .gap_2()
                .child(
                    div()
                        .text_size(px(12.))
                        .font_semibold()
                        .text_color(cx.theme().muted_foreground)
                        .child(dict.browse.filters),
                )
                .child(filter_scroller(
                    &format!("list-filters-{}", category.as_str()),
                    rtl,
                    list_chips,
                ))
                .child(filter_scroller(
                    &format!("detail-filters-{}", category.as_str()),
                    rtl,
                    detail_chips,
                )),
        )
        .child(if state.loading && state.items.is_empty() {
            skeleton_grid(8, rtl)
        } else if let Some(err) = state.error.clone().filter(|_| state.items.is_empty()) {
            let failed_title = match category {
                MediaCategory::Movie => dict.common.movies_failed,
                MediaCategory::Series => dict.common.series_failed,
            };
            network_error_state(
                dict.common.network_title,
                failed_title,
                dict.common.network_body,
                Some(err),
                Some(
                    Button::new(SharedString::from(format!(
                        "catalog-retry-{}",
                        category.as_str()
                    )))
                    .primary()
                    .rounded(px(13.))
                    .icon(IconName::RefreshCw)
                    .label(dict.actions.retry)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.reload_catalog(category, cx);
                    }))
                    .into_any_element(),
                ),
                cx,
            )
        } else if cards.is_empty() {
            empty_state(
                IconName::Inbox,
                dict.browse.empty,
                dict.browse.empty_hint,
                None,
                cx,
            )
        } else {
            let failed_title = match category {
                MediaCategory::Movie => dict.common.movies_failed,
                MediaCategory::Series => dict.common.series_failed,
            };
            let page_error = state.error.clone();
            v_flex()
                .gap_6()
                .child(
                    h_flex()
                        .flex_wrap()
                        .gap(px(22.))
                        .py(px(12.))
                        .when(rtl, |row| row.flex_row_reverse())
                        .children(cards)
                        .when(state.loading_more, |row| {
                            row.children((0..4).map(|_| skeleton_card(PosterCardStyle::Poster)))
                        }),
                )
                .when_some(page_error, |col, message| {
                    col.child(network_error_state(
                        dict.common.network_title,
                        failed_title,
                        dict.common.network_body,
                        Some(message),
                        Some(
                            Button::new(SharedString::from(format!(
                                "load-more-retry-{}",
                                category.as_str()
                            )))
                            .primary()
                            .rounded(px(13.))
                            .icon(IconName::RefreshCw)
                            .label(dict.actions.retry)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.load_more_catalog(category, cx);
                            }))
                            .into_any_element(),
                        ),
                        cx,
                    ))
                })
                .into_any_element()
        })
        .into_any_element()
}

fn list_filter_chips(
    _app: &StreamovieApp,
    category: MediaCategory,
    state: &CatalogGridState,
    cx: &mut Context<StreamovieApp>,
) -> Vec<AnyElement> {
    let dict = _app.locale().dict();
    let plain = !state.chart && state.sort == BrowseSort::Created && state.genre_id == 0;
    let movies = category == MediaCategory::Movie && plain;
    let series = category == MediaCategory::Series && plain;
    let popular = !state.chart && state.sort == BrowseSort::Imdb && state.genre_id == 0;
    let top_movies = category == MediaCategory::Movie && state.chart;
    let top_series = category == MediaCategory::Series && state.chart;
    vec![
        filter_chip(
            format!("list-movies-{}", category.as_str()),
            dict.browse.movies,
            movies,
            cx,
            |this, cx| this.navigate_to_catalog_with_sort(NavSection::Movies, BrowseSort::Created, cx),
        ),
        filter_chip(
            format!("list-series-{}", category.as_str()),
            dict.browse.series,
            series,
            cx,
            |this, cx| this.navigate_to_catalog_with_sort(NavSection::Series, BrowseSort::Created, cx),
        ),
        filter_chip(
            format!("list-popular-{}", category.as_str()),
            dict.browse.popular,
            popular,
            cx,
            |this, cx| this.open_popular_list(cx),
        ),
        filter_chip(
            format!("list-top-movies-{}", category.as_str()),
            dict.browse.top_movies,
            top_movies,
            cx,
            |this, cx| this.open_chart(MediaCategory::Movie, cx),
        ),
        filter_chip(
            format!("list-top-series-{}", category.as_str()),
            dict.browse.top_series,
            top_series,
            cx,
            |this, cx| this.open_chart(MediaCategory::Series, cx),
        ),
    ]
}

fn detail_filter_chips(
    category: MediaCategory,
    state: &CatalogGridState,
    genres: &[crate::models::ApiGenre],
    dict: &Dictionary,
    cx: &mut Context<StreamovieApp>,
) -> Vec<AnyElement> {
    let mut chips = vec![
        filter_chip(
            format!("sort-new-{}", category.as_str()),
            dict.browse.sort_latest,
            !state.chart && state.sort == BrowseSort::Created,
            cx,
            move |this, cx| this.set_catalog_sort(category, BrowseSort::Created, cx),
        ),
        filter_chip(
            format!("sort-imdb-{}", category.as_str()),
            dict.browse.sort_imdb,
            !state.chart && state.sort == BrowseSort::Imdb,
            cx,
            move |this, cx| this.set_catalog_sort(category, BrowseSort::Imdb, cx),
        ),
        filter_chip(
            format!("genre-all-{}", category.as_str()),
            dict.browse.all_genres,
            !state.chart && state.genre_id == 0,
            cx,
            move |this, cx| this.set_catalog_genre(category, 0, cx),
        ),
    ];
    for genre in genres
        .iter()
        .filter(|genre| genre.id != 0 && genre.id != TOP250_GENRE_ID)
    {
        let gid = genre.id;
        let selected = !state.chart && state.genre_id == gid;
        chips.push(filter_chip(
            format!("genre-{}-{}", category.as_str(), gid),
            genre.display_name(),
            selected,
            cx,
            move |this, cx| this.set_catalog_genre(category, gid, cx),
        ));
    }
    let dirty = state.chart || state.genre_id != 0 || state.sort != BrowseSort::Created;
    if dirty {
        chips.push(filter_chip(
            format!("clear-filters-{}", category.as_str()),
            dict.browse.clear_filters,
            false,
            cx,
            move |this, cx| this.reset_catalog_filters(category, cx),
        ));
    }
    chips
}

fn filter_chip(
    id: String,
    label: impl Into<SharedString>,
    selected: bool,
    cx: &mut Context<StreamovieApp>,
    on_click: impl Fn(&mut StreamovieApp, &mut Context<StreamovieApp>) + 'static,
) -> AnyElement {
    Button::new(SharedString::from(id))
        .small()
        .rounded(px(13.))
        .selected(selected)
        .when(selected, |btn| btn.primary())
        .when(!selected, |btn| btn.secondary())
        .label(label)
        .on_click(cx.listener(move |this, _, _, cx| on_click(this, cx)))
        .into_any_element()
}

fn filter_scroller(id: &str, rtl: bool, chips: Vec<AnyElement>) -> AnyElement {
    div()
        .id(SharedString::from(id))
        .w_full()
        .min_w_0()
        .overflow_x_scroll()
        .child(
            h_flex()
                .flex_none()
                .gap_2()
                .py(px(2.))
                .when(rtl, |row| row.flex_row_reverse())
                .children(chips),
        )
        .into_any_element()
}
