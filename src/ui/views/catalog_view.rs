//! List / Catalog screen completely redesigned to match Streamovie Browse UI/UX.
//!
//! Features:
//! - Streamovie high-impact catalog header with section title, subtitle, and results counter
//! - Streamovie dark glass filter surface with Sort pills (Latest / IMDb Top Rated) and Genre pills
//! - Clean multi-column grid of Streamovie poster cards with hover elevation and luminous glow
//! - Streamovie "LOAD MORE" CTA button

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
    let radius = cx.theme().radius; // 8px

    let heading_title = match category {
        MediaCategory::Movie => "MOVIES",
        MediaCategory::Series => "TV SHOWS",
    };
    let heading_subtitle = match category {
        MediaCategory::Movie => "Explore blockbuster movies, exclusive premieres, and iconic cinema classics.",
        MediaCategory::Series => "Stream full seasons, hit original series, and binge-worthy drama in 4K UHD.",
    };

    let selected_sort_ix = match state.sort {
        BrowseSort::Created => 0,
        BrowseSort::Imdb => 1,
    };

    let sort_tab_id = SharedString::from(format!("streamovie-catalog-sort-{}", category.as_str()));
    let scope_key = format!("catalog-{}", category.as_str());

    let mut genre_buttons = Vec::new();
    if !genres.is_empty() {
        let active_genre = state.genre_id;
        genre_buttons.push(
            Button::new("streamovie-genre-all")
                .small()
                .selected(active_genre == 0)
                .when(active_genre == 0, |btn| btn.primary())
                .when(active_genre != 0, |btn| btn.outline())
                .label(dict.browse.all_genres.to_uppercase())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_catalog_genre(category, 0, cx);
                })),
        );
        for genre in genres.into_iter().take(18) {
            let gid = genre.id;
            let is_selected = active_genre == gid;
            genre_buttons.push(
                Button::new(SharedString::from(format!("streamovie-genre-{gid}")))
                    .small()
                    .selected(is_selected)
                    .when(is_selected, |btn| btn.primary())
                    .when(!is_selected, |btn| btn.outline())
                    .label(genre.display_name().to_uppercase())
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
        .pb_12()
        // Streamovie Catalog Header & Filter Surface
        .child(
            v_flex()
                .w_full()
                .p_5()
                .gap_4()
                .rounded(px(10.))
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
                                        .w(px(3.5))
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
                                                        .font_extrabold()
                                                        .child(heading_title),
                                                )
                                                .child(
                                                    Tag::primary()
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
                        // Streamovie Sort Tabs
                        .child(
                            directional_row(h_flex(), rtl)
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_bold()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("SORT:"),
                                )
                                .child(
                                    TabBar::new(sort_tab_id)
                                        .pill()
                                        .selected_index(selected_sort_ix)
                                        .child(Tab::new().label("RECENTLY ADDED"))
                                        .child(Tab::new().label("TOP RATED (IMDb)"))
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
                                            Button::new("streamovie-catalog-clear")
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
                    .child(Alert::error("streamovie-catalog-error", err).title(dict.browse.load_error))
                    .child(
                        directional_row(h_flex(), rtl).child(
                            Button::new("streamovie-catalog-retry")
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
        // Catalog Grid Body
        .child(if state.loading && cards.is_empty() {
            h_flex()
                .w_full()
                .flex_wrap()
                .gap_4()
                .children((0..12).map(move |_| {
                    Skeleton::new()
                        .w(px(192.))
                        .h(px(288.))
                        .rounded(radius)
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
                        .pt_2()
                        .pb_2()
                        .when(rtl, |this| this.flex_row_reverse())
                        .children(cards),
                )
                .when(state.has_more, |col| {
                    col.child(
                        h_flex().w_full().justify_center().pt_3().child(
                            Button::new("streamovie-catalog-load-more")
                                .primary()
                                .large()
                                .loading(state.loading_more)
                                .icon(IconName::Plus)
                                .label(if state.loading_more {
                                    "LOADING TITLES...".to_string()
                                } else {
                                    "LOAD MORE TITLES".to_string()
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
