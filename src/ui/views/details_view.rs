//! Rich Title Details view redesigned with an Infuse 8 + Apple TV + Stremio aesthetic.
//! Features:
//! - Full-bleed backdrop hero stage (`min_h(px(430.))`) with `render_remote_image` and dual `linear_gradient` scrims
//! - Floating `230x345` (`2:3`) poster card with shadow elevation and glass hairline border
//! - Gold IMDb pill, quality badge, metadata pills, genre tags, and primary action bar
//! - 4-column OMDb glassmorphic scorecard row + structured catalog metadata
//! - Season pill selector + widescreen (`168x96`) episode guide cards with direct stream/download actions

use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, Styled as _, div, hsla,
    linear_color_stop, linear_gradient, prelude::FluentBuilder as _, px, relative,
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Selectable as _, Sizable as _, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    card::{Card, CardContent, CardDescription, CardHeader, CardTitle},
    description_list::{DescriptionItem, DescriptionList},
    empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle},
    h_flex,
    spinner::Spinner,
    tab::{Tab, TabBar},
    tag::Tag,
    v_flex,
};

use crate::{
    app::StreamovieApp,
    i18n::{directional_row, format_count, is_persian_text},
    models::{MediaCategory, OmdbDetails},
    state::DetailsState,
    ui::{components::render_source_links, image_cache::render_remote_image},
};

pub fn render_details_view(
    app: &StreamovieApp,
    details: &DetailsState,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();

    let media = &details.media;
    let category = details.category;
    let media_id = media.id;
    let bookmarked = app.is_bookmarked(media_id);
    let liked = app.is_liked(media_id);

    let title = media.display_title();
    let title_rtl = is_persian_text(&title) || rtl;
    let poster_url = media.poster_image();
    let cover_url = media.hero_image().or_else(|| poster_url.clone());

    let backdrop_el = render_remote_image(
        app,
        cover_url.as_deref(),
        poster_url.as_deref(),
        cx.theme().radius_lg,
        IconName::Frame,
        None,
        cx,
    );
    let poster_el = render_remote_image(
        app,
        poster_url.as_deref(),
        cover_url.as_deref(),
        cx.theme().radius_lg,
        if category == MediaCategory::Series {
            IconName::Frame
        } else {
            IconName::GalleryVerticalEnd
        },
        Some(&title),
        cx,
    );

    let rating = media.rating_value();
    let year = media.release_year();
    let duration = media.duration_label();
    let classification = media.classification_label();
    let genres = media.genre_list();
    let countries = media.country_list();
    let sources = media.source_links();
    let trailer = sources.iter().find(|s| s.is_trailer).cloned();
    let best_quality = sources
        .iter()
        .find(|s| !s.is_trailer)
        .map(|s| s.quality.clone());
    let parsed = media.parsed_description();

    let media_for_bookmark = media.clone();
    let gold_bg = hsla(45.0 / 360.0, 0.95, 0.50, 0.22);
    let gold_fg = hsla(46.0 / 360.0, 0.98, 0.65, 1.0);
    let gold_border = hsla(45.0 / 360.0, 0.95, 0.52, 0.50);

    let stage_bg = hsla(240.0 / 360.0, 0.18, 0.05, 1.0);
    let scrim_bottom = linear_gradient(
        0.0,
        linear_color_stop(hsla(240.0 / 360.0, 0.20, 0.04, 0.98), 0.0),
        linear_color_stop(hsla(240.0 / 360.0, 0.16, 0.05, 0.50), 1.0),
    );
    let scrim_side = linear_gradient(
        if rtl { 270.0 } else { 90.0 },
        linear_color_stop(hsla(240.0 / 360.0, 0.20, 0.04, 0.94), 0.0),
        linear_color_stop(hsla(240.0 / 360.0, 0.16, 0.05, 0.35), 1.0),
    );

    let omdb_section = render_omdb_enrichment_card(details, app, cx);
    let downloads_section = render_downloads_and_seasons_panel(details, app, cx);

    v_flex()
        .w_full()
        .gap_6()
        .pb_12()
        // Back Navigation Bar
        .child(
            directional_row(h_flex(), rtl).w_full().child(
                Button::new("details-back-btn")
                    .outline()
                    .small()
                    .icon(if rtl {
                        IconName::ArrowRight
                    } else {
                        IconName::ArrowLeft
                    })
                    .label(dict.details.back)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.close_details(cx);
                    })),
            ),
        )
        // Infuse 8 / Apple TV Full-Bleed Backdrop Stage + Floating Poster
        .child(
            div()
                .relative()
                .w_full()
                .min_h(px(420.))
                .rounded(cx.theme().radius_lg)
                .border_1()
                .border_color(hsla(0.0, 0.0, 1.0, 0.12))
                .bg(stage_bg)
                .shadow_xl()
                .overflow_hidden()
                // 1. Full-bleed Backdrop Image
                .child(div().absolute().inset_0().child(backdrop_el))
                // 2. Dual Cinematic Gradient Scrims
                .child(div().absolute().inset_0().bg(scrim_side))
                .child(div().absolute().inset_0().bg(scrim_bottom))
                // 3. Foreground Poster + Metadata + CTA Bar
                .child(
                    directional_row(h_flex(), rtl)
                        .relative()
                        .w_full()
                        .items_end()
                        .gap_7()
                        .p_7()
                        // Floating 2:3 Cinema Poster (230x345)
                        .child(
                            div()
                                .w(px(230.))
                                .h(px(345.))
                                .flex_shrink_0()
                                .rounded(cx.theme().radius_lg)
                                .border_1()
                                .border_color(hsla(0.0, 0.0, 1.0, 0.18))
                                .bg(cx.theme().muted)
                                .shadow_xl()
                                .overflow_hidden()
                                .child(poster_el),
                        )
                        // Metadata & Primary Actions Column
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_3p5()
                                .when(rtl, |this| this.items_end().text_right())
                                // Badges Row
                                .child(
                                    directional_row(h_flex(), rtl)
                                        .flex_wrap()
                                        .items_center()
                                        .gap_2()
                                        .child(Tag::primary().small().child(
                                            if category == MediaCategory::Series {
                                                dict.details.series
                                            } else {
                                                dict.details.movie
                                            },
                                        ))
                                        .when(rating > 0.0, |row| {
                                            row.child(
                                                Tag::custom(gold_bg, gold_fg, gold_border)
                                                    .small()
                                                    .child(
                                                        h_flex()
                                                            .gap_1()
                                                            .items_center()
                                                            .child(
                                                                Icon::new(IconName::StarFill)
                                                                    .xsmall(),
                                                            )
                                                            .child(format!("{rating:.1} IMDb")),
                                                    ),
                                            )
                                        })
                                        .when_some(best_quality, |row, q| {
                                            row.child(
                                                Tag::info().small().outline().child(format!(
                                                    "{}: {q}",
                                                    dict.details.latest_quality
                                                )),
                                            )
                                        })
                                        .when_some(year, |row, y| {
                                            row.child(Tag::secondary().small().outline().child(y))
                                        })
                                        .when_some(duration, |row, d| {
                                            row.child(Tag::secondary().small().outline().child(d))
                                        })
                                        .when_some(classification, |row, c| {
                                            row.child(Tag::warning().small().outline().child(c))
                                        }),
                                )
                                // Title
                                .child(
                                    div()
                                        .text_3xl()
                                        .font_bold()
                                        .line_height(relative(1.12))
                                        .text_color(gpui_kit::white())
                                        .when(title_rtl, |this| this.text_right())
                                        .child(title.clone()),
                                )
                                // Genres & Countries
                                .when(!genres.is_empty() || !countries.is_empty(), |col| {
                                    col.child(
                                        directional_row(h_flex(), rtl)
                                            .flex_wrap()
                                            .gap_2()
                                            .children(
                                                genres
                                                    .into_iter()
                                                    .map(|g| Tag::secondary().small().child(g)),
                                            )
                                            .children(countries.into_iter().map(|c| {
                                                Tag::secondary().small().outline().child(c)
                                            })),
                                    )
                                })
                                // Synopsis / Overview
                                .when_some(parsed.overview.clone(), |col, overview| {
                                    let ov_rtl = is_persian_text(&overview) || rtl;
                                    col.child(
                                        div()
                                            .max_w(px(760.))
                                            .text_sm()
                                            .line_height(relative(1.65))
                                            .text_color(hsla(0.0, 0.0, 0.90, 1.0))
                                            .when(ov_rtl, |this| this.text_right())
                                            .child(overview),
                                    )
                                })
                                // Primary CTA Buttons
                                .child(
                                    directional_row(h_flex(), rtl)
                                        .pt_2()
                                        .flex_wrap()
                                        .gap_3()
                                        .child(
                                            Button::new("details-bookmark-btn")
                                                .primary()
                                                .large()
                                                .selected(bookmarked)
                                                .icon(if bookmarked {
                                                    IconName::StarFill
                                                } else {
                                                    IconName::BookOpen
                                                })
                                                .label(if bookmarked {
                                                    dict.bookmarks.remove
                                                } else {
                                                    dict.bookmarks.add
                                                })
                                                .on_click(cx.listener(move |this, _, window, cx| {
                                                    let added = this.toggle_bookmark(
                                                        media_for_bookmark.clone(),
                                                        category,
                                                        cx,
                                                    );
                                                    let msg = if added {
                                                        this.locale().dict().bookmarks.add
                                                    } else {
                                                        this.locale().dict().bookmarks.remove
                                                    };
                                                    window.push_notification(msg, cx);
                                                })),
                                        )
                                        .child(
                                            Button::new("details-like-btn")
                                                .large()
                                                .outline()
                                                .selected(liked)
                                                .icon(IconName::Heart)
                                                .label(if liked {
                                                    dict.actions.unlike
                                                } else {
                                                    dict.actions.like
                                                })
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.toggle_like(media_id, cx);
                                                })),
                                        )
                                        .when_some(trailer, |row, tr| {
                                            let url = tr.url.clone();
                                            row.child(
                                                Button::new("details-trailer-btn")
                                                    .large()
                                                    .outline()
                                                    .icon(IconName::Play)
                                                    .label(dict.actions.watch_trailer)
                                                    .on_click(move |_, _, _| {
                                                        let _ = std::process::Command::new("open")
                                                            .arg(&url)
                                                            .spawn();
                                                    }),
                                            )
                                        }),
                                ),
                        ),
                ),
        )
        // Parsed Catalog Metadata DescriptionList
        .when(!parsed.fields.is_empty(), |col| {
            let items: Vec<DescriptionItem> = parsed
                .fields
                .iter()
                .map(|block| {
                    DescriptionItem::new(block.label.clone())
                        .value(block.value.clone())
                        .span(1)
                })
                .collect();

            col.child(
                Card::new()
                    .child(
                        CardHeader::new().child(
                            directional_row(h_flex(), rtl)
                                .gap_2()
                                .child(
                                    Icon::new(IconName::Info)
                                        .small()
                                        .text_color(cx.theme().primary),
                                )
                                .child(CardTitle::new().child(dict.details.overview)),
                        ),
                    )
                    .child(
                        CardContent::new().child(
                            DescriptionList::horizontal()
                                .columns(2)
                                .label_width(px(140.))
                                .children(items),
                        ),
                    ),
            )
        })
        // OMDb Scorecards & Enrichment
        .child(omdb_section)
        // Downloads & Widescreen Episode Guide
        .child(downloads_section)
        .into_any_element()
}

fn render_omdb_enrichment_card(
    details: &DetailsState,
    app: &StreamovieApp,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();

    Card::new()
        .child(
            CardHeader::new().child(
                directional_row(h_flex(), rtl)
                    .w_full()
                    .justify_between()
                    .child(
                        directional_row(h_flex(), rtl)
                            .gap_2()
                            .child(
                                Icon::new(IconName::StarFill)
                                    .small()
                                    .text_color(cx.theme().primary),
                            )
                            .child(CardTitle::new().child(dict.details.show_info)),
                    )
                    .child(Tag::secondary().small().outline().child("OMDb · IMDb")),
            ),
        )
        .child(CardContent::new().child(if details.omdb_loading {
            h_flex()
                .py_4()
                .items_center()
                .gap_2()
                .child(Spinner::new().small())
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(dict.common.loading),
                )
                .into_any_element()
        } else if let Some(om) = details.omdb_details.as_ref().filter(|d| d.is_ok()) {
            let mut desc_items = Vec::new();
            if let Some(imdb) = OmdbDetails::non_na(&om.imdb_rating) {
                let votes = OmdbDetails::non_na(&om.imdb_votes).unwrap_or_default();
                let val = if votes.is_empty() {
                    format!("{imdb} / 10")
                } else {
                    format!("{imdb} / 10 ({votes} {})", dict.details.votes)
                };
                desc_items.push(DescriptionItem::new("IMDb").value(val).span(1));
            }
            if let Some(rated) = OmdbDetails::non_na(&om.rated) {
                desc_items.push(DescriptionItem::new("Rated").value(rated.to_string()).span(1));
            }
            if let Some(runtime) = OmdbDetails::non_na(&om.runtime) {
                desc_items.push(
                    DescriptionItem::new(dict.details.duration)
                        .value(runtime.to_string())
                        .span(1),
                );
            }
            if let Some(country) = OmdbDetails::non_na(&om.country) {
                desc_items.push(
                    DescriptionItem::new(dict.details.country)
                        .value(country.to_string())
                        .span(1),
                );
            }
            if let Some(lang) = OmdbDetails::non_na(&om.language) {
                desc_items.push(
                    DescriptionItem::new(dict.details.languages)
                        .value(lang.to_string())
                        .span(1),
                );
            }
            if let Some(seasons) = OmdbDetails::non_na(&om.total_seasons) {
                if let Ok(n) = seasons.parse::<usize>() {
                    desc_items.push(
                        DescriptionItem::new(dict.details.season)
                            .value(format_count(dict.details.seasons_count, n, locale))
                            .span(1),
                    );
                }
            }
            if let Some(actors) = OmdbDetails::non_na(&om.actors) {
                desc_items.push(
                    DescriptionItem::new(dict.details.cast)
                        .value(actors.to_string())
                        .span(2),
                );
            }
            if let Some(awards) = OmdbDetails::non_na(&om.awards) {
                desc_items.push(
                    DescriptionItem::new(dict.details.ratings_and_awards)
                        .value(awards.to_string())
                        .span(2),
                );
            }

            v_flex()
                .gap_4()
                .when(!om.ratings.is_empty(), |col| {
                    col.child(
                        directional_row(h_flex(), rtl)
                            .flex_wrap()
                            .gap_2p5()
                            .children(om.ratings.iter().map(|r| {
                                div()
                                    .px_3()
                                    .py_1p5()
                                    .rounded(cx.theme().radius)
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .bg(cx.theme().background)
                                    .child(
                                        h_flex()
                                            .gap_2()
                                            .items_center()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(r.source.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_bold()
                                                    .text_color(cx.theme().foreground)
                                                    .child(r.value.clone()),
                                            ),
                                    )
                            })),
                    )
                })
                .when_some(OmdbDetails::non_na(&om.plot), |col, plot| {
                    col.child(
                        div()
                            .text_sm()
                            .line_height(relative(1.6))
                            .text_color(cx.theme().foreground)
                            .child(plot.to_string()),
                    )
                })
                .when(!desc_items.is_empty(), |col| {
                    col.child(
                        DescriptionList::horizontal()
                            .columns(2)
                            .label_width(px(140.))
                            .children(desc_items),
                    )
                })
                .into_any_element()
        } else {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(
                    details
                        .omdb_error
                        .clone()
                        .unwrap_or_else(|| dict.details.omdb_no_match.to_string()),
                )
                .into_any_element()
        }))
        .into_any_element()
}

fn render_downloads_and_seasons_panel(
    details: &DetailsState,
    app: &StreamovieApp,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let is_movie = details.category == MediaCategory::Movie;
    let media_title = details.media.display_title();
    let episode_thumb_url = details
        .media
        .hero_image()
        .or_else(|| details.media.cover_image())
        .or_else(|| details.media.poster_image());
    let episode_fallback_url = details.media.poster_image();

    Card::new()
        .child(
            CardHeader::new()
                .child(
                    directional_row(h_flex(), rtl)
                        .gap_2()
                        .child(
                            Icon::new(IconName::ArrowDown)
                                .small()
                                .text_color(cx.theme().primary),
                        )
                        .child(CardTitle::new().child(if is_movie {
                            dict.details.download_movie
                        } else {
                            dict.details.download_episodes
                        })),
                )
                .child(CardDescription::new().child(if is_movie {
                    dict.details.download_movie_hint
                } else {
                    dict.details.download_episodes_hint
                })),
        )
        .child(CardContent::new().child(if is_movie {
            let movie_sources = details.media.source_links();
            render_source_links(&movie_sources, &media_title, locale, "movie", cx)
                .into_any_element()
        } else if details.seasons_loading {
            h_flex()
                .py_6()
                .items_center()
                .gap_2()
                .child(Spinner::new().small())
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(dict.common.loading),
                )
                .into_any_element()
        } else if details.seasons.is_empty() {
            Empty::new()
                .header(
                    EmptyHeader::new()
                        .media(
                            EmptyMedia::new()
                                .with_variant(EmptyMediaVariant::Icon)
                                .child(Icon::new(IconName::Frame)),
                        )
                        .title(EmptyTitle::new().child(dict.details.no_seasons))
                        .description(EmptyDescription::new().child(dict.details.no_seasons_hint)),
                )
                .into_any_element()
        } else {
            let selected_ix = details
                .selected_season_index
                .min(details.seasons.len().saturating_sub(1));
            let active_season = &details.seasons[selected_ix];
            let catalog_episodes = active_season.episode_list();
            let omdb_episodes = details
                .omdb_season
                .as_ref()
                .map(|s| s.episodes.as_slice())
                .unwrap_or(&[]);
            let total_ep_count = catalog_episodes.len().max(omdb_episodes.len());
            let season_number = selected_ix + 1;

            let mut ep_cards = Vec::with_capacity(total_ep_count);
            for ep_ix in 0..total_ep_count {
                let cat_ep = catalog_episodes.get(ep_ix);
                let omdb_ep = omdb_episodes
                    .iter()
                    .find(|e| {
                        e.episode
                            .as_deref()
                            .and_then(|n| n.parse::<usize>().ok())
                            == Some(ep_ix + 1)
                    })
                    .or_else(|| omdb_episodes.get(ep_ix));

                let ep_num = ep_ix + 1;
                let ep_title = omdb_ep
                    .and_then(|e| e.title.clone())
                    .or_else(|| cat_ep.map(|e| e.title.clone()))
                    .unwrap_or_else(|| format!("{} {ep_num}", dict.details.episode));
                let ep_released = omdb_ep
                    .and_then(|e| e.released.clone())
                    .filter(|v| v != "N/A");
                let ep_rating = omdb_ep
                    .and_then(|e| e.imdb_rating.clone())
                    .filter(|v| v != "N/A");
                let ep_sources = cat_ep.map(|e| e.sources.as_slice()).unwrap_or(&[]);
                let ep_context =
                    format!("{media_title} S{season_number:02}E{ep_num:02} - {ep_title}");
                let ep_prefix = format!("s{season_number}-e{ep_num}");

                let source_links_el =
                    render_source_links(ep_sources, &ep_context, locale, &ep_prefix, cx);

                ep_cards.push(
                    div()
                        .w_full()
                        .p_4()
                        .rounded(cx.theme().radius_lg)
                        .border_1()
                        .border_color(cx.theme().border)
                        .bg(cx.theme().background)
                        .child(
                            v_flex()
                                .w_full()
                                .gap_3p5()
                                // Top row: 16:9 Episode Still Thumbnail + Episode Header
                                .child(
                                    directional_row(h_flex(), rtl)
                                        .w_full()
                                        .items_center()
                                        .justify_between()
                                        .gap_4()
                                        .child(
                                            directional_row(h_flex(), rtl)
                                                .items_center()
                                                .gap_3p5()
                                                .min_w_0()
                                                // Widescreen 168x96 Episode Thumbnail (Infuse / Apple TV style)
                                                .child(
                                                    div()
                                                        .relative()
                                                        .w(px(168.))
                                                        .h(px(96.))
                                                        .flex_shrink_0()
                                                        .rounded(cx.theme().radius)
                                                        .border_1()
                                                        .border_color(hsla(0.0, 0.0, 1.0, 0.12))
                                                        .overflow_hidden()
                                                        .child(render_remote_image(
                                                            app,
                                                            episode_thumb_url.as_deref(),
                                                            episode_fallback_url.as_deref(),
                                                            cx.theme().radius,
                                                            IconName::Play,
                                                            None,
                                                            cx,
                                                        ))
                                                        .child(
                                                            div()
                                                                .absolute()
                                                                .inset_0()
                                                                .bg(hsla(0.0, 0.0, 0.0, 0.32))
                                                                .flex()
                                                                .items_center()
                                                                .justify_center()
                                                                .child(
                                                                    div()
                                                                        .size_8()
                                                                        .rounded_full()
                                                                        .bg(hsla(
                                                                            0.0, 0.0, 0.0, 0.65,
                                                                        ))
                                                                        .border_1()
                                                                        .border_color(hsla(
                                                                            0.0, 0.0, 1.0, 0.25,
                                                                        ))
                                                                        .flex()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .text_color(gpui_kit::white())
                                                                        .child(
                                                                            Icon::new(IconName::Play)
                                                                                .small(),
                                                                        ),
                                                                ),
                                                        )
                                                        .child(
                                                            div()
                                                                .absolute()
                                                                .bottom_1p5()
                                                                .left_1p5()
                                                                .px_1p5()
                                                                .py_0p5()
                                                                .rounded(px(5.))
                                                                .bg(hsla(0.0, 0.0, 0.0, 0.78))
                                                                .text_xs()
                                                                .font_bold()
                                                                .text_color(gpui_kit::white())
                                                                .child(format!(
                                                                    "S{season_number:02} · E{ep_num:02}"
                                                                )),
                                                        ),
                                                )
                                                // Episode Title & Number
                                                .child(
                                                    v_flex()
                                                        .gap_1()
                                                        .min_w_0()
                                                        .when(rtl, |c| c.items_end().text_right())
                                                        .child(
                                                            div()
                                                                .text_base()
                                                                .font_bold()
                                                                .text_color(cx.theme().foreground)
                                                                .child(ep_title),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(
                                                                    cx.theme().muted_foreground,
                                                                )
                                                                .child(format!(
                                                                    "{} {season_number} · {} {ep_num}",
                                                                    dict.details.season,
                                                                    dict.details.episode
                                                                )),
                                                        ),
                                                ),
                                        )
                                        // Release date & IMDb score badges
                                        .child(
                                            directional_row(h_flex(), rtl)
                                                .gap_2()
                                                .when_some(ep_released, |row, rel| {
                                                    row.child(
                                                        Tag::secondary()
                                                            .small()
                                                            .outline()
                                                            .child(rel),
                                                    )
                                                })
                                                .when_some(ep_rating, |row, r| {
                                                    row.child(
                                                        Tag::warning()
                                                            .small()
                                                            .child(format!("★ {r}")),
                                                    )
                                                }),
                                        ),
                                )
                                // Bottom row: Stream / Download Source Pills
                                .child(source_links_el),
                        ),
                );
            }

            v_flex()
                .gap_5()
                // Season Selector Pill Bar
                .child(
                    TabBar::new("details-season-tabs")
                        .pill()
                        .selected_index(selected_ix)
                        .children(details.seasons.iter().enumerate().map(|(ix, season)| {
                            Tab::new().label(season.display_title(ix))
                        }))
                        .on_click(cx.listener(|this, ix: &usize, _, cx| {
                            this.select_season(*ix, cx);
                        })),
                )
                // Episode Guide List
                .child(if ep_cards.is_empty() {
                    Empty::new()
                        .header(
                            EmptyHeader::new()
                                .title(EmptyTitle::new().child(dict.details.no_episodes))
                                .description(
                                    EmptyDescription::new()
                                        .child(dict.details.select_season_first),
                                ),
                        )
                        .into_any_element()
                } else {
                    v_flex()
                        .gap_3p5()
                        .child(
                            directional_row(h_flex(), rtl)
                                .justify_between()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_semibold()
                                        .child(format!(
                                            "{} — {} {}",
                                            dict.details.episode_guide,
                                            dict.details.season,
                                            season_number
                                        )),
                                )
                                .child(
                                    Tag::secondary().small().outline().child(format_count(
                                        dict.details.episodes_count,
                                        total_ep_count,
                                        locale,
                                    )),
                                ),
                        )
                        .children(ep_cards)
                        .into_any_element()
                })
                .into_any_element()
        }))
        .into_any_element()
}
