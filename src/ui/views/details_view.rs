//! Movie and series detail pages.

use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _, WindowExt as _, h_flex,
    spinner::Spinner, v_flex,
};
use gpui_kit::{
    AnyElement, App, ClickEvent, Context, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    hsla, linear_color_stop, linear_gradient, prelude::FluentBuilder as _, px, relative, white,
};

use crate::{
    app::StreamovieApp,
    download::DownloadPhase,
    i18n::{directional_row, is_persian_text},
    models::{ApiMedia, Episode, MediaCategory, OmdbDetails, SourceLink},
    state::DetailsState,
    ui::{components::render_poster_card, image_cache::render_remote_image},
};

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
    let cast = details
        .omdb_details
        .as_ref()
        .and_then(|item| OmdbDetails::non_na(&item.actors))
        .map(|actors| {
            actors
                .split(',')
                .map(|name| name.trim().to_string())
                .filter(|name| !name.is_empty())
                .take(5)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let artwork = artwork_urls(media);
    let poster = media.poster_image();
    let cover = media.hero_image().or(poster.clone());
    let accent = cx.theme().primary;
    let stage = hsla(0.0, 0.0, 0.04, 1.0);
    let sources = media.source_links();
    let has_trailer = sources.iter().any(|source| source.is_trailer);

    let body = if category == MediaCategory::Series {
        render_series_page(
            app,
            details,
            &title,
            title_rtl,
            overview.as_deref(),
            year.as_deref(),
            runtime.as_deref(),
            rating,
            votes.as_deref(),
            &genres,
            &cast,
            cover.as_deref(),
            poster.as_deref(),
            bookmarked,
            accent,
            stage,
            rtl,
            cx,
        )
    } else {
        render_movie_page(
            app,
            &title,
            title_rtl,
            overview.as_deref(),
            year.as_deref(),
            runtime.as_deref(),
            rating,
            votes.as_deref(),
            &genres,
            &cast,
            &artwork,
            cover.as_deref(),
            poster.as_deref(),
            bookmarked,
            has_trailer,
            &sources,
            category,
            accent,
            stage,
            rtl,
            cx,
        )
    };

    body
}

fn render_movie_page(
    app: &StreamovieApp,
    title: &str,
    title_rtl: bool,
    overview: Option<&str>,
    year: Option<&str>,
    runtime: Option<&str>,
    rating: f64,
    votes: Option<&str>,
    genres: &[String],
    cast: &[String],
    artwork: &[String],
    cover: Option<&str>,
    poster: Option<&str>,
    bookmarked: bool,
    has_trailer: bool,
    sources: &[SourceLink],
    category: MediaCategory,
    accent: Hsla,
    stage: Hsla,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let media_for_watch = app_media(app);
    let media_for_bookmark = media_for_watch.clone();
    let trailer_sources = sources.to_vec();
    let watch_title = title.to_string();
    let trailer_heading = format!("{watch_title} · {}", dict.actions.watch_trailer);

    let hero = div()
        .relative()
        .w_full()
        .h(px(640.))
        .overflow_hidden()
        .bg(stage)
        .child(backdrop(app, cover, poster, stage, rtl, cx))
        .child(top_bar(app, bookmarked, rtl, cx))
        .child(
            v_flex()
                .absolute()
                .bottom(px(36.))
                .w(px(480.))
                .gap(px(14.))
                .when(!rtl, |col| col.left(px(40.)))
                .when(rtl, |col| col.right(px(40.)).items_end().text_right())
                .child(
                    div()
                        .text_size(px(48.))
                        .font_bold()
                        .line_height(relative(1.0))
                        .text_color(white())
                        .when(title_rtl, |el| el.text_right())
                        .child(title.to_string()),
                )
                .child(meta_row(rating, votes, runtime, genres, year, rtl))
                .when_some(overview, |col, text| {
                    let shown: String = text.chars().take(180).collect();
                    let text_rtl = is_persian_text(&shown) || rtl;
                    col.child(
                        div()
                            .max_w(px(420.))
                            .text_size(px(14.))
                            .line_height(relative(1.45))
                            .text_color(hsla(0.0, 0.0, 0.86, 0.86))
                            .when(text_rtl, |el| el.text_right())
                            .child(shown),
                    )
                })
                .child(
                    directional_row(h_flex(), rtl)
                        .gap(px(12.))
                        .pt(px(6.))
                        .child(action_pill(
                            "details-watch".into(),
                            dict.actions.watch_now,
                            IconName::Play,
                            true,
                            accent,
                            cx.listener(move |this, _, _, cx| {
                                this.watch_title(media_for_watch.clone(), category, cx);
                            }),
                        ))
                        .when(has_trailer, |row| {
                            row.child(action_pill(
                                "details-trailer".into(),
                                dict.actions.watch_trailer,
                                IconName::Play,
                                false,
                                accent,
                                cx.listener(move |this, _, _, cx| {
                                    this.watch_trailer(
                                        trailer_heading.clone(),
                                        trailer_sources.clone(),
                                        cx,
                                    );
                                }),
                            ))
                        })
                        .when(!has_trailer, |row| {
                            row.child(action_pill(
                                "details-save".into(),
                                if bookmarked {
                                    dict.bookmarks.remove
                                } else {
                                    dict.bookmarks.add
                                },
                                if bookmarked {
                                    IconName::Check
                                } else {
                                    IconName::Plus
                                },
                                false,
                                accent,
                                cx.listener(move |this, _, window, cx| {
                                    let added = this.toggle_bookmark(
                                        media_for_bookmark.clone(),
                                        category,
                                        cx,
                                    );
                                    window.push_notification(
                                        if added {
                                            this.locale().dict().bookmarks.add
                                        } else {
                                            this.locale().dict().bookmarks.remove
                                        },
                                        cx,
                                    );
                                }),
                            ))
                        }),
                ),
        );

    v_flex()
        .w_full()
        .min_w_0()
        .bg(stage)
        .child(hero)
        .child(
            v_flex()
                .w_full()
                .min_w_0()
                .px(px(40.))
                .pt(px(28.))
                .pb(px(48.))
                .gap(px(36.))
                .when(!artwork.is_empty(), |col| {
                    col.child(poster_row(app, artwork, dict.details.posters, rtl, cx))
                })
                .when(!cast.is_empty(), |col| {
                    col.child(cast_row(cast, dict.details.cast, rtl))
                })
                .child(similar_rail(app, category, rtl, cx)),
        )
        .into_any_element()
}

fn render_series_page(
    app: &StreamovieApp,
    details: &DetailsState,
    title: &str,
    title_rtl: bool,
    overview: Option<&str>,
    year: Option<&str>,
    runtime: Option<&str>,
    rating: f64,
    votes: Option<&str>,
    genres: &[String],
    cast: &[String],
    cover: Option<&str>,
    poster: Option<&str>,
    bookmarked: bool,
    accent: Hsla,
    stage: Hsla,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let media_for_watch = app_media(app);

    v_flex()
        .w_full()
        .min_w_0()
        .bg(stage)
        .child(
            div()
                .relative()
                .w_full()
                .h(px(560.))
                .overflow_hidden()
                .child(backdrop(app, cover, poster, stage, rtl, cx))
                .child(top_bar(app, bookmarked, rtl, cx))
                .child(
                    v_flex()
                        .absolute()
                        .bottom(px(36.))
                        .w(px(560.))
                        .gap(px(12.))
                        .when(!rtl, |col| col.left(px(40.)))
                        .when(rtl, |col| col.right(px(40.)).items_end().text_right())
                        .child(
                            div()
                                .text_size(px(44.))
                                .font_bold()
                                .line_height(relative(1.05))
                                .text_color(white())
                                .when(title_rtl, |el| el.text_right())
                                .child(title.to_string()),
                        )
                        .child(meta_row(rating, votes, runtime, genres, year, rtl))
                        .when_some(overview, |col, text| {
                            let shown: String = text.chars().take(180).collect();
                            let text_rtl = is_persian_text(&shown) || rtl;
                            col.child(
                                div()
                                    .max_w(px(480.))
                                    .text_size(px(14.))
                                    .line_height(relative(1.45))
                                    .text_color(hsla(0.0, 0.0, 0.86, 0.86))
                                    .when(text_rtl, |el| el.text_right())
                                    .child(shown),
                            )
                        })
                        .child(action_pill(
                            "series-watch".into(),
                            dict.actions.watch_now,
                            IconName::Play,
                            true,
                            accent,
                            cx.listener(move |this, _, _, cx| {
                                this.watch_title(
                                    media_for_watch.clone(),
                                    MediaCategory::Series,
                                    cx,
                                );
                            }),
                        )),
                ),
        )
        .child(
            v_flex()
                .w_full()
                .min_w_0()
                .px(px(40.))
                .pt(px(28.))
                .pb(px(48.))
                .gap(px(36.))
                .child(
                    v_flex()
                        .w_full()
                        .min_w_0()
                        .gap(px(14.))
                        .child(section_label(dict.player.episodes))
                        .child(render_episodes(app, details, accent, rtl, cx)),
                )
                .when(!cast.is_empty(), |col| {
                    col.child(cast_row(cast, dict.details.cast, rtl))
                })
                .child(similar_rail(app, MediaCategory::Series, rtl, cx)),
        )
        .into_any_element()
}

fn render_episodes(
    app: &StreamovieApp,
    details: &DetailsState,
    accent: Hsla,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();

    if details.seasons_loading {
        return directional_row(h_flex(), rtl)
            .gap_2()
            .child(Spinner::new().small())
            .child(
                div()
                    .text_size(px(14.))
                    .text_color(white().opacity(0.6))
                    .child(dict.common.loading),
            )
            .into_any_element();
    }

    if details.seasons.is_empty() {
        return div()
            .text_size(px(14.))
            .text_color(white().opacity(0.65))
            .child(dict.details.no_seasons)
            .into_any_element();
    }

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
                accent,
                rtl,
                cx,
            )
        })
        .collect();

    v_flex()
        .w_full()
        .min_w_0()
        .gap(px(18.))
        .child(directional_row(h_flex(), rtl).gap(px(22.)).children(
            details.seasons.iter().enumerate().map(|(index, season)| {
                let active = index == selected;
                let label = season.display_title(index);
                div()
                    .id(SharedString::from(format!("season-{series_id}-{index}")))
                    .cursor_pointer()
                    .text_size(px(14.))
                    .when(active, |el| el.font_semibold().text_color(white()))
                    .when(!active, |el| el.text_color(white().opacity(0.42)))
                    .child(label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.select_season(index, cx);
                    }))
                    .into_any_element()
            }),
        ))
        .child(if cards.is_empty() {
            div()
                .text_size(px(14.))
                .text_color(white().opacity(0.6))
                .child(dict.details.no_episodes)
                .into_any_element()
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
                        .gap(px(16.))
                        .py(px(8.))
                        .when(rtl, |row| row.flex_row_reverse())
                        .children(cards),
                )
                .into_any_element()
        })
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
    accent: Hsla,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
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
    let can_download = crate::models::best_download_source(&sources).is_some();
    let watch_sources = sources.clone();
    let download_sources = sources;
    let watch_heading = format!("{series_title} · {title}");
    let download_heading = watch_heading.clone();
    let title_rtl = is_persian_text(&title) || rtl;

    v_flex()
        .id(SharedString::from(format!("ep-card-{key}")))
        .w(px(220.))
        .flex_shrink_0()
        .gap(px(8.))
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| {
            this.watch_sources(watch_heading.clone(), watch_sources.clone(), cx);
        }))
        .child(
            div()
                .relative()
                .w(px(220.))
                .h(px(124.))
                .rounded(px(12.))
                .overflow_hidden()
                .child(render_remote_image(
                    app,
                    thumb,
                    thumb_fallback,
                    px(8.),
                    IconName::Frame,
                    None,
                    cx,
                )),
        )
        .child(
            div()
                .text_size(px(11.))
                .font_semibold()
                .text_color(accent)
                .child(format!("{} {number}", dict.details.episode)),
        )
        .child(
            div()
                .text_size(px(13.))
                .font_semibold()
                .text_color(white())
                .when(title_rtl, |el| el.text_right())
                .child(title),
        )
        .when_some(released, |col, released| {
            col.child(
                div()
                    .text_size(px(12.))
                    .line_height(relative(1.4))
                    .text_color(white().opacity(0.62))
                    .child(released),
            )
        })
        .child(download_status_line(app, &key, cx))
        .child(
            div()
                .id(SharedString::from(format!("ep-download-{key}")))
                .text_size(px(12.))
                .text_color(white().opacity(0.72))
                .cursor_pointer()
                .child(download_label(app, &key, can_download))
                .on_click(cx.listener(move |this, _, window, cx| {
                    cx.stop_propagation();
                    if let Some(job) = this.download_snapshot(&key) {
                        match job.phase {
                            DownloadPhase::Running => this.cancel_download(&key, cx),
                            DownloadPhase::Finished => this.reveal_download(&key, cx),
                            _ => {
                                if !can_download {
                                    window.push_notification(
                                        this.locale().dict().player.no_download,
                                        cx,
                                    );
                                    return;
                                }
                                this.start_episode_download(
                                    key.clone(),
                                    download_heading.clone(),
                                    download_sources.clone(),
                                    window,
                                    cx,
                                );
                            }
                        }
                        return;
                    }
                    if !can_download {
                        window.push_notification(this.locale().dict().player.no_download, cx);
                        return;
                    }
                    this.start_episode_download(
                        key.clone(),
                        download_heading.clone(),
                        download_sources.clone(),
                        window,
                        cx,
                    );
                })),
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
        .gap(px(12.))
        .child(
            div()
                .text_size(px(20.))
                .font_semibold()
                .text_color(white())
                .child(label),
        )
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
                        .gap(px(12.))
                        .py(px(8.))
                        .when(rtl, |row| row.flex_row_reverse())
                        .children(cards),
                ),
        )
        .into_any_element()
}

fn backdrop(
    app: &StreamovieApp,
    cover: Option<&str>,
    poster: Option<&str>,
    stage: Hsla,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    div()
        .absolute()
        .inset_0()
        .child(render_remote_image(
            app,
            cover,
            poster,
            px(0.),
            IconName::Frame,
            None,
            cx,
        ))
        .child(div().absolute().inset_0().bg(linear_gradient(
            0.0,
            linear_color_stop(stage, 0.0),
            linear_color_stop(stage.opacity(0.15), 0.62),
        )))
        .child(div().absolute().inset_0().bg(linear_gradient(
            if rtl { 270.0 } else { 90.0 },
            linear_color_stop(stage.opacity(0.72), 0.0),
            linear_color_stop(stage.opacity(0.05), 0.72),
        )))
        .into_any_element()
}

fn top_bar(
    app: &StreamovieApp,
    bookmarked: bool,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let media = app_media(app);
    let category = app
        .active_details()
        .map(|details| details.category)
        .unwrap_or(MediaCategory::Movie);

    directional_row(h_flex(), rtl)
        .absolute()
        .top(px(22.))
        .left(px(28.))
        .right(px(28.))
        .justify_between()
        .items_center()
        .child(
            directional_row(h_flex(), rtl)
                .gap(px(22.))
                .items_center()
                .child(
                    div()
                        .id("details-back")
                        .cursor_pointer()
                        .child(
                            Icon::new(if rtl {
                                IconName::ChevronRight
                            } else {
                                IconName::ChevronLeft
                            })
                            .small()
                            .text_color(white()),
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.close_details(cx))),
                )
                .child(
                    div()
                        .text_size(px(18.))
                        .font_bold()
                        .text_color(white())
                        .child(dict.brand.name),
                )
                .child(nav_caption(dict.details.cast))
                .child(nav_caption(dict.details.posters)),
        )
        .child(
            div()
                .id("details-bookmark-icon")
                .cursor_pointer()
                .child(
                    Icon::new(if bookmarked {
                        IconName::Check
                    } else {
                        IconName::Plus
                    })
                    .small()
                    .text_color(white()),
                )
                .on_click(cx.listener(move |this, _, window, cx| {
                    let added = this.toggle_bookmark(media.clone(), category, cx);
                    window.push_notification(
                        if added {
                            this.locale().dict().bookmarks.add
                        } else {
                            this.locale().dict().bookmarks.remove
                        },
                        cx,
                    );
                })),
        )
        .into_any_element()
}

fn poster_row(
    app: &StreamovieApp,
    artwork: &[String],
    label: &str,
    rtl: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let thumbs: Vec<_> = artwork
        .iter()
        .take(8)
        .map(|url| {
            div()
                .w(px(160.))
                .h(px(90.))
                .flex_shrink_0()
                .rounded(px(8.))
                .overflow_hidden()
                .child(render_remote_image(
                    app,
                    Some(url),
                    None,
                    px(8.),
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
        .gap(px(12.))
        .when(rtl, |col| col.items_end())
        .child(section_label(label))
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
                        .gap(px(10.))
                        .children(thumbs),
                ),
        )
        .into_any_element()
}

fn cast_row(cast: &[String], label: &str, rtl: bool) -> AnyElement {
    let people: Vec<_> = cast
        .iter()
        .enumerate()
        .map(|(index, name)| cast_chip(index, name))
        .collect();
    v_flex()
        .gap(px(12.))
        .when(rtl, |col| col.items_end())
        .child(section_label(label))
        .child(directional_row(h_flex(), rtl).gap(px(14.)).children(people))
        .into_any_element()
}

fn meta_row(
    rating: f64,
    votes: Option<&str>,
    runtime: Option<&str>,
    genres: &[String],
    year: Option<&str>,
    rtl: bool,
) -> AnyElement {
    let mut facts: Vec<String> = Vec::new();
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
    if let Some(year) = year {
        facts.push(year.to_string());
    }

    directional_row(h_flex(), rtl)
        .gap(px(10.))
        .items_center()
        .text_size(px(13.))
        .text_color(hsla(0.0, 0.0, 0.82, 0.82))
        .when(rating > 0.0, |row| {
            row.child(
                directional_row(h_flex(), rtl)
                    .gap(px(6.))
                    .items_center()
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(hsla(45.0 / 360.0, 0.95, 0.58, 1.0))
                            .child("★"),
                    )
                    .child(format!("{rating:.1}"))
                    .when_some(votes, |row, votes| {
                        row.child(
                            div()
                                .text_color(white().opacity(0.45))
                                .child(format!("| {votes}")),
                        )
                    }),
            )
        })
        .when(!facts.is_empty(), |row| row.child(facts.join("  •  ")))
        .into_any_element()
}

fn action_pill(
    id: SharedString,
    label: &str,
    icon: IconName,
    filled: bool,
    _accent: Hsla,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    h_flex()
        .id(id)
        .h(px(36.))
        .px(px(18.))
        .gap(px(8.))
        .rounded(px(4.))
        .items_center()
        .justify_center()
        .cursor_pointer()
        .bg(if filled {
            white()
        } else {
            hsla(0.0, 0.0, 1.0, 0.18)
        })
        .text_size(px(14.))
        .font_semibold()
        .text_color(if filled {
            hsla(0.0, 0.0, 0.08, 1.0)
        } else {
            white()
        })
        .child(Icon::new(icon).small().text_color(if filled {
            hsla(0.0, 0.0, 0.08, 1.0)
        } else {
            white()
        }))
        .child(label.to_string())
        .on_click(on_click)
        .into_any_element()
}

fn nav_caption(label: &str) -> AnyElement {
    div()
        .text_size(px(12.))
        .font_semibold()
        .text_color(white().opacity(0.92))
        .child(label.to_string())
        .into_any_element()
}

fn section_label(label: &str) -> AnyElement {
    div()
        .text_size(px(13.))
        .font_semibold()
        .text_color(white())
        .child(label.to_string())
        .into_any_element()
}

fn cast_chip(index: usize, name: &str) -> AnyElement {
    let initials = initials_of(name);
    let mut words = name.split_whitespace();
    let first = words.next().unwrap_or(name).to_string();
    let rest = words.collect::<Vec<_>>().join(" ");
    v_flex()
        .id(SharedString::from(format!("cast-{index}")))
        .w(px(72.))
        .items_center()
        .gap(px(6.))
        .child(
            h_flex()
                .w(px(52.))
                .h(px(52.))
                .rounded_full()
                .items_center()
                .justify_center()
                .bg(hsla(0.0, 0.0, 0.18, 0.85))
                .border_1()
                .border_color(white().opacity(0.22))
                .text_size(px(13.))
                .font_semibold()
                .text_color(white())
                .child(initials),
        )
        .child(
            div()
                .w_full()
                .text_size(px(11.))
                .text_center()
                .line_height(relative(1.25))
                .text_color(white().opacity(0.88))
                .child(first),
        )
        .when(!rest.is_empty(), |col| {
            col.child(
                div()
                    .w_full()
                    .text_size(px(11.))
                    .text_center()
                    .text_color(white().opacity(0.88))
                    .child(rest),
            )
        })
        .into_any_element()
}

fn download_label(app: &StreamovieApp, key: &str, can_download: bool) -> String {
    let dict = app.locale().dict();
    match app.download_snapshot(key).map(|job| job.phase) {
        Some(DownloadPhase::Running) => dict.player.cancel.to_string(),
        Some(DownloadPhase::Finished) => dict.player.show_in_finder.to_string(),
        _ if can_download => dict.actions.download.to_string(),
        _ => dict.player.no_download.to_string(),
    }
}

fn download_status_line(
    app: &StreamovieApp,
    key: &str,
    _cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let Some(job) = app.download_snapshot(key) else {
        return div().into_any_element();
    };
    let label = match job.phase {
        DownloadPhase::Running => {
            let percent = job.total.filter(|total| *total > 0).map(|total| {
                ((job.received as f64 / total as f64) * 100.0).clamp(0.0, 100.0) as u8
            });
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
    };
    div()
        .text_size(px(12.))
        .text_color(white().opacity(0.6))
        .child(label)
        .into_any_element()
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
