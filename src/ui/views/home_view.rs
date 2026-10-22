//! Home discovery view (`Home`) redesigned with an Infuse 8 + Apple TV + Netflix aesthetic.
//! Features:
//! - Full-bleed `450px` cinematic `HeroBanner` with interactive widescreen spotlight selector
//! - Four Netflix-style horizontal scrolling carousels (`ScrollHandle` + `<` / `>` scroll buttons):
//!   1. Latest Movies (`2:3` Infuse/Apple TV posters)
//!   2. Latest Series (`16:9` Netflix/Apple TV widescreen cards)
//!   3. IMDb Top Rated Movies (`2:3` Infuse/Apple TV posters)
//!   4. IMDb Top Rated Series (`2:3` Infuse/Apple TV posters)

use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};
use gpui_kit::component::{
    ActiveTheme as _, IconName, Sizable as _, StyledExt as _,
    alert::Alert,
    button::{Button, ButtonVariants as _},
    h_flex,
    skeleton::Skeleton,
    tag::Tag,
    v_flex,
};

use crate::{
    app::StreamovieApp,
    i18n::{directional_row, format_count},
    models::{ApiMedia, BrowseSort, MediaCategory},
    state::NavSection,
    ui::components::{PosterCardStyle, render_hero_banner, render_poster_card_with_style},
};

pub fn render_home_view(
    app: &StreamovieApp,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let home = app.home_state();
    let radius_lg = cx.theme().radius_lg;

    if home.loading && home.latest_movies.is_empty() && home.latest_series.is_empty() {
        return v_flex()
            .w_full()
            .gap_8()
            .child(Skeleton::new().w_full().h(px(450.)).rounded(radius_lg))
            .child(render_skeleton_row(radius_lg, PosterCardStyle::Poster))
            .child(render_skeleton_row(radius_lg, PosterCardStyle::Widescreen))
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
            .w_full()
            .gap_2()
            .child(Alert::error("home-load-error", err).title(dict.home.unavailable_title))
            .child(
                directional_row(h_flex(), rtl).child(
                    Button::new("home-retry-btn")
                        .small()
                        .outline()
                        .icon(IconName::RefreshCw)
                        .label(dict.actions.retry)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.reload_home_feeds(cx);
                        })),
                ),
            )
            .into_any_element()
    } else {
        div().into_any_element()
    };

    let latest_movies_sec = render_carousel_section(
        app,
        0,
        "latest-movies",
        dict.home.movies,
        dict.home.movies_subtitle,
        &home.latest_movies,
        MediaCategory::Movie,
        NavSection::Movies,
        BrowseSort::Created,
        PosterCardStyle::Poster,
        cx,
    );
    let latest_series_sec = render_carousel_section(
        app,
        1,
        "latest-series",
        dict.home.series_row,
        dict.home.series_subtitle,
        &home.latest_series,
        MediaCategory::Series,
        NavSection::Series,
        BrowseSort::Created,
        PosterCardStyle::Widescreen,
        cx,
    );
    let imdb_movies_sec = render_carousel_section(
        app,
        2,
        "imdb-movies",
        dict.home.imdb_movies,
        dict.home.imdb_movies_subtitle,
        &home.imdb_movies,
        MediaCategory::Movie,
        NavSection::Movies,
        BrowseSort::Imdb,
        PosterCardStyle::Poster,
        cx,
    );
    let imdb_series_sec = render_carousel_section(
        app,
        3,
        "imdb-series",
        dict.home.imdb_series,
        dict.home.imdb_series_subtitle,
        &home.imdb_series,
        MediaCategory::Series,
        NavSection::Series,
        BrowseSort::Imdb,
        PosterCardStyle::Poster,
        cx,
    );

    v_flex()
        .w_full()
        .gap_8()
        .pb_10()
        .child(hero)
        .child(latest_movies_sec)
        .child(latest_series_sec)
        .child(imdb_movies_sec)
        .child(imdb_series_sec)
        .when(rtl, |this| this.items_end())
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_carousel_section(
    app: &StreamovieApp,
    row_index: usize,
    section_key: &str,
    title: &str,
    subtitle: &str,
    items: &[ApiMedia],
    category: MediaCategory,
    target_nav: NavSection,
    target_sort: BrowseSort,
    card_style: PosterCardStyle,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let count_badge = format_count(dict.home.titles_count, items.len(), locale);
    let see_more_id = SharedString::from(format!("see-more-{section_key}"));
    let scroll_left_id = SharedString::from(format!("carousel-prev-{section_key}"));
    let scroll_right_id = SharedString::from(format!("carousel-next-{section_key}"));
    let track_id = SharedString::from(format!("carousel-track-{section_key}"));
    let scroll_handle = app.home_row_scroll_handle(row_index).clone();

    let visible_items: Vec<ApiMedia> = items.iter().take(16).cloned().collect();
    let mut cards = Vec::with_capacity(visible_items.len());
    for media in &visible_items {
        cards.push(render_poster_card_with_style(
            app,
            media,
            category,
            app.is_bookmarked(media.id),
            locale,
            section_key,
            card_style,
            cx,
        ));
    }

    v_flex()
        .w_full()
        .gap_3p5()
        // Section Header: Accent bar + Title + Count Pill + Scroll Controls + See More
        .child(
            directional_row(h_flex(), rtl)
                .w_full()
                .justify_between()
                .items_center()
                .gap_4()
                .child(
                    directional_row(h_flex(), rtl)
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .w(px(4.))
                                .h(px(22.))
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
                                        .gap_2p5()
                                        .child(
                                            div()
                                                .text_lg()
                                                .font_bold()
                                                .text_color(cx.theme().foreground)
                                                .child(title.to_string()),
                                        )
                                        .child(
                                            Tag::secondary().small().outline().child(count_badge),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(subtitle.to_string()),
                                ),
                        ),
                )
                .child(
                    directional_row(h_flex(), rtl)
                        .items_center()
                        .gap_2()
                        .child(
                            Button::new(scroll_left_id)
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
                            Button::new(scroll_right_id)
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
                            Button::new(see_more_id)
                                .small()
                                .ghost()
                                .icon(if rtl {
                                    IconName::ArrowLeft
                                } else {
                                    IconName::ArrowRight
                                })
                                .label(dict.actions.see_more)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.navigate_to_catalog_with_sort(target_nav, target_sort, cx);
                                })),
                        ),
                ),
        )
        // Horizontal Carousel Track
        .child(if cards.is_empty() {
            div()
                .w_full()
                .p_6()
                .rounded(cx.theme().radius_lg)
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().popover)
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(dict.home.empty)
                .into_any_element()
        } else {
            div()
                .id(track_id)
                .w_full()
                .overflow_x_scroll()
                .track_scroll(&scroll_handle)
                .child(
                    h_flex()
                        .gap_4()
                        .py_1()
                        .when(rtl, |this| this.flex_row_reverse())
                        .children(cards),
                )
                .into_any_element()
        })
        .into_any_element()
}

fn render_skeleton_row(radius_lg: gpui_kit::Pixels, style: PosterCardStyle) -> AnyElement {
    let (w, h) = match style {
        PosterCardStyle::Poster => (px(196.), px(294.)),
        PosterCardStyle::Widescreen => (px(292.), px(166.)),
    };
    h_flex()
        .w_full()
        .gap_4()
        .overflow_hidden()
        .children((0..6).map(move |_| Skeleton::new().w(w).h(h).rounded(radius_lg)))
        .into_any_element()
}
