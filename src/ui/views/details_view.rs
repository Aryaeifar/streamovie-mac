//! Rich Title Details view completely redesigned with Paramount+ aesthetic.
//!
//! Features:
//! - Full-bleed backdrop hero stage with dual gradient scrims and Paramount+ branding
//! - Floating 2:3 cinema poster card with crisp 8px corners and luminous Paramount Blue shadow
//! - Neon cyan status kicker ("NOW STREAMING"), metadata row (DRAMA · YEAR · TV-MA · 4K UHD · ★ IMDb)
//! - Primary electric Paramount Blue ("#0064FF") "▶ WATCH NOW" CTA + Watchlist "+" + Trailer actions
//! - OMDb / IMDb scorecards and catalog metadata
//! - Paramount+ Season pill selector + widescreen (180x101) episode guide cards with direct stream and download options

use gpui_kit::{
    AnyElement, BoxShadow, Context, IntoElement, ParentElement as _, Styled as _, div, hsla,
    linear_color_stop, linear_gradient, point, prelude::FluentBuilder as _, px, relative, white,
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
        px(8.), // Paramount+ 8px radius
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
    let classification = media.classification_label().unwrap_or_else(|| "TV-MA".to_string());
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
    let media_for_watch = media.clone();
    let primary_accent = cx.theme().primary; // #0064FF
    let cyan_kicker = hsla(188.0 / 360.0, 1.0, 0.52, 1.0); // #00E5FF

    let gold_bg = hsla(45.0 / 360.0, 0.95, 0.50, 0.22);
    let gold_fg = hsla(46.0 / 360.0, 0.98, 0.65, 1.0);
    let gold_border = hsla(45.0 / 360.0, 0.95, 0.52, 0.50);

    let stage_bg = hsla(222.0 / 360.0, 0.52, 0.032, 1.0);
    let scrim_bottom = linear_gradient(
        0.0,
        linear_color_stop(stage_bg.opacity(0.98), 0.0),
        linear_color_stop(stage_bg.opacity(0.0), 0.75),
    );
    let scrim_side = linear_gradient(
        if rtl { 270.0 } else { 90.0 },
        linear_color_stop(stage_bg.opacity(0.98), 0.0),
        linear_color_stop(stage_bg.opacity(0.10), 0.90),
    );

    let omdb_section = render_omdb_enrichment_card(details, app, cx);
    let downloads_section = render_downloads_and_seasons_panel(details, app, cx);

    v_flex()
        .w_full()
        .gap_6()
        .pb_12()
        // Paramount+ Back Navigation Bar
        .child(
            directional_row(h_flex(), rtl).w_full().child(
                Button::new("paramount-details-back-btn")
                    .outline()
                    .small()
                    .icon(if rtl {
                        IconName::ArrowRight
                    } else {
                        IconName::ArrowLeft
                    })
                    .label(dict.details.back.to_uppercase())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.close_details(cx);
                    })),
            ),
        )
        // Paramount+ Cinematic Full-Bleed Backdrop Stage + Floating Poster
        .child(
            div()
                .relative()
                .w_full()
                .min_h(px(480.))
                .rounded(px(14.))
                .border_1()
                .border_color(hsla(216.0 / 360.0, 0.40, 0.85, 0.12))
                .bg(stage_bg)
                .shadow_2xl()
                .overflow_hidden()
                // 1. Full-bleed Backdrop Image
                .child(div().absolute().inset_0().child(backdrop_el))
                // 2. Dual Cinematic Gradient Scrims
                .child(div().absolute().inset_0().bg(scrim_side))
                .child(div().absolute().inset_0().bg(scrim_bottom))
                // 3. Top-Right Paramount+ Original Watermark
                .child(
                    directional_row(h_flex(), rtl)
                        .absolute()
                        .top_5()
                        .when(!rtl, |el| el.right_6())
                        .when(rtl, |el| el.left_6())
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py_1p5()
                        .rounded(px(6.))
                        .bg(hsla(222.0 / 360.0, 0.50, 0.05, 0.65))
                        .border_1()
                        .border_color(hsla(0.0, 0.0, 1.0, 0.15))
                        .child(
                            div()
                                .text_size(px(13.))
                                .font_extrabold()
                                .text_color(white())
                                .child("Paramount+"),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded(px(3.))
                                .bg(primary_accent)
                                .text_size(px(9.5))
                                .font_bold()
                                .text_color(white())
                                .child("ORIGINAL"),
                        ),
                )
                // 4. Foreground Poster + Metadata + CTA Bar
                .child(
                    directional_row(h_flex(), rtl)
                        .relative()
                        .w_full()
                        .items_end()
                        .gap_7()
                        .p_7()
                        // Floating 2:3 Cinema Poster (210x315) with Paramount+ Blue Shadow
                        .child(
                            div()
                                .w(px(210.))
                                .h(px(315.))
                                .flex_shrink_0()
                                .rounded(px(8.))
                                .border_1()
                                .border_color(hsla(216.0 / 360.0, 0.40, 0.85, 0.20))
                                .bg(cx.theme().muted)
                                .shadow(vec![
                                    BoxShadow {
                                        offset: point(px(0.), px(10.)),
                                        blur_radius: px(26.),
                                        spread_radius: px(0.),
                                        inset: false,
                                        color: primary_accent.opacity(0.40),
                                    },
                                    BoxShadow {
                                        offset: point(px(0.), px(6.)),
                                        blur_radius: px(14.),
                                        spread_radius: px(0.),
                                        inset: false,
                                        color: hsla(0., 0., 0., 0.90),
                                    },
                                ])
                                .overflow_hidden()
                                .child(poster_el),
                        )
                        // Metadata & Primary Actions Column
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_3()
                                .when(rtl, |this| this.items_end().text_right())
                                // Status Kicker: "NOW STREAMING" in neon cyan
                                .child(
                                    directional_row(h_flex(), rtl)
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .size(px(6.))
                                                .rounded_full()
                                                .bg(cyan_kicker),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_extrabold()
                                                .text_color(cyan_kicker)
                                                .child("NOW STREAMING"),
                                        ),
                                )
                                // Display Title
                                .child(
                                    div()
                                        .text_size(px(38.))
                                        .font_extrabold()
                                        .line_height(relative(1.10))
                                        .text_color(white())
                                        .when(title_rtl, |this| this.text_right())
                                        .child(title.clone()),
                                )
                                // Paramount+ Metadata Row: GENRE · YEAR · TV-MA · RUNTIME · 4K UHD · ★ IMDb
                                .child(
                                    directional_row(h_flex(), rtl)
                                        .flex_wrap()
                                        .items_center()
                                        .gap_2()
                                        .text_xs()
                                        .font_semibold()
                                        .text_color(hsla(215.0 / 360.0, 0.20, 0.80, 1.0))
                                        // Category / Genre
                                        .child(
                                            div()
                                                .font_bold()
                                                .text_color(white())
                                                .child(if category == MediaCategory::Series {
                                                    "SERIES"
                                                } else {
                                                    "MOVIE"
                                                }),
                                        )
                                        .when(!genres.is_empty(), |row| {
                                            row.child(div().child("·")).child(
                                                div().child(genres[0].to_uppercase()),
                                            )
                                        })
                                        .when_some(year, |row, y| {
                                            row.child(div().child("·")).child(div().child(y))
                                        })
                                        // Content rating badge
                                        .child(div().child("·"))
                                        .child(
                                            div()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded(px(4.))
                                                .border_1()
                                                .border_color(hsla(0.0, 0.0, 1.0, 0.25))
                                                .bg(hsla(220.0 / 360.0, 0.35, 0.08, 0.70))
                                                .text_size(px(10.))
                                                .font_bold()
                                                .text_color(white())
                                                .child(classification),
                                        )
                                        .when_some(duration, |row, d| {
                                            row.child(div().child("·")).child(div().child(d))
                                        })
                                        // 4K UHD Badge
                                        .child(div().child("·"))
                                        .child(
                                            div()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded(px(4.))
                                                .border_1()
                                                .border_color(hsla(0.0, 0.0, 1.0, 0.25))
                                                .bg(hsla(220.0 / 360.0, 0.35, 0.08, 0.70))
                                                .text_size(px(10.))
                                                .font_bold()
                                                .text_color(white())
                                                .child("4K UHD"),
                                        )
                                        .when_some(best_quality, |row, q| {
                                            row.child(div().child("·")).child(
                                                div().text_color(primary_accent).child(q),
                                            )
                                        })
                                        .when(rating > 0.0, |row| {
                                            row.child(div().child("·")).child(
                                                h_flex()
                                                    .gap_1()
                                                    .items_center()
                                                    .px_2()
                                                    .py_0p5()
                                                    .rounded(px(4.))
                                                    .bg(gold_bg)
                                                    .border_1()
                                                    .border_color(gold_border)
                                                    .text_xs()
                                                    .font_bold()
                                                    .text_color(gold_fg)
                                                    .child(Icon::new(IconName::StarFill).xsmall())
                                                    .child(format!("{rating:.1}")),
                                            )
                                        }),
                                )
                                // Genres & Countries pills
                                .when(!genres.is_empty() || !countries.is_empty(), |col| {
                                    col.child(
                                        directional_row(h_flex(), rtl)
                                            .flex_wrap()
                                            .gap_1p5()
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
                                            .max_w(px(740.))
                                            .text_sm()
                                            .line_height(relative(1.6))
                                            .text_color(hsla(0.0, 0.0, 0.90, 1.0))
                                            .when(ov_rtl, |this| this.text_right())
                                            .child(overview),
                                    )
                                })
                                // Primary Paramount+ Action Bar
                                .child(
                                    directional_row(h_flex(), rtl)
                                        .pt_2()
                                        .flex_wrap()
                                        .items_center()
                                        .gap_3()
                                        // Primary Watch Now CTA
                                        .child(
                                            Button::new("paramount-details-watch")
                                                .primary()
                                                .large()
                                                .icon(IconName::Play)
                                                .label(dict.actions.watch_now.to_uppercase())
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.open_details(
                                                        media_for_watch.clone(),
                                                        category,
                                                        cx,
                                                    );
                                                })),
                                        )
                                        // Watchlist "+" Button
                                        .child(
                                            Button::new("paramount-details-bm-btn")
                                                .large()
                                                .outline()
                                                .selected(bookmarked)
                                                .icon(if bookmarked {
                                                    IconName::Check
                                                } else {
                                                    IconName::Plus
                                                })
                                                .label(if bookmarked {
                                                    dict.bookmarks.remove.to_uppercase()
                                                } else {
                                                    dict.bookmarks.add.to_uppercase()
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
                                        // Like Button
                                        .child(
                                            Button::new("paramount-details-like")
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
                                        // Trailer Button
                                        .when_some(trailer, |row, tr| {
                                            let url = tr.url.clone();
                                            row.child(
                                                Button::new("paramount-details-trailer")
                                                    .large()
                                                    .outline()
                                                    .icon(IconName::Play)
                                                    .label(dict.actions.watch_trailer.to_uppercase())
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
                    .child(Tag::primary().small().outline().child("Paramount+ · OMDb")),
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
                                    .rounded(px(6.))
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
                            dict.details.download_movie.to_uppercase()
                        } else {
                            dict.details.download_episodes.to_uppercase()
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
                        .rounded(px(8.)) // Crisp Paramount+ 8px radius
                        .border_1()
                        .border_color(cx.theme().border)
                        .bg(cx.theme().popover)
                        .child(
                            v_flex()
                                .w_full()
                                .gap_3p5()
                                // Top row: 16:9 Episode Thumbnail + Episode Details
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
                                                // Widescreen 180x101 Episode Thumbnail (Paramount+ style)
                                                .child(
                                                    div()
                                                        .relative()
                                                        .w(px(180.))
                                                        .h(px(101.))
                                                        .flex_shrink_0()
                                                        .rounded(px(8.))
                                                        .border_1()
                                                        .border_color(hsla(216.0 / 360.0, 0.40, 0.85, 0.14))
                                                        .overflow_hidden()
                                                        .child(render_remote_image(
                                                            app,
                                                            episode_thumb_url.as_deref(),
                                                            episode_fallback_url.as_deref(),
                                                            px(8.),
                                                            IconName::Play,
                                                            None,
                                                            cx,
                                                        ))
                                                        .child(
                                                            div()
                                                                .absolute()
                                                                .inset_0()
                                                                .bg(hsla(0.0, 0.0, 0.0, 0.35))
                                                                .flex()
                                                                .items_center()
                                                                .justify_center()
                                                                .child(
                                                                    div()
                                                                        .size_8()
                                                                        .rounded_full()
                                                                        .bg(cx.theme().primary.opacity(0.90))
                                                                        .border_1()
                                                                        .border_color(white().opacity(0.40))
                                                                        .flex()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .text_color(white())
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
                                                                .rounded(px(4.))
                                                                .bg(hsla(222.0 / 360.0, 0.50, 0.04, 0.85))
                                                                .text_xs()
                                                                .font_bold()
                                                                .text_color(white())
                                                                .child(format!(
                                                                    "S{season_number:02} · E{ep_num:02}"
                                                                )),
                                                        ),
                                                )
                                                // Episode Title & Subtitle
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
                    TabBar::new("paramount-details-season-tabs")
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
                                    Tag::primary().small().outline().child(format_count(
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
