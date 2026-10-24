//! Movies and series browsing.

use gpui_kit::component::{
    ActiveTheme as _, IconName, Selectable as _, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, SharedString, Styled as _, div,
    prelude::FluentBuilder as _, px,
};

use crate::{
    app::StreamovieApp,
    i18n::{directional_row, format_count},
    models::{BrowseSort, MediaCategory},
    ui::components::{
        PosterCardStyle, empty_state, network_error_state, render_poster_card, skeleton_card,
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

    let title = match category {
        MediaCategory::Movie => dict.nav.movies,
        MediaCategory::Series => dict.nav.series,
    };
    let subtitle = match category {
        MediaCategory::Movie => dict.home.movies_subtitle,
        MediaCategory::Series => dict.home.series_subtitle,
    };

    let mut genre_buttons = Vec::new();
    let active_genre = state.genre_id;
    genre_buttons.push(
        Button::new(SharedString::from(format!(
            "genre-all-{}",
            category.as_str()
        )))
        .small()
        .rounded(px(13.))
        .selected(active_genre == 0)
        .when(active_genre == 0, |btn| btn.primary())
        .when(active_genre != 0, |btn| btn.secondary())
        .label(dict.browse.all_genres)
        .on_click(cx.listener(move |this, _, _, cx| {
            this.set_catalog_genre(category, 0, cx);
        })),
    );
    for genre in genres.into_iter().take(16) {
        let gid = genre.id;
        let selected = active_genre == gid;
        genre_buttons.push(
            Button::new(SharedString::from(format!(
                "genre-{}-{}",
                category.as_str(),
                gid
            )))
            .small()
            .rounded(px(13.))
            .selected(selected)
            .when(selected, |btn| btn.primary())
            .when(!selected, |btn| btn.secondary())
            .label(genre.display_name())
            .on_click(cx.listener(move |this, _, _, cx| {
                this.set_catalog_genre(category, gid, cx);
            })),
        );
    }

    let cards: Vec<_> = state
        .items
        .iter()
        .map(|media| {
            render_poster_card(
                app,
                media,
                category,
                app.is_bookmarked(media.id),
                locale,
                &format!("catalog-{}", category.as_str()),
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
            directional_row(h_flex(), rtl)
                .w_full()
                .gap_2()
                .flex_wrap()
                .child(
                    Button::new(SharedString::from(format!(
                        "sort-new-{}",
                        category.as_str()
                    )))
                    .small()
                    .rounded(px(13.))
                    .selected(state.sort == BrowseSort::Created)
                    .when(state.sort == BrowseSort::Created, |btn| btn.primary())
                    .when(state.sort != BrowseSort::Created, |btn| btn.secondary())
                    .label(dict.browse.sort_latest)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_catalog_sort(category, BrowseSort::Created, cx);
                    })),
                )
                .child(
                    Button::new(SharedString::from(format!(
                        "sort-imdb-{}",
                        category.as_str()
                    )))
                    .small()
                    .rounded(px(13.))
                    .selected(state.sort == BrowseSort::Imdb)
                    .when(state.sort == BrowseSort::Imdb, |btn| btn.primary())
                    .when(state.sort != BrowseSort::Imdb, |btn| btn.secondary())
                    .label(dict.browse.sort_imdb)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_catalog_sort(category, BrowseSort::Imdb, cx);
                    })),
                )
                .children(genre_buttons),
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
