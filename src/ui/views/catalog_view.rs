//! Movies and series browsing.

use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Selectable as _, Sizable as _, StyledExt as _,
    alert::Alert,
    button::{Button, ButtonVariants as _},
    empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle},
    h_flex,
    skeleton::Skeleton,
    v_flex,
};
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, SharedString, Styled as _, div,
    prelude::FluentBuilder as _, px,
};

use crate::{
    app::StreamovieApp,
    i18n::{directional_row, format_count},
    models::{BrowseSort, MediaCategory},
    ui::components::render_poster_card,
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
        .selected(active_genre == 0)
        .when(active_genre == 0, |btn| btn.primary())
        .when(active_genre != 0, |btn| btn.outline())
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
            .selected(selected)
            .when(selected, |btn| btn.primary())
            .when(!selected, |btn| btn.outline())
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
        .px_8()
        .pt_6()
        .pb_12()
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
                        .child(div().text_2xl().font_semibold().child(title))
                        .child(
                            div()
                                .text_sm()
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
                    .selected(state.sort == BrowseSort::Created)
                    .when(state.sort == BrowseSort::Created, |btn| btn.primary())
                    .when(state.sort != BrowseSort::Created, |btn| btn.outline())
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
                    .selected(state.sort == BrowseSort::Imdb)
                    .when(state.sort == BrowseSort::Imdb, |btn| btn.primary())
                    .when(state.sort != BrowseSort::Imdb, |btn| btn.outline())
                    .label(dict.browse.sort_imdb)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_catalog_sort(category, BrowseSort::Imdb, cx);
                    })),
                )
                .children(genre_buttons),
        )
        .child(if state.loading && state.items.is_empty() {
            h_flex()
                .gap_4()
                .flex_wrap()
                .children((0..8).map(|_| {
                    Skeleton::new()
                        .w(px(168.))
                        .h(px(252.))
                        .rounded(px(10.))
                        .into_any_element()
                }))
                .into_any_element()
        } else if let Some(err) = state.error.clone() {
            v_flex()
                .gap_3()
                .child(Alert::error("catalog-error", err).title(dict.browse.load_error))
                .child(
                    Button::new("catalog-retry")
                        .outline()
                        .icon(IconName::RefreshCw)
                        .label(dict.actions.retry)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.reload_catalog(category, cx);
                        })),
                )
                .into_any_element()
        } else if cards.is_empty() {
            Empty::new()
                .header(
                    EmptyHeader::new()
                        .media(
                            EmptyMedia::new()
                                .with_variant(EmptyMediaVariant::Icon)
                                .child(Icon::new(IconName::Search)),
                        )
                        .title(EmptyTitle::new().child(dict.browse.empty))
                        .description(EmptyDescription::new().child(dict.browse.empty_hint)),
                )
                .into_any_element()
        } else {
            v_flex()
                .gap_6()
                .child(
                    h_flex()
                        .flex_wrap()
                        .gap_x_4()
                        .gap_y_6()
                        .when(rtl, |row| row.flex_row_reverse())
                        .children(cards),
                )
                .when(state.has_more, |col| {
                    col.child(
                        Button::new(SharedString::from(format!(
                            "load-more-{}",
                            category.as_str()
                        )))
                        .outline()
                        .label(if state.loading_more {
                            dict.browse.loading_more
                        } else {
                            dict.browse.load_more
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.load_more_catalog(category, cx);
                        })),
                    )
                })
                .into_any_element()
        })
        .into_any_element()
}
