//! Home discovery: spotlight and horizontal rails.

use gpui_kit::component::{
    ActiveTheme as _, IconName, Sizable as _, StyledExt as _, alert::Alert, button::Button, h_flex,
    skeleton::Skeleton, v_flex,
};
use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};

use crate::{
    app::StreamovieApp,
    i18n::directional_row,
    models::{ApiMedia, BrowseSort, MediaCategory},
    state::NavSection,
    ui::components::{PosterCardStyle, render_hero_banner, render_poster_card_with_style},
};

pub fn render_home_view(app: &StreamovieApp, cx: &mut Context<StreamovieApp>) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let home = app.home_state();

    if home.loading && home.latest_movies.is_empty() && home.latest_series.is_empty() {
        return v_flex()
            .w_full()
            .gap_6()
            .child(Skeleton::new().w_full().h(px(560.)))
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

    let hero = if !home.spotlight.is_empty() {
        let active_ix = home
            .spotlight_index
            .min(home.spotlight.len().saturating_sub(1));
        let active_id = home.spotlight[active_ix].media.id;
        render_hero_banner(
            app,
            &home.spotlight,
            active_ix,
            locale,
            app.is_bookmarked(active_id),
            app.is_liked(active_id),
            cx,
        )
    } else if let Some(err) = home.error.clone() {
        v_flex()
            .px_8()
            .pt_8()
            .gap_3()
            .child(Alert::error("home-load-error", err).title(dict.home.unavailable_title))
            .child(
                Button::new("home-retry")
                    .outline()
                    .icon(IconName::RefreshCw)
                    .label(dict.actions.retry)
                    .on_click(cx.listener(|this, _, _, cx| this.reload_home_feeds(cx))),
            )
            .into_any_element()
    } else {
        div().h(px(48.)).into_any_element()
    };

    v_flex()
        .w_full()
        .gap_8()
        .pb_16()
        .child(hero)
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

    v_flex()
        .w_full()
        .px_8()
        .gap_3()
        .child(
            directional_row(h_flex(), rtl)
                .w_full()
                .justify_between()
                .items_center()
                .child(div().text_lg().font_semibold().child(title.to_string()))
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
                .overflow_x_scroll()
                .track_scroll(&scroll_handle)
                .child(
                    h_flex()
                        .gap_4()
                        .pb_2()
                        .when(rtl, |row| row.flex_row_reverse())
                        .children(cards),
                )
                .into_any_element()
        })
        .into_any_element()
}
