//! Streamovie Home discovery: cinematic hero, discovery shortcuts, and catalog rails.

use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, hsla, prelude::FluentBuilder as _, px,
    white,
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
    let radius = cx.theme().radius; // 8px

    if home.loading && home.latest_movies.is_empty() && home.latest_series.is_empty() {
        return v_flex()
            .w_full()
            .gap_8()
            .child(Skeleton::new().w_full().h(px(520.)).rounded(radius))
            .child(render_skeleton_row(radius, PosterCardStyle::Widescreen))
            .child(render_skeleton_row(radius, PosterCardStyle::Poster))
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

    // Discovery shortcuts
    let brand_hubs_rail = render_discovery_hubs(app, cx);

    // Carousels
    let trending_shows_sec = render_carousel_section(
        app,
        0,
        "trending-shows",
        "TRENDING SHOWS FOR YOU",
        dict.home.series_subtitle,
        &home.latest_series,
        MediaCategory::Series,
        NavSection::Series,
        BrowseSort::Created,
        PosterCardStyle::Widescreen,
        cx,
    );
    let blockbuster_movies_sec = render_carousel_section(
        app,
        1,
        "blockbuster-movies",
        "BLOCKBUSTER MOVIES",
        dict.home.movies_subtitle,
        &home.latest_movies,
        MediaCategory::Movie,
        NavSection::Movies,
        BrowseSort::Created,
        PosterCardStyle::Poster,
        cx,
    );
    let top_rated_movies_sec = render_carousel_section(
        app,
        2,
        "top-rated-movies",
        "CRITICALLY ACCLAIMED MOVIES",
        dict.home.imdb_movies_subtitle,
        &home.imdb_movies,
        MediaCategory::Movie,
        NavSection::Movies,
        BrowseSort::Imdb,
        PosterCardStyle::Poster,
        cx,
    );
    let top_rated_series_sec = render_carousel_section(
        app,
        3,
        "binge-worthy-series",
        "BINGE-WORTHY TV SERIES",
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
        .pb_12()
        .child(hero)
        .child(brand_hubs_rail)
        .child(trending_shows_sec)
        .child(blockbuster_movies_sec)
        .child(top_rated_movies_sec)
        .child(top_rated_series_sec)
        .when(rtl, |this| this.items_end())
        .into_any_element()
}

/// Streamovie discovery shortcuts (no third-party brand hubs).
fn render_discovery_hubs(
    app: &StreamovieApp,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let primary = cx.theme().primary;

    let hubs = [
        (dict.nav.series, "ORIGINALS", NavSection::Series),
        (dict.home.trending, "FOR YOU", NavSection::Series),
        (dict.nav.top_rated, "IMDB", NavSection::Movies),
        (dict.nav.movies, "CINEMA", NavSection::Movies),
        (dict.nav.search, "FIND", NavSection::Search),
    ];

    let mut hub_cards = Vec::with_capacity(hubs.len());
    for (ix, (brand_title, tag, target_nav)) in hubs.iter().enumerate() {
        let hub_id = SharedString::from(format!("streamovie-hub-{ix}"));
        let nav = *target_nav;

        hub_cards.push(
            div()
                .id(hub_id)
                .flex_1()
                .h(px(64.))
                .min_w(px(140.))
                .rounded(px(8.))
                .bg(hsla(220.0 / 360.0, 0.38, 0.08, 0.90))
                .border_1()
                .border_color(hsla(216.0 / 360.0, 0.40, 0.85, 0.12))
                .cursor_pointer()
                .hover(move |s| {
                    s.border_color(primary)
                        .bg(hsla(216.0 / 360.0, 0.60, 0.14, 0.90))
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.select_section(nav, cx);
                }))
                .child(
                    v_flex()
                        .size_full()
                        .items_center()
                        .justify_center()
                        .gap_1()
                        .child(
                            div()
                                .text_base()
                                .font_extrabold()
                                .text_color(white())
                                .child(*brand_title),
                        )
                        .child(
                            div()
                                .text_size(px(9.5))
                                .font_bold()
                                .text_color(hsla(216.0 / 360.0, 1.0, 0.65, 1.0))
                                .child(*tag),
                        ),
                ),
        );
    }

    v_flex()
        .w_full()
        .gap_3()
        .child(
            directional_row(h_flex(), rtl)
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w(px(3.))
                        .h(px(14.))
                        .rounded_full()
                        .bg(primary),
                )
                .child(
                    div()
                        .text_xs()
                        .font_extrabold()
                        .text_color(hsla(215.0 / 360.0, 0.20, 0.75, 1.0))
                        .child("DISCOVER"),
                ),
        )
        .child(
            directional_row(h_flex(), rtl)
                .w_full()
                .gap_3()
                .children(hub_cards),
        )
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
    let scroll_left_id = SharedString::from(format!("streamovie-prev-{section_key}"));
    let scroll_right_id = SharedString::from(format!("streamovie-next-{section_key}"));
    let track_id = SharedString::from(format!("streamovie-track-{section_key}"));
    let scroll_handle = app.home_row_scroll_handle(row_index).clone();

    let visible_items: Vec<ApiMedia> = items.iter().take(18).cloned().collect();
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
        // Section header
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
                                .w(px(3.5))
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
                                                .font_extrabold()
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
                                .label(dict.actions.see_more.to_uppercase())
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.navigate_to_catalog_with_sort(target_nav, target_sort, cx);
                                })),
                        ),
                ),
        )
        // Carousel track
        .child(if cards.is_empty() {
            div()
                .w_full()
                .p_6()
                .rounded(cx.theme().radius)
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
                        .pt_2()
                        .pb_3()
                        .when(rtl, |this| this.flex_row_reverse())
                        .children(cards),
                )
                .into_any_element()
        })
        .into_any_element()
}

fn render_skeleton_row(radius: gpui_kit::Pixels, style: PosterCardStyle) -> AnyElement {
    let (w, h) = match style {
        PosterCardStyle::Poster => (px(192.), px(288.)),
        PosterCardStyle::Widescreen => (px(288.), px(162.)),
    };
    h_flex()
        .w_full()
        .gap_4()
        .overflow_hidden()
        .children((0..6).map(move |_| Skeleton::new().w(w).h(h).rounded(radius)))
        .into_any_element()
}
