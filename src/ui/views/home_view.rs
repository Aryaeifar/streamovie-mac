//! Home discovery: horizontal rails, promo banners, and browse shelves.

use std::sync::{Arc, OnceLock};

use gpui_kit::component::{
    IconName, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    AnyElement, Context, Image, ImageFormat, InteractiveElement as _, IntoElement, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ObjectFit, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, StyledImage as _, div, hsla, img,
    linear_color_stop, linear_gradient, prelude::FluentBuilder as _, px, white,
};

use crate::{
    api::streamovie::TOP250_GENRE_ID,
    app::StreamovieApp,
    i18n::{directional_row, is_persian_text},
    models::{
        ApiMedia, BrowseSort, MediaCategory, SavedMediaEntry,
        imdb_top250::chart_rank,
    },
    state::NavSection,
    ui::{
        apple::ApplePaint,
        components::{
            PosterCardStyle, WATCHLIST_CARD_W, empty_state, home_skeleton, network_error_state,
            render_media_card, render_watchlist_card, skeleton_row,
        },
        image_cache::render_remote_image,
    },
};

const CONTINUE_ROW: usize = 0;
const SERIES_ROW: usize = 1;
const MOVIES_ROW: usize = 2;
const WATCHLIST_ROW: usize = 3;
const IMDB_MOVIES_ROW: usize = 4;
const IMDB_SERIES_ROW: usize = 5;
const GENRE_ROW: usize = 6;
const COLLECTION_ROW: usize = 7;

const TILE_W: f32 = 188.;
const TILE_H: f32 = 250.;
const TILE_GAP: f32 = 16.;
const COLLECTION_W: f32 = 340.;
const COLLECTION_H: f32 = 420.;
const COLLECTION_RADIUS: f32 = 22.;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SeeMore {
    None,
    Catalog(NavSection, BrowseSort),
    Chart(MediaCategory),
    Section(NavSection),
}

struct CardSpec<'a> {
    media: &'a ApiMedia,
    category: MediaCategory,
    rank: Option<u16>,
    progress: Option<f32>,
    show_bookmark: bool,
}

#[derive(Clone, Copy)]
enum ShelfTarget {
    Chart(MediaCategory),
    Latest(NavSection, BrowseSort),
}

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

    let continue_specs: Vec<CardSpec> = app
        .continue_items()
        .iter()
        .map(|entry| CardSpec {
            media: &entry.media,
            category: entry.kind,
            rank: None,
            progress: Some(entry.progress),
            show_bookmark: true,
        })
        .collect();
    let saved = {
        let mut items = app.ordered_bookmarks();
        items.reverse();
        items
    };
    let genres: Vec<_> = app
        .genres()
        .iter()
        .filter(|genre| genre.id != 0 && genre.id != TOP250_GENRE_ID)
        .cloned()
        .collect();

    v_flex()
        .w_full()
        .min_w_0()
        .max_w_full()
        .overflow_hidden()
        .gap(px(36.))
        .pt(px(8.))
        .pb(px(56.))
        .when(!continue_specs.is_empty(), |col| {
            col.child(media_rail(
                app,
                CONTINUE_ROW,
                "continue",
                dict.home.continue_watching,
                &continue_specs,
                None,
                "",
                false,
                SeeMore::None,
                PosterCardStyle::Widescreen,
                rtl,
                cx,
            ))
        })
        .child(single_rail(
            app,
            SERIES_ROW,
            "latest-series",
            dict.home.series_row,
            &home.latest_series,
            home.latest_series_error.as_deref(),
            dict.common.series_failed,
            home.loading,
            MediaCategory::Series,
            SeeMore::Catalog(NavSection::Series, BrowseSort::Created),
            PosterCardStyle::Widescreen,
            Some(18),
            false,
            rtl,
            cx,
        ))
        .child(promo_banner(
            app,
            "banner-top-movies",
            dict.home.imdb_movies,
            dict.home.banner_movies_body,
            dict.actions.see_more,
            movies_banner_art(),
            ShelfTarget::Chart(MediaCategory::Movie),
            rtl,
            cx,
        ))
        .child(single_rail(
            app,
            MOVIES_ROW,
            "latest-movies",
            dict.home.movies,
            &home.latest_movies,
            home.latest_movies_error.as_deref(),
            dict.common.movies_failed,
            home.loading,
            MediaCategory::Movie,
            SeeMore::Catalog(NavSection::Movies, BrowseSort::Created),
            PosterCardStyle::Poster,
            Some(18),
            false,
            rtl,
            cx,
        ))
        .when(!saved.is_empty(), |col| {
            col.child(watchlist_rail(app, &saved, rtl, cx))
        })
        .child(promo_banner(
            app,
            "banner-top-series",
            dict.home.imdb_series,
            dict.home.banner_series_body,
            dict.actions.see_more,
            series_banner_art(),
            ShelfTarget::Chart(MediaCategory::Series),
            rtl,
            cx,
        ))
        .child(single_rail(
            app,
            IMDB_MOVIES_ROW,
            "imdb-movies",
            dict.home.imdb_movies,
            &home.imdb_movies,
            home.imdb_movies_error.as_deref(),
            dict.common.movies_failed,
            home.loading,
            MediaCategory::Movie,
            SeeMore::Chart(MediaCategory::Movie),
            PosterCardStyle::Poster,
            None,
            true,
            rtl,
            cx,
        ))
        .child(single_rail(
            app,
            IMDB_SERIES_ROW,
            "imdb-series",
            dict.home.imdb_series,
            &home.imdb_series,
            home.imdb_series_error.as_deref(),
            dict.common.series_failed,
            home.loading,
            MediaCategory::Series,
            SeeMore::Chart(MediaCategory::Series),
            PosterCardStyle::Poster,
            None,
            true,
            rtl,
            cx,
        ))
        .when(!genres.is_empty(), |col| {
            col.child(genre_rail(app, &genres, rtl, cx))
        })
        .child(collection_rail(app, rtl, cx))
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn single_rail(
    app: &StreamovieApp,
    row_index: usize,
    section_key: &str,
    title: &str,
    items: &[ApiMedia],
    fetch_error: Option<&str>,
    failed_title: &str,
    loading: bool,
    category: MediaCategory,
    see_more: SeeMore,
    card_style: PosterCardStyle,
    item_limit: Option<usize>,
    numbered: bool,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let visible = match item_limit {
        Some(limit) => &items[..items.len().min(limit)],
        None => items,
    };
    let specs: Vec<CardSpec> = visible
        .iter()
        .enumerate()
        .map(|(index, media)| CardSpec {
            media,
            category,
            rank: if numbered {
                chart_rank(media, category).or(Some((index + 1) as u16))
            } else {
                None
            },
            progress: None,
            show_bookmark: !numbered,
        })
        .collect();
    media_rail(
        app,
        row_index,
        section_key,
        title,
        &specs,
        fetch_error,
        failed_title,
        loading,
        see_more,
        card_style,
        rtl,
        cx,
    )
}

#[allow(clippy::too_many_arguments)]
fn media_rail(
    app: &StreamovieApp,
    row_index: usize,
    section_key: &str,
    title: &str,
    specs: &[CardSpec],
    fetch_error: Option<&str>,
    failed_title: &str,
    loading: bool,
    see_more: SeeMore,
    card_style: PosterCardStyle,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let locale = app.locale();
    let (card_w, _) = card_style.frame();
    let card_gap = px(20.);
    let cards: Vec<_> = specs
        .iter()
        .map(|spec| {
            render_media_card(
                app,
                spec.media,
                spec.category,
                app.is_bookmarked(spec.media.id),
                locale,
                section_key,
                card_style,
                spec.rank,
                spec.progress,
                spec.show_bookmark,
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

    let body = if cards.is_empty() && loading {
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
        horizontal_scroller(app, row_index, section_key, row_w, card_gap, rtl, cards, cx)
    };

    shelf_frame(app, row_index, section_key, title, see_more, rtl, body, cx)
}

fn genre_rail(
    app: &StreamovieApp,
    genres: &[crate::models::ApiGenre],
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let paint = ApplePaint::from_cx(cx);
    let tiles: Vec<_> = genres
        .iter()
        .map(|genre| {
            let gid = genre.id;
            let name = genre.display_name();
            let name_rtl = is_persian_text(&name) || rtl;
            let tile_id = SharedString::from(format!("genre-tile-{gid}"));
            let hover_id = tile_id.clone();
            let hovered = app.control_is_hovered(&tile_id);
            let letter = name.chars().next().unwrap_or('•').to_string();
            div()
                .id(tile_id)
                .w(px(TILE_W))
                .h(px(TILE_H))
                .flex_none()
                .relative()
                .rounded(px(18.))
                .overflow_hidden()
                .cursor_pointer()
                .bg(genre_fill(gid))
                .border_1()
                .border_color(if hovered { white().opacity(0.7) } else { paint.hairline })
                .shadow(paint.poster_shadow(hovered))
                .on_hover(cx.listener(move |this, hovered, _, cx| {
                    this.set_control_hovered(&hover_id, *hovered, cx);
                }))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.open_genre(gid, cx);
                }))
                .child(
                    div()
                        .absolute()
                        .top(px(18.))
                        .when(!rtl, |label| label.left(px(16.)))
                        .when(rtl, |label| label.right(px(16.)))
                        .text_size(px(64.))
                        .font_semibold()
                        .text_color(white().opacity(0.28))
                        .child(letter),
                )
                .child(
                    div()
                        .absolute()
                        .bottom_0()
                        .left_0()
                        .right_0()
                        .h(px(120.))
                        .rounded_b(px(18.))
                        .bg(linear_gradient(
                            180.,
                            linear_color_stop(hsla(0.0, 0.0, 0.0, 0.0), 0.),
                            linear_color_stop(hsla(0.0, 0.0, 0.0, 0.72), 1.),
                        )),
                )
                .child(
                    div()
                        .absolute()
                        .bottom(px(16.))
                        .left(px(16.))
                        .right(px(16.))
                        .text_size(px(18.))
                        .font_semibold()
                        .text_color(white())
                        .line_height(gpui_kit::relative(1.2))
                        .when(name_rtl, |label| label.text_right())
                        .child(name),
                )
                .into_any_element()
        })
        .collect();
    let count = tiles.len() as f32;
    let row_w = px(TILE_W) * count + px(TILE_GAP) * (count - 1.);
    let body = horizontal_scroller(app, GENRE_ROW, "genres", row_w, px(TILE_GAP), rtl, tiles, cx);
    shelf_frame(
        app,
        GENRE_ROW,
        "genres",
        dict.home.browse_genre,
        SeeMore::None,
        rtl,
        body,
        cx,
    )
}

fn watchlist_rail(
    app: &StreamovieApp,
    saved: &[SavedMediaEntry],
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let cards: Vec<_> = saved
        .iter()
        .enumerate()
        .map(|(index, entry)| render_watchlist_card(app, &entry.media, entry.kind, index, cx))
        .collect();
    let count = cards.len() as f32;
    let gap = px(16.);
    let row_w = px(WATCHLIST_CARD_W) * count + gap * (count - 1.);
    let body = horizontal_scroller(app, WATCHLIST_ROW, "watchlist", row_w, gap, rtl, cards, cx);
    shelf_frame(
        app,
        WATCHLIST_ROW,
        "watchlist",
        dict.home.watchlist_row,
        SeeMore::Section(NavSection::Bookmarks),
        rtl,
        body,
        cx,
    )
}

fn collection_rail(
    app: &StreamovieApp,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let home = app.home_state();
    let collections = [
        (
            "collection-top-movies",
            dict.home.imdb_movies,
            dict.home.imdb_movies_subtitle,
            art_pair(home.imdb_movies.first()),
            ShelfTarget::Chart(MediaCategory::Movie),
            IconName::GalleryVerticalEnd,
        ),
        (
            "collection-top-series",
            dict.home.imdb_series,
            dict.home.imdb_series_subtitle,
            art_pair(home.imdb_series.first()),
            ShelfTarget::Chart(MediaCategory::Series),
            IconName::Frame,
        ),
        (
            "collection-latest-movies",
            dict.home.movies,
            dict.home.movies_subtitle,
            art_pair(home.latest_movies.first()),
            ShelfTarget::Latest(NavSection::Movies, BrowseSort::Created),
            IconName::GalleryVerticalEnd,
        ),
        (
            "collection-latest-series",
            dict.home.series_row,
            dict.home.series_subtitle,
            art_pair(home.latest_series.first()),
            ShelfTarget::Latest(NavSection::Series, BrowseSort::Created),
            IconName::Frame,
        ),
    ];
    let paint = ApplePaint::from_cx(cx);
    let radius = px(COLLECTION_RADIUS);
    let tiles: Vec<_> = collections
        .into_iter()
        .map(|(key, title, subtitle, (primary, fallback), target, icon)| {
            let tile_id = SharedString::from(key);
            let hover_id = tile_id.clone();
            let hovered = app.control_is_hovered(&tile_id);
            let title_rtl = is_persian_text(title) || rtl;
            div()
                .id(tile_id)
                .w(px(COLLECTION_W))
                .h(px(COLLECTION_H))
                .flex_none()
                .relative()
                .rounded(radius)
                .overflow_hidden()
                .cursor_pointer()
                .bg(paint.chip)
                .border_1()
                .border_color(if hovered { white().opacity(0.75) } else { paint.hairline })
                .shadow(paint.poster_shadow(hovered))
                .on_hover(cx.listener(move |this, hovered, _, cx| {
                    this.set_control_hovered(&hover_id, *hovered, cx);
                }))
                .on_click(cx.listener(move |this, _, _, cx| open_shelf(this, target, cx)))
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded(radius)
                        .overflow_hidden()
                        .bg(paint.chip)
                        .child(render_remote_image(
                            app,
                            primary.as_deref(),
                            fallback.as_deref(),
                            radius,
                            icon,
                            None,
                            cx,
                        )),
                )
                .child(
                    div()
                        .absolute()
                        .bottom_0()
                        .left_0()
                        .right_0()
                        .h(px(210.))
                        .rounded_b(radius)
                        .bg(linear_gradient(
                            180.,
                            linear_color_stop(hsla(0.0, 0.0, 0.0, 0.0), 0.),
                            linear_color_stop(hsla(0.0, 0.0, 0.0, 0.92), 1.),
                        )),
                )
                .child(
                    v_flex()
                        .absolute()
                        .bottom(px(20.))
                        .left(px(20.))
                        .right(px(20.))
                        .gap(px(6.))
                        .when(title_rtl, |col| col.items_end())
                        .child(
                            div()
                                .w_full()
                                .text_size(px(26.))
                                .font_semibold()
                                .line_height(gpui_kit::relative(1.1))
                                .text_color(white())
                                .when(title_rtl, |label| label.text_right())
                                .child(title.to_string()),
                        )
                        .child(
                            div()
                                .w_full()
                                .text_size(px(13.))
                                .text_color(white().opacity(0.82))
                                .when(title_rtl, |label| label.text_right())
                                .child(subtitle.to_string()),
                        ),
                )
                .into_any_element()
        })
        .collect();
    let count = tiles.len() as f32;
    let row_w = px(COLLECTION_W) * count + px(20.) * (count - 1.);
    let body = horizontal_scroller(
        app,
        COLLECTION_ROW,
        "collections",
        row_w,
        px(20.),
        rtl,
        tiles,
        cx,
    );
    shelf_frame(
        app,
        COLLECTION_ROW,
        "collections",
        dict.home.browse_collection,
        SeeMore::None,
        rtl,
        body,
        cx,
    )
}

fn promo_banner(
    app: &StreamovieApp,
    key: &str,
    title: &str,
    body: &str,
    action: &str,
    art: Arc<Image>,
    target: ShelfTarget,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let paint = ApplePaint::from_cx(cx);
    let title_rtl = is_persian_text(title) || rtl;
    let banner_id = SharedString::from(key);
    let hover_id = banner_id.clone();
    let hovered = app.control_is_hovered(&banner_id);

    div()
        .px(px(32.))
        .child(
            div()
                .id(banner_id)
                .relative()
                .w_full()
                .h(px(248.))
                .rounded(px(22.))
                .overflow_hidden()
                .cursor_pointer()
                .bg(hsla(240.0 / 360.0, 0.04, 0.12, 1.0))
                .border_1()
                .border_color(paint.hairline)
                .shadow(paint.poster_shadow(hovered))
                .on_hover(cx.listener(move |this, hovered, _, cx| {
                    this.set_control_hovered(&hover_id, *hovered, cx);
                }))
                .on_click(cx.listener(move |this, _, _, cx| open_shelf(this, target, cx)))
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded(px(22.))
                        .overflow_hidden()
                        .bg(hsla(240.0 / 360.0, 0.04, 0.12, 1.0))
                        .child(
                            div().relative().size_full().child(
                                img(art)
                                    .absolute()
                                    .inset_0()
                                    .size_full()
                                    .rounded(px(22.))
                                    .object_fit(ObjectFit::Cover),
                            ),
                        ),
                )
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded(px(22.))
                        .bg(hsla(0.0, 0.0, 0.0, 0.28)),
                )
                .child(
                    div().absolute().inset_0().rounded(px(22.)).bg(linear_gradient(
                        if rtl { 270. } else { 90. },
                        linear_color_stop(hsla(0.0, 0.0, 0.0, 0.78), 0.),
                        linear_color_stop(hsla(0.0, 0.0, 0.0, 0.08), 1.),
                    )),
                )
                .child(
                    v_flex()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .w(px(560.))
                        .justify_center()
                        .gap(px(10.))
                        .px(px(36.))
                        .when(!rtl, |col| col.left_0().items_start())
                        .when(rtl, |col| col.right_0().items_end())
                        .child(
                            div()
                                .text_size(px(13.))
                                .font_semibold()
                                .text_color(white().opacity(0.82))
                                .child(app.locale().dict().brand.name.to_string()),
                        )
                        .child(
                            div()
                                .text_size(px(36.))
                                .font_semibold()
                                .line_height(gpui_kit::relative(1.05))
                                .text_color(white())
                                .when(title_rtl, |label| label.text_right())
                                .child(title.to_string()),
                        )
                        .child(
                            div()
                                .max_w(px(420.))
                                .text_size(px(14.))
                                .text_color(white().opacity(0.86))
                                .when(title_rtl, |label| label.text_right())
                                .child(body.to_string()),
                        )
                        .child(
                            h_flex()
                                .mt(px(6.))
                                .h(px(34.))
                                .px(px(16.))
                                .rounded_full()
                                .items_center()
                                .justify_center()
                                .bg(white())
                                .text_size(px(13.))
                                .font_semibold()
                                .text_color(paint.ink)
                                .child(action.to_string()),
                        ),
                ),
        )
        .into_any_element()
}

fn shelf_frame(
    app: &StreamovieApp,
    row_index: usize,
    section_key: &str,
    title: &str,
    see_more: SeeMore,
    rtl: bool,
    body: AnyElement,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let paint = ApplePaint::from_cx(cx);
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
                        .text_color(paint.ink)
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
                        .when(see_more != SeeMore::None, |row| {
                            row.child(
                                Button::new(SharedString::from(format!("rail-more-{section_key}")))
                                    .small()
                                    .ghost()
                                    .label(dict.actions.see_more)
                                    .on_click(cx.listener(move |this, _, _, cx| match see_more {
                                        SeeMore::None => {}
                                        SeeMore::Catalog(nav, sort) => {
                                            this.navigate_to_catalog_with_sort(nav, sort, cx);
                                        }
                                        SeeMore::Chart(category) => this.open_chart(category, cx),
                                        SeeMore::Section(section) => {
                                            this.select_section(section, cx);
                                        }
                                    })),
                            )
                        }),
                ),
        )
        .child(body)
        .into_any_element()
}

fn horizontal_scroller(
    app: &StreamovieApp,
    row_index: usize,
    section_key: &str,
    row_w: gpui_kit::Pixels,
    gap: gpui_kit::Pixels,
    rtl: bool,
    children: Vec<AnyElement>,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let scroll_handle = app.home_row_scroll_handle(row_index).clone();
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
                .gap(gap)
                .py(px(16.))
                .when(rtl, |row| row.flex_row_reverse())
                .children(children),
        )
        .into_any_element()
}

fn open_shelf(app: &mut StreamovieApp, target: ShelfTarget, cx: &mut Context<StreamovieApp>) {
    match target {
        ShelfTarget::Chart(category) => app.open_chart(category, cx),
        ShelfTarget::Latest(nav, sort) => app.navigate_to_catalog_with_sort(nav, sort, cx),
    }
}

fn movies_banner_art() -> Arc<Image> {
    // Cinema auditorium, cropped to the banner strip. Unsplash photo-1485095329183.
    static IMAGE: OnceLock<Arc<Image>> = OnceLock::new();
    banner_art(
        include_bytes!("../../../assets/banners/top-movies.jpg"),
        &IMAGE,
    )
}

fn series_banner_art() -> Arc<Image> {
    // Friends watching a show, cropped to the banner strip. Pexels 23495578.
    static IMAGE: OnceLock<Arc<Image>> = OnceLock::new();
    banner_art(
        include_bytes!("../../../assets/banners/top-series.jpg"),
        &IMAGE,
    )
}

fn banner_art(bytes: &'static [u8], slot: &'static OnceLock<Arc<Image>>) -> Arc<Image> {
    slot.get_or_init(|| Arc::new(Image::from_bytes(ImageFormat::Jpeg, bytes.to_vec())))
        .clone()
}

fn art_pair(media: Option<&ApiMedia>) -> (Option<String>, Option<String>) {
    let Some(media) = media else {
        return (None, None);
    };
    (
        media.hero_image().or_else(|| media.cover_image()),
        media.poster_image(),
    )
}

fn genre_fill(id: u64) -> gpui_kit::Hsla {
    let hue = ((id.wrapping_mul(47)) % 360) as f32 / 360.0;
    hsla(hue, 0.62, 0.42, 1.0)
}
