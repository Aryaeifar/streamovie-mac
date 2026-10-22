//! Filterable, paginated Movies and Series catalog grid view redesigned with a
//! Linear + Infuse 8 filter bar and cinema poster grid.

use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, SharedString, Styled as _, div,
    prelude::FluentBuilder as _, px,
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Selectable as _, Sizable as _, StyledExt as _,
    alert::Alert,
    button::{Button, ButtonVariants as _},
    empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle},
    h_flex,
    skeleton::Skeleton,
    tab::{Tab, TabBar},
    tag::Tag,
    v_flex,
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
    let radius_lg = cx.theme().radius_lg;

    let heading_title = match category {
        MediaCategory::Movie => dict.nav.movies,
        MediaCategory::Series => dict.nav.series,
    };
    let heading_subtitle = match category {
        MediaCategory::Movie => dict.home.movies_subtitle,
        MediaCategory::Series => dict.home.series_subtitle,
    };

    let selected_sort_ix = match state.sort {
        BrowseSort::Created => 0,
        BrowseSort::Imdb => 1,
    };

    let sort_tab_id = SharedString::from(format!("catalog-sort-tabs-{}", category.as_str()));
    let scope_key = format!("catalog-{}", category.as_str());

    let mut genre_buttons = Vec::new();
    if !genres.is_empty() {
        let active_genre = state.genre_id;
        genre_buttons.push(
            Button::new("genre-all")
                .small()
                .selected(active_genre == 0)
                .when(active_genre == 0, |btn| btn.primary())
                .when(active_genre != 0, |btn| btn.outline())
                .label(dict.browse.all_genres)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_catalog_genre(category, 0, cx);
                })),
        );
        for genre in genres.into_iter().take(18) {
            let gid = genre.id;
            let is_selected = active_genre == gid;
            genre_buttons.push(
                Button::new(SharedString::from(format!("genre-pill-{gid}")))
                    .small()
                    .selected(is_selected)
                    .when(is_selected, |btn| btn.primary())
                    .when(!is_selected, |btn| btn.outline())
                    .label(genre.display_name())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_catalog_genre(category, gid, cx);
                    })),
            );
        }
    }

    let mut cards = Vec::with_capacity(state.items.len());
    for media in &state.items {
        cards.push(render_poster_card(
            app,
            media,
            category,
            app.is_bookmarked(media.id),
            locale,
            &scope_key,
            cx,
        ));
    }

    v_flex()
        .w_full()
        .gap_6()
        .pb_10()
        // Linear-inspired Elevated Filter & Sort Surface
        .child(
            v_flex()
                .w_full()
                .p_5()
                .gap_4()
                .rounded(cx.theme().radius_lg)
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().popover)
                .child(
                    directional_row(h_flex(), rtl)
                        .w_full()
                        .justify_between()
                        .items_center()
                        .flex_wrap()
                        .gap_4()
                        .child(
                            directional_row(h_flex(), rtl)
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .w(px(4.))
                                        .h(px(26.))
                                        .rounded_full()
                                        .bg(cx.theme().primary),
                                )
                                .child(
                                    v_flex()
                                        .gap_0p5()
                                        .when(rtl, |this| this.items_end().text_right())
                                        .child(
                                            directional_row(h_flex(), rtl)
                                                .items_center()
                                                .gap_3()
                                                .child(
                                                    div()
                                                        .text_2xl()
                                                        .font_bold()
                                                        .child(heading_title),
                                                )
                                                .child(
                                                    Tag::secondary()
                                                        .small()
                                                        .outline()
                                                        .child(format_count(
                                                            dict.browse.results,
                                                            state.items.len(),
                                                            locale,
                                                        )),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(heading_subtitle),
                                        ),
                                ),
                        )
                        .child(
                            directional_row(h_flex(), rtl)
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_semibold()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(dict.browse.sort),
                                )
                                .child(
                                    TabBar::new(sort_tab_id)
                                        .segmented()
                                        .selected_index(selected_sort_ix)
                                        .child(Tab::new().label(dict.browse.sort_latest))
                                        .child(Tab::new().label(dict.browse.sort_imdb))
                                        .on_click(cx.listener(move |this, ix: &usize, _, cx| {
                                            let next_sort = if *ix == 1 {
                                                BrowseSort::Imdb
                                            } else {
                                                BrowseSort::Created
                                            };
                                            this.set_catalog_sort(category, next_sort, cx);
                                        })),
                                )
                                .when(
                                    state.genre_id > 0 || state.sort != BrowseSort::Created,
                                    |row| {
                                        row.child(
                                            Button::new("catalog-reset-filters")
                                                .small()
                                                .outline()
                                                .icon(IconName::Undo2)
                                                .label(dict.browse.clear_filters)
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.reset_catalog_filters(category, cx);
                                                })),
                                        )
                                    },
                                ),
                        ),
                )
                // Genre Filter Pills
                .when(!genre_buttons.is_empty(), |col| {
                    col.child(
                        directional_row(h_flex(), rtl)
                            .w_full()
                            .flex_wrap()
                            .gap_2()
                            .children(genre_buttons),
                    )
                }),
        )
        // Error Banner
        .when_some(state.error.clone(), |col, err| {
            col.child(
                v_flex()
                    .w_full()
                    .gap_2()
                    .child(Alert::error("catalog-error", err).title(dict.browse.load_error))
                    .child(
                        directional_row(h_flex(), rtl).child(
                            Button::new("catalog-retry")
                                .small()
                                .outline()
                                .icon(IconName::RefreshCw)
                                .label(dict.actions.retry)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.reload_catalog(category, cx);
                                })),
                        ),
                    ),
            )
        })
        // Grid Body
        .child(if state.loading && cards.is_empty() {
            h_flex()
                .w_full()
                .flex_wrap()
                .gap_4()
                .children((0..12).map(move |_| {
                    Skeleton::new()
                        .w(px(196.))
                        .h(px(294.))
                        .rounded(radius_lg)
                }))
                .into_any_element()
        } else if cards.is_empty() {
            Empty::new()
                .header(
                    EmptyHeader::new()
                        .media(
                            EmptyMedia::new()
                                .with_variant(EmptyMediaVariant::Icon)
                                .child(Icon::new(IconName::GalleryVerticalEnd)),
                        )
                        .title(EmptyTitle::new().child(dict.browse.empty))
                        .description(EmptyDescription::new().child(dict.browse.empty_hint)),
                )
                .into_any_element()
        } else {
            v_flex()
                .w_full()
                .gap_6()
                .child(
                    h_flex()
                        .w_full()
                        .flex_wrap()
                        .gap_4()
                        .when(rtl, |this| this.flex_row_reverse())
                        .children(cards),
                )
                .when(state.has_more, |col| {
                    col.child(
                        h_flex().w_full().justify_center().pt_2().child(
                            Button::new("catalog-load-more")
                                .primary()
                                .large()
                                .loading(state.loading_more)
                                .icon(IconName::Plus)
                                .label(if state.loading_more {
                                    dict.browse.loading_more
                                } else {
                                    dict.browse.load_more
                                })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.load_more_catalog(category, cx);
                                })),
                        ),
                    )
                })
                .into_any_element()
        })
        .into_any_element()
}
