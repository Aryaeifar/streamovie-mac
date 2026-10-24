//! Movie and series detail pages, laid out like the Apple TV title screen.

use gpui_kit::component::{
    ActiveTheme as _, IconName, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, linear_color_stop, linear_gradient,
    prelude::FluentBuilder as _, px, relative,
};

use crate::{
    app::StreamovieApp,
    download::{DownloadPhase, DownloadSnapshot},
    i18n::{directional_row, is_persian_text},
    models::{ApiMedia, Episode, MediaCategory, OmdbDetails, SourceLink, best_download_source},
    state::DetailsState,
    ui::{
        apple::{AppleButtonKind, ApplePaint, apple_button, apple_progress, apple_section_title},
        components::{
            PosterCardStyle, cast_skeleton_row, empty_state, episode_skeleton_row,
            network_error_state, render_poster_card, skeleton_row,
        },
        image_cache::render_remote_image,
    },
};

struct Credit {
    name: String,
    role: Option<String>,
}

pub fn render_details_view(
    app: &StreamovieApp,
    details: &DetailsState,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let locale = app.locale();
    let rtl = locale.is_rtl();
    let media = &details.media;
    let category = details.category;
    let title = media.display_title();
    let title_rtl = is_persian_text(&title) || rtl;
    let bookmarked = app.is_bookmarked(media.id);
    let parsed = media.parsed_description();
    let overview = parsed.overview.clone().or_else(|| {
        details
            .omdb_details
            .as_ref()
            .and_then(|item| OmdbDetails::non_na(&item.plot).map(str::to_string))
    });
    let year = media.release_year().or_else(|| {
        details
            .omdb_details
            .as_ref()
            .and_then(|item| OmdbDetails::non_na(&item.year).map(str::to_string))
    });
    let runtime = media.duration_label().or_else(|| {
        details
            .omdb_details
            .as_ref()
            .and_then(|item| OmdbDetails::non_na(&item.runtime).map(str::to_string))
    });
    let rating = details
        .omdb_details
        .as_ref()
        .and_then(|item| OmdbDetails::non_na(&item.imdb_rating))
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| *value > 0.0)
        .unwrap_or_else(|| media.rating_value());
    let votes = details
        .omdb_details
        .as_ref()
        .and_then(|item| OmdbDetails::non_na(&item.imdb_votes).map(str::to_string));
    let mut genres = media.genre_list();
    if genres.is_empty() {
        if let Some(genre) = details
            .omdb_details
            .as_ref()
            .and_then(|item| OmdbDetails::non_na(&item.genre))
        {
            genres = genre
                .split(',')
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect();
        }
    }
    let mut credits = Vec::new();
    if let Some(director) = details
        .omdb_details
        .as_ref()
        .and_then(|item| OmdbDetails::non_na(&item.director))
    {
        let role = if rtl { "کارگردان" } else { "Director" };
        for name in director
            .split(',')
            .map(|name| name.trim())
            .filter(|name| !name.is_empty())
            .take(2)
        {
            credits.push(Credit {
                name: name.to_string(),
                role: Some(role.to_string()),
            });
        }
    }
    if let Some(actors) = details
        .omdb_details
        .as_ref()
        .and_then(|item| OmdbDetails::non_na(&item.actors))
    {
        for name in actors
            .split(',')
            .map(|name| name.trim())
            .filter(|name| !name.is_empty())
            .take(8)
        {
            credits.push(Credit {
                name: name.to_string(),
                role: None,
            });
        }
    }
    let teaser = overview.as_ref().map(|text| {
        let shown: String = text.chars().take(150).collect();
        if text.chars().count() > 150 {
            format!("{shown}…")
        } else {
            shown
        }
    });
    let artwork = artwork_urls(media);
    let poster = media.poster_image();
    let cover = media.hero_image().or(poster.clone());
    let sources = media.source_links();
    let has_trailer = sources.iter().any(|source| source.is_trailer);
    let paint = ApplePaint::from_cx(cx);

    let hero_and_body = if category == MediaCategory::Series {
        series_content(
            app,
            details,
            &title,
            title_rtl,
            teaser.as_deref(),
            overview.as_deref(),
            year.as_deref(),
            runtime.as_deref(),
            rating,
            votes.as_deref(),
            &genres,
            &credits,
            cover.as_deref(),
            poster.as_deref(),
            bookmarked,
            rtl,
            cx,
        )
    } else {
        movie_content(
            app,
            &title,
            title_rtl,
            teaser.as_deref(),
            overview.as_deref(),
            year.as_deref(),
            runtime.as_deref(),
            rating,
            votes.as_deref(),
            &genres,
            &credits,
            &artwork,
            cover.as_deref(),
            poster.as_deref(),
            bookmarked,
            has_trailer,
            &sources,
            category,
            details.omdb_loading,
            rtl,
            cx,
        )
    };

    v_flex()
        .w_full()
        .min_w_0()
        .bg(paint.canvas)
        .child(hero_and_body)
        .into_any_element()
}

fn movie_content(
    app: &StreamovieApp,
    title: &str,
    title_rtl: bool,
    teaser: Option<&str>,
    overview: Option<&str>,
    year: Option<&str>,
    runtime: Option<&str>,
    rating: f64,
    votes: Option<&str>,
    genres: &[String],
    credits: &[Credit],
    artwork: &[String],
    cover: Option<&str>,
    poster: Option<&str>,
    bookmarked: bool,
    has_trailer: bool,
    sources: &[SourceLink],
    category: MediaCategory,
    omdb_loading: bool,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let paint = ApplePaint::from_cx(cx);
    let media = app_media(app);
    let media_for_watch = media.clone();
    let media_for_bookmark = media.clone();
    let download_key = format!("movie-{}", media.id);
    let download_key_for_label = download_key.clone();
    let download_sources = sources.to_vec();
    let download_heading = title.to_string();
    let trailer_sources = sources.to_vec();
    let trailer_heading = format!("{title} · {}", dict.actions.watch_trailer);
    let show_overview = overview.is_some_and(|text| text.chars().count() > 150);

    let actions = directional_row(h_flex(), rtl)
        .gap(px(10.))
        .child(apple_button(
            app,
            "details-watch",
            dict.actions.watch_now,
            Some(IconName::Play),
            AppleButtonKind::Primary,
            false,
            cx.listener(move |this, _, _, cx| {
                this.watch_title(media_for_watch.clone(), category, cx);
            }),
            cx,
        ))
        .child(apple_button(
            app,
            "details-download",
            download_action_label(app, &download_key_for_label),
            Some(IconName::ArrowDown),
            AppleButtonKind::Secondary,
            false,
            cx.listener(move |this, _, window, cx| {
                on_download_click(
                    this,
                    download_key.clone(),
                    download_heading.clone(),
                    download_sources.clone(),
                    window,
                    cx,
                );
            }),
            cx,
        ))
        .when(has_trailer, |row| {
            row.child(apple_button(
                app,
                "details-trailer",
                dict.actions.watch_trailer,
                Some(IconName::Play),
                AppleButtonKind::Secondary,
                false,
                cx.listener(move |this, _, _, cx| {
                    this.watch_trailer(trailer_heading.clone(), trailer_sources.clone(), cx);
                }),
                cx,
            ))
        })
        .when(!has_trailer, |row| {
            row.child(apple_button(
                app,
                "details-save",
                if bookmarked {
                    dict.bookmarks.remove
                } else {
                    dict.bookmarks.add
                },
                Some(if bookmarked {
                    IconName::Check
                } else {
                    IconName::Plus
                }),
                AppleButtonKind::Secondary,
                false,
                cx.listener(move |this, _, window, cx| {
                    let added = this.toggle_bookmark(media_for_bookmark.clone(), category, cx);
                    window.push_notification(
                        if added {
                            this.locale().dict().bookmarks.add
                        } else {
                            this.locale().dict().bookmarks.remove
                        },
                        cx,
                    );
                }),
                cx,
            ))
        });

    v_flex()
        .w_full()
        .min_w_0()
        .child(render_hero(
            app,
            title,
            title_rtl,
            teaser,
            year,
            runtime,
            rating,
            votes,
            genres,
            cover,
            poster,
            bookmarked,
            rtl,
            actions.into_any_element(),
            cx,
        ))
        .child(
            v_flex()
                .w_full()
                .min_w_0()
                .px(px(48.))
                .pt(px(8.))
                .pb(px(64.))
                .gap(px(40.))
                .when(show_overview, |col| {
                    col.child(overview_block(
                        overview.unwrap_or_default(),
                        dict.details.overview,
                        rtl,
                        &paint,
                    ))
                })
                .child(movie_download_card(
                    app,
                    &download_key_for_label,
                    dict.details.download_movie,
                    dict.details.download_movie_hint,
                    rtl,
                    cx,
                ))
                .when(omdb_loading && credits.is_empty(), |col| {
                    col.child(cast_section_skeleton(dict.details.cast, rtl, &paint))
                })
                .when(!credits.is_empty(), |col| {
                    col.child(cast_row(credits, dict.details.cast, rtl, &paint))
                })
                .when(!artwork.is_empty(), |col| {
                    col.child(poster_row(app, artwork, dict.details.posters, rtl, cx))
                })
                .child(similar_rail(app, category, rtl, cx)),
        )
        .into_any_element()
}

fn series_content(
    app: &StreamovieApp,
    details: &DetailsState,
    title: &str,
    title_rtl: bool,
    teaser: Option<&str>,
    overview: Option<&str>,
    year: Option<&str>,
    runtime: Option<&str>,
    rating: f64,
    votes: Option<&str>,
    genres: &[String],
    credits: &[Credit],
    cover: Option<&str>,
    poster: Option<&str>,
    bookmarked: bool,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let paint = ApplePaint::from_cx(cx);
    let media_for_watch = app_media(app);
    let show_overview = overview.is_some_and(|text| text.chars().count() > 150);
    let actions = apple_button(
        app,
        "series-watch",
        dict.actions.watch_now,
        Some(IconName::Play),
        AppleButtonKind::Primary,
        false,
        cx.listener(move |this, _, _, cx| {
            this.watch_title(media_for_watch.clone(), MediaCategory::Series, cx);
        }),
        cx,
    );

    v_flex()
        .w_full()
        .min_w_0()
        .child(render_hero(
            app, title, title_rtl, teaser, year, runtime, rating, votes, genres, cover, poster,
            bookmarked, rtl, actions, cx,
        ))
        .child(
            v_flex()
                .w_full()
                .min_w_0()
                .px(px(48.))
                .pt(px(8.))
                .pb(px(64.))
                .gap(px(40.))
                .when(show_overview, |col| {
                    col.child(overview_block(
                        overview.unwrap_or_default(),
                        dict.details.overview,
                        rtl,
                        &paint,
                    ))
                })
                .child(
                    v_flex()
                        .w_full()
                        .min_w_0()
                        .gap(px(14.))
                        .when(rtl, |col| col.items_end())
                        .child(apple_section_title(dict.details.download_episodes, &paint))
                        .child(
                            div()
                                .max_w(px(640.))
                                .text_size(px(13.))
                                .line_height(relative(1.4))
                                .text_color(paint.secondary_text)
                                .when(rtl, |el| el.text_right())
                                .child(dict.details.download_episodes_hint),
                        )
                        .child(render_episodes(app, details, rtl, cx)),
                )
                .when(details.omdb_loading && credits.is_empty(), |col| {
                    col.child(cast_section_skeleton(dict.details.cast, rtl, &paint))
                })
                .when(!credits.is_empty(), |col| {
                    col.child(cast_row(credits, dict.details.cast, rtl, &paint))
                })
                .child(similar_rail(app, MediaCategory::Series, rtl, cx)),
        )
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_hero(
    app: &StreamovieApp,
    title: &str,
    title_rtl: bool,
    teaser: Option<&str>,
    year: Option<&str>,
    runtime: Option<&str>,
    rating: f64,
    votes: Option<&str>,
    genres: &[String],
    cover: Option<&str>,
    poster: Option<&str>,
    bookmarked: bool,
    rtl: bool,
    actions: AnyElement,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let paint = ApplePaint::from_cx(cx);
    v_flex()
        .w_full()
        .child(
            div()
                .relative()
                .w_full()
                .h(px(460.))
                .overflow_hidden()
                .bg(paint.chip)
                .child(render_remote_image(
                    app,
                    cover,
                    poster,
                    px(0.),
                    IconName::Frame,
                    None,
                    cx,
                ))
                .child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .h(px(220.))
                        .bg(linear_gradient(
                            0.0,
                            linear_color_stop(paint.canvas, 0.0),
                            linear_color_stop(paint.canvas.opacity(0.0), 1.0),
                        )),
                )
                .child(top_bar(app, bookmarked, rtl, cx)),
        )
        .child(
            v_flex()
                .w_full()
                .px(px(48.))
                .mt(px(-28.))
                .pb(px(8.))
                .gap(px(14.))
                .when(rtl, |col| col.items_end().text_right())
                .child(
                    div()
                        .max_w(px(760.))
                        .text_size(px(46.))
                        .font_bold()
                        .line_height(relative(1.05))
                        .text_color(paint.ink)
                        .when(title_rtl, |el| el.text_right())
                        .child(title.to_string()),
                )
                .child(meta_row(rating, votes, runtime, genres, year, rtl, &paint))
                .when_some(teaser, |col, text| {
                    let text_rtl = is_persian_text(text) || rtl;
                    let text = text.to_string();
                    col.child(
                        div()
                            .max_w(px(640.))
                            .text_size(px(15.))
                            .line_height(relative(1.45))
                            .text_color(paint.body)
                            .when(text_rtl, |el| el.text_right())
                            .child(text),
                    )
                })
                .child(actions),
        )
        .into_any_element()
}

fn render_episodes(
    app: &StreamovieApp,
    details: &DetailsState,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();

    if details.seasons_loading && details.seasons.is_empty() {
        return episode_skeleton_row(rtl);
    }

    if details.seasons.is_empty() {
        if let Some(err) = details.seasons_error.clone() {
            return network_error_state(
                dict.common.network_title,
                dict.common.series_failed,
                dict.details.episodes_unavailable,
                Some(err),
                Some(season_retry_button(dict.actions.retry, cx)),
                cx,
            );
        }
        return empty_state(
            IconName::Inbox,
            dict.details.no_seasons,
            dict.details.no_seasons_hint,
            None,
            cx,
        );
    }

    let seasons_error = details.seasons_error.clone();

    let selected = details
        .selected_season_index
        .min(details.seasons.len().saturating_sub(1));
    let season = &details.seasons[selected];
    let episodes = season.episode_list();
    let omdb_episodes = details
        .omdb_season
        .as_ref()
        .map(|season| season.episodes.clone())
        .unwrap_or_default();
    let series_title = details.media.display_title();
    let series_id = details.media.id;
    let thumb = details
        .media
        .hero_image()
        .or_else(|| details.media.poster_image());
    let thumb_fallback = details.media.poster_image();

    let cards: Vec<_> = episodes
        .iter()
        .enumerate()
        .map(|(index, episode)| {
            episode_card(
                app,
                episode,
                index,
                selected,
                &omdb_episodes,
                &series_title,
                series_id,
                thumb.as_deref(),
                thumb_fallback.as_deref(),
                rtl,
                cx,
            )
        })
        .collect();

    v_flex()
        .w_full()
        .min_w_0()
        .gap(px(16.))
        .child(
            div()
                .id("details-season-rail")
                .w_full()
                .min_w_0()
                .overflow_x_scroll()
                .restrict_scroll_to_axis()
                .child(
                    directional_row(h_flex(), rtl)
                        .flex_shrink_0()
                        .gap(px(8.))
                        .py(px(2.))
                        .children(details.seasons.iter().enumerate().map(|(index, season)| {
                            season_pill(
                                app,
                                series_id,
                                index,
                                selected,
                                &season.display_title(index),
                                cx,
                            )
                        })),
                ),
        )
        .when_some(seasons_error, |col, err| {
            col.child(network_error_state(
                dict.common.network_title,
                dict.common.series_failed,
                dict.details.episodes_unavailable,
                Some(err),
                Some(season_retry_button(dict.actions.retry, cx)),
                cx,
            ))
        })
        .child(if cards.is_empty() {
            empty_state(IconName::Inbox, dict.details.no_episodes, "", None, cx)
        } else {
            div()
                .id("details-episode-rail")
                .w_full()
                .min_w_0()
                .overflow_x_scroll()
                .restrict_scroll_to_axis()
                .child(
                    h_flex()
                        .flex_shrink_0()
                        .gap(px(18.))
                        .py(px(8.))
                        .when(rtl, |row| row.flex_row_reverse())
                        .children(cards),
                )
                .into_any_element()
        })
        .into_any_element()
}

fn season_pill(
    _app: &StreamovieApp,
    series_id: u64,
    index: usize,
    selected: usize,
    label: &str,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let paint = ApplePaint::from_cx(cx);
    let active = index == selected;
    div()
        .id(SharedString::from(format!("season-{series_id}-{index}")))
        .h(px(32.))
        .px(px(14.))
        .rounded(px(12.))
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .text_size(px(13.))
        .when(active, |el| {
            el.font_semibold()
                .bg(paint.primary_fill)
                .text_color(paint.on_primary)
        })
        .when(!active, |el| {
            el.bg(paint.chip).text_color(paint.secondary_text)
        })
        .child(label.to_string())
        .on_click(cx.listener(move |this, _, _, cx| {
            this.select_season(index, cx);
        }))
        .into_any_element()
}

fn episode_card(
    app: &StreamovieApp,
    episode: &Episode,
    index: usize,
    season_index: usize,
    omdb_episodes: &[crate::models::OmdbEpisode],
    series_title: &str,
    series_id: u64,
    thumb: Option<&str>,
    thumb_fallback: Option<&str>,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let paint = ApplePaint::from_cx(cx);
    let accent = cx.theme().primary;
    let number = index + 1;
    let omdb = omdb_episodes.iter().find(|item| {
        item.episode
            .as_deref()
            .and_then(|value| value.parse::<usize>().ok())
            == Some(number)
    });
    let title = omdb
        .and_then(|item| item.title.clone())
        .filter(|title| !title.is_empty() && title != "N/A")
        .unwrap_or_else(|| episode.title.clone());
    let released = omdb.and_then(|item| OmdbDetails::non_na(&item.released).map(str::to_string));
    let key = format!("{series_id}-s{}-e{number}", season_index + 1);
    let sources = episode.sources.clone();
    let can_download = best_download_source(&sources).is_some();
    let watch_sources = sources.clone();
    let download_sources = sources;
    let watch_heading = format!("{series_title} · {title}");
    let watch_heading_for_play = watch_heading.clone();
    let watch_sources_for_play = watch_sources.clone();
    let download_heading = watch_heading.clone();
    let title_rtl = is_persian_text(&title) || rtl;
    let job = app.download_snapshot(&key);
    let fraction = job.as_ref().and_then(progress_fraction);
    let running = job
        .as_ref()
        .is_some_and(|job| job.phase == DownloadPhase::Running);

    v_flex()
        .id(SharedString::from(format!("ep-card-{key}")))
        .w(px(292.))
        .flex_shrink_0()
        .rounded(px(16.))
        .overflow_hidden()
        .bg(paint.surface)
        .border_1()
        .border_color(paint.hairline)
        .shadow(paint.poster_shadow(false))
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| {
            this.watch_sources(watch_heading.clone(), watch_sources.clone(), cx);
        }))
        .child(
            div()
                .relative()
                .w_full()
                .h(px(164.))
                .bg(paint.chip)
                .child(render_remote_image(
                    app,
                    thumb,
                    thumb_fallback,
                    px(0.),
                    IconName::Frame,
                    None,
                    cx,
                ))
                .when(running, |frame| {
                    frame.child(
                        div()
                            .absolute()
                            .left(px(10.))
                            .right(px(10.))
                            .bottom(px(10.))
                            .child(apple_progress(
                                fraction.unwrap_or(0.35),
                                accent,
                                paint.track,
                            )),
                    )
                }),
        )
        .child(
            v_flex()
                .p(px(12.))
                .gap(px(6.))
                .when(title_rtl, |col| col.items_end())
                .child(
                    div()
                        .text_size(px(11.))
                        .font_semibold()
                        .text_color(paint.secondary_text)
                        .child(format!("{} {number}", dict.details.episode)),
                )
                .child(
                    div()
                        .w_full()
                        .text_size(px(14.))
                        .font_semibold()
                        .line_height(relative(1.3))
                        .text_color(paint.ink)
                        .when(title_rtl, |el| el.text_right())
                        .child(title),
                )
                .when_some(released, |col, released| {
                    col.child(
                        div()
                            .text_size(px(12.))
                            .text_color(paint.secondary_text)
                            .child(released),
                    )
                })
                .child(download_status_line(app, &key, &paint))
                .child(
                    directional_row(h_flex(), rtl)
                        .flex_wrap()
                        .gap(px(8.))
                        .pt(px(4.))
                        .child(apple_button(
                            app,
                            SharedString::from(format!("ep-play-{key}")),
                            dict.actions.watch_now,
                            Some(IconName::Play),
                            AppleButtonKind::Primary,
                            true,
                            cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.watch_sources(
                                    watch_heading_for_play.clone(),
                                    watch_sources_for_play.clone(),
                                    cx,
                                );
                            }),
                            cx,
                        ))
                        .child(apple_button(
                            app,
                            SharedString::from(format!("ep-download-{key}")),
                            download_action_label(app, &key),
                            Some(IconName::ArrowDown),
                            AppleButtonKind::Secondary,
                            true,
                            cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                on_download_click(
                                    this,
                                    key.clone(),
                                    download_heading.clone(),
                                    download_sources.clone(),
                                    window,
                                    cx,
                                );
                            }),
                            cx,
                        )),
                )
                .when(!can_download && job.is_none(), |col| {
                    col.child(
                        div()
                            .text_size(px(11.))
                            .text_color(paint.secondary_text)
                            .child(dict.player.no_download),
                    )
                }),
        )
        .into_any_element()
}

fn movie_download_card(
    app: &StreamovieApp,
    key: &str,
    title: &str,
    hint: &str,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let paint = ApplePaint::from_cx(cx);
    let accent = cx.theme().primary;
    let job = app.download_snapshot(key);
    if job.is_none() {
        return div().into_any_element();
    }
    let fraction = job.as_ref().and_then(progress_fraction);
    let running = job
        .as_ref()
        .is_some_and(|job| job.phase == DownloadPhase::Running);

    v_flex()
        .w_full()
        .max_w(px(720.))
        .gap(px(12.))
        .when(rtl, |col| col.items_end())
        .child(apple_section_title(title, &paint))
        .child(
            v_flex()
                .w_full()
                .gap(px(10.))
                .p(px(16.))
                .rounded(px(16.))
                .bg(paint.surface)
                .border_1()
                .border_color(paint.hairline)
                .shadow(paint.poster_shadow(false))
                .when(rtl, |col| col.items_end())
                .child(
                    div()
                        .text_size(px(13.))
                        .line_height(relative(1.4))
                        .text_color(paint.secondary_text)
                        .when(rtl, |el| el.text_right())
                        .child(if job.is_some() {
                            download_status_text(app, key)
                        } else {
                            hint.to_string()
                        }),
                )
                .when(running || fraction.is_some(), |col| {
                    col.child(apple_progress(
                        fraction.unwrap_or(0.35),
                        if job
                            .as_ref()
                            .is_some_and(|job| job.phase == DownloadPhase::Failed)
                        {
                            paint.danger
                        } else if job
                            .as_ref()
                            .is_some_and(|job| job.phase == DownloadPhase::Finished)
                        {
                            paint.success
                        } else {
                            accent
                        },
                        paint.track,
                    ))
                }),
        )
        .into_any_element()
}

fn similar_rail(
    app: &StreamovieApp,
    category: MediaCategory,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let current_id = app.active_details().map(|details| details.media.id);
    let locale = app.locale();
    let paint = ApplePaint::from_cx(cx);
    let label = locale.dict().details.more_like_this;
    let mut seen = std::collections::HashSet::new();
    let feed = if category == MediaCategory::Series {
        app.home_state().latest_series.clone()
    } else {
        app.home_state().latest_movies.clone()
    };
    let mut items = Vec::new();
    for media in app
        .catalog_state(category)
        .items
        .iter()
        .cloned()
        .chain(feed)
    {
        if Some(media.id) == current_id || !seen.insert(media.id) {
            continue;
        }
        items.push(media);
        if items.len() == 14 {
            break;
        }
    }
    if items.is_empty() {
        let home = app.home_state();
        let catalog = app.catalog_state(category);
        let loading = home.loading || (catalog.loading && catalog.items.is_empty());
        let fetch_error = match category {
            MediaCategory::Movie => home.latest_movies_error.clone().or(catalog.error.clone()),
            MediaCategory::Series => home.latest_series_error.clone().or(catalog.error.clone()),
        };
        if loading && fetch_error.is_none() {
            return v_flex()
                .w_full()
                .min_w_0()
                .gap(px(14.))
                .when(rtl, |col| col.items_end())
                .child(apple_section_title(label, &paint))
                .child(skeleton_row(PosterCardStyle::Poster, 6, rtl))
                .into_any_element();
        }
        if let Some(err) = fetch_error {
            let failed_title = match category {
                MediaCategory::Movie => locale.dict().common.movies_failed,
                MediaCategory::Series => locale.dict().common.series_failed,
            };
            return v_flex()
                .w_full()
                .min_w_0()
                .gap(px(14.))
                .when(rtl, |col| col.items_end())
                .child(apple_section_title(label, &paint))
                .child(network_error_state(
                    locale.dict().common.network_title,
                    failed_title,
                    locale.dict().common.network_body,
                    Some(err),
                    Some(
                        Button::new("similar-retry")
                            .primary()
                            .rounded(px(13.))
                            .icon(IconName::RefreshCw)
                            .label(locale.dict().actions.retry)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.reload_home_feeds(cx);
                                this.reload_catalog(category, cx);
                            }))
                            .into_any_element(),
                    ),
                    cx,
                ))
                .into_any_element();
        }
        return div().into_any_element();
    }

    let cards: Vec<_> = items
        .iter()
        .map(|media| {
            render_poster_card(
                app,
                media,
                category,
                app.is_bookmarked(media.id),
                locale,
                "similar",
                cx,
            )
        })
        .collect();

    v_flex()
        .w_full()
        .min_w_0()
        .gap(px(14.))
        .when(rtl, |col| col.items_end())
        .child(apple_section_title(label, &paint))
        .child(
            div()
                .id("details-similar-rail")
                .w_full()
                .min_w_0()
                .overflow_x_scroll()
                .restrict_scroll_to_axis()
                .child(
                    h_flex()
                        .flex_shrink_0()
                        .gap(px(20.))
                        .py(px(12.))
                        .when(rtl, |row| row.flex_row_reverse())
                        .children(cards),
                ),
        )
        .into_any_element()
}

fn top_bar(
    app: &StreamovieApp,
    bookmarked: bool,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let media = app_media(app);
    let category = app
        .active_details()
        .map(|details| details.category)
        .unwrap_or(MediaCategory::Movie);

    directional_row(h_flex(), rtl)
        .absolute()
        .top(px(20.))
        .left(px(28.))
        .right(px(28.))
        .justify_between()
        .items_center()
        .child(apple_button(
            app,
            "details-back",
            "",
            Some(if rtl {
                IconName::ChevronRight
            } else {
                IconName::ChevronLeft
            }),
            AppleButtonKind::Glass,
            false,
            cx.listener(|this, _, _, cx| this.close_details(cx)),
            cx,
        ))
        .child(apple_button(
            app,
            "details-bookmark-icon",
            "",
            Some(if bookmarked {
                IconName::Check
            } else {
                IconName::Plus
            }),
            AppleButtonKind::Glass,
            false,
            cx.listener(move |this, _, window, cx| {
                let added = this.toggle_bookmark(media.clone(), category, cx);
                window.push_notification(
                    if added {
                        this.locale().dict().bookmarks.add
                    } else {
                        this.locale().dict().bookmarks.remove
                    },
                    cx,
                );
            }),
            cx,
        ))
        .into_any_element()
}

fn poster_row(
    app: &StreamovieApp,
    artwork: &[String],
    label: &str,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let paint = ApplePaint::from_cx(cx);
    let thumbs: Vec<_> = artwork
        .iter()
        .take(8)
        .map(|url| {
            div()
                .w(px(120.))
                .h(px(180.))
                .flex_shrink_0()
                .rounded(px(14.))
                .overflow_hidden()
                .border_1()
                .border_color(paint.hairline)
                .bg(paint.chip)
                .shadow(paint.poster_shadow(false))
                .child(render_remote_image(
                    app,
                    Some(url),
                    None,
                    px(14.),
                    IconName::GalleryVerticalEnd,
                    None,
                    cx,
                ))
                .into_any_element()
        })
        .collect();
    v_flex()
        .w_full()
        .min_w_0()
        .gap(px(14.))
        .when(rtl, |col| col.items_end())
        .child(apple_section_title(label, &paint))
        .child(
            div()
                .id("details-poster-rail")
                .w_full()
                .min_w_0()
                .overflow_x_scroll()
                .restrict_scroll_to_axis()
                .child(
                    directional_row(h_flex(), rtl)
                        .flex_shrink_0()
                        .gap(px(16.))
                        .py(px(8.))
                        .children(thumbs),
                ),
        )
        .into_any_element()
}

fn season_retry_button(label: &str, cx: &mut Context<StreamovieApp>) -> AnyElement {
    Button::new("seasons-retry")
        .primary()
        .rounded(px(13.))
        .icon(IconName::RefreshCw)
        .label(label)
        .on_click(cx.listener(|this, _, _, cx| this.retry_active_seasons(cx)))
        .into_any_element()
}

fn cast_section_skeleton(label: &str, rtl: bool, paint: &ApplePaint) -> AnyElement {
    v_flex()
        .w_full()
        .min_w_0()
        .gap(px(14.))
        .when(rtl, |col| col.items_end())
        .child(apple_section_title(label, paint))
        .child(cast_skeleton_row(rtl))
        .into_any_element()
}

fn cast_row(credits: &[Credit], label: &str, rtl: bool, paint: &ApplePaint) -> AnyElement {
    let people: Vec<_> = credits
        .iter()
        .enumerate()
        .map(|(index, credit)| cast_chip(index, credit, paint))
        .collect();
    v_flex()
        .w_full()
        .min_w_0()
        .gap(px(14.))
        .when(rtl, |col| col.items_end())
        .child(apple_section_title(label, paint))
        .child(
            div()
                .id("details-cast-rail")
                .w_full()
                .min_w_0()
                .overflow_x_scroll()
                .restrict_scroll_to_axis()
                .child(
                    directional_row(h_flex(), rtl)
                        .flex_shrink_0()
                        .gap(px(18.))
                        .py(px(4.))
                        .children(people),
                ),
        )
        .into_any_element()
}

fn overview_block(text: &str, label: &str, rtl: bool, paint: &ApplePaint) -> AnyElement {
    let text_rtl = is_persian_text(text) || rtl;
    v_flex()
        .w_full()
        .max_w(px(720.))
        .gap(px(10.))
        .when(rtl, |col| col.items_end())
        .child(apple_section_title(label, paint))
        .child(
            div()
                .text_size(px(16.))
                .line_height(relative(1.5))
                .text_color(paint.body)
                .when(text_rtl, |el| el.text_right())
                .child(text.to_string()),
        )
        .into_any_element()
}

fn meta_row(
    rating: f64,
    votes: Option<&str>,
    runtime: Option<&str>,
    genres: &[String],
    year: Option<&str>,
    rtl: bool,
    paint: &ApplePaint,
) -> AnyElement {
    let mut facts: Vec<String> = Vec::new();
    if let Some(year) = year {
        facts.push(year.to_string());
    }
    if let Some(runtime) = runtime {
        facts.push(runtime.to_string());
    }
    if !genres.is_empty() {
        facts.push(
            genres
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", "),
        );
    }

    directional_row(h_flex(), rtl)
        .gap(px(10.))
        .items_center()
        .text_size(px(14.))
        .text_color(paint.secondary_text)
        .when(rating > 0.0, |row| {
            row.child(
                directional_row(h_flex(), rtl)
                    .gap(px(4.))
                    .items_center()
                    .child(div().text_size(px(13.)).text_color(paint.star).child("★"))
                    .child(
                        div()
                            .font_semibold()
                            .text_color(paint.ink)
                            .child(format!("{rating:.1}")),
                    )
                    .when_some(votes, |row, votes| {
                        row.child(
                            div()
                                .text_color(paint.secondary_text)
                                .child(votes.to_string()),
                        )
                    }),
            )
        })
        .when(!facts.is_empty(), |row| row.child(facts.join("   ·   ")))
        .into_any_element()
}

fn cast_chip(index: usize, credit: &Credit, paint: &ApplePaint) -> AnyElement {
    let name_rtl = is_persian_text(&credit.name);
    v_flex()
        .id(SharedString::from(format!("cast-{index}")))
        .w(px(88.))
        .flex_shrink_0()
        .items_center()
        .gap(px(8.))
        .child(
            h_flex()
                .size(px(64.))
                .rounded_full()
                .items_center()
                .justify_center()
                .bg(paint.chip)
                .border_1()
                .border_color(paint.hairline)
                .text_size(px(15.))
                .font_semibold()
                .text_color(paint.ink)
                .child(initials_of(&credit.name)),
        )
        .child(
            div()
                .w_full()
                .text_size(px(12.))
                .font_semibold()
                .text_center()
                .line_height(relative(1.25))
                .text_color(paint.ink)
                .when(name_rtl, |el| el.text_right())
                .child(credit.name.clone()),
        )
        .when_some(credit.role.clone(), |col, role| {
            col.child(
                div()
                    .w_full()
                    .text_size(px(11.))
                    .text_center()
                    .text_color(paint.secondary_text)
                    .child(role),
            )
        })
        .into_any_element()
}

fn on_download_click(
    app: &mut StreamovieApp,
    key: String,
    heading: String,
    sources: Vec<SourceLink>,
    window: &mut gpui_kit::Window,
    cx: &mut Context<StreamovieApp>,
) {
    let can_download = best_download_source(&sources).is_some();
    if let Some(job) = app.download_snapshot(&key) {
        match job.phase {
            DownloadPhase::Running => app.cancel_download(&key, cx),
            DownloadPhase::Finished => app.reveal_download(&key, cx),
            _ => {
                if !can_download {
                    window.push_notification(app.locale().dict().player.no_download, cx);
                    return;
                }
                app.start_episode_download(key, heading, sources, window, cx);
            }
        }
        return;
    }
    if !can_download {
        window.push_notification(app.locale().dict().player.no_download, cx);
        return;
    }
    app.start_episode_download(key, heading, sources, window, cx);
}

fn download_action_label(app: &StreamovieApp, key: &str) -> String {
    let dict = app.locale().dict();
    match app.download_snapshot(key).map(|job| job.phase) {
        Some(DownloadPhase::Running) => dict.player.cancel.to_string(),
        Some(DownloadPhase::Finished) => dict.player.show_in_finder.to_string(),
        _ => dict.actions.download.to_string(),
    }
}

fn download_status_text(app: &StreamovieApp, key: &str) -> String {
    let dict = app.locale().dict();
    let Some(job) = app.download_snapshot(key) else {
        return String::new();
    };
    match job.phase {
        DownloadPhase::Running => {
            let percent = progress_fraction(&job).map(|fraction| (fraction * 100.0) as u8);
            match percent {
                Some(percent) => format!("{} {percent}%", dict.player.downloading),
                None => dict.player.downloading.to_string(),
            }
        }
        DownloadPhase::Finished => dict.player.downloaded.to_string(),
        DownloadPhase::Failed => job
            .error
            .unwrap_or_else(|| dict.player.download_failed.to_string()),
        DownloadPhase::Cancelled => dict.player.download_cancelled.to_string(),
    }
}

fn download_status_line(app: &StreamovieApp, key: &str, paint: &ApplePaint) -> AnyElement {
    let text = download_status_text(app, key);
    if text.is_empty() {
        return div().into_any_element();
    }
    let failed = app
        .download_snapshot(key)
        .is_some_and(|job| job.phase == DownloadPhase::Failed);
    div()
        .text_size(px(12.))
        .text_color(if failed {
            paint.danger
        } else {
            paint.secondary_text
        })
        .child(text)
        .into_any_element()
}

fn progress_fraction(job: &DownloadSnapshot) -> Option<f32> {
    job.total
        .filter(|total| *total > 0)
        .map(|total| (job.received as f32 / total as f32).clamp(0.0, 1.0))
}

fn artwork_urls(media: &ApiMedia) -> Vec<String> {
    let mut urls = Vec::new();
    for url in [
        media.poster_image(),
        media.cover_image(),
        media.hero_image(),
    ] {
        let Some(url) = url.filter(|value| !value.trim().is_empty()) else {
            continue;
        };
        if !urls.iter().any(|existing| existing == &url) {
            urls.push(url);
        }
    }
    urls.truncate(4);
    urls
}

fn initials_of(name: &str) -> String {
    let mut letters = String::new();
    for word in name.split_whitespace().take(2) {
        if let Some(ch) = word.chars().next() {
            letters.push(ch);
        }
    }
    if letters.is_empty() {
        "?".to_string()
    } else {
        letters
    }
}

fn app_media(app: &StreamovieApp) -> ApiMedia {
    app.active_details()
        .map(|details| details.media.clone())
        .unwrap_or_default()
}
