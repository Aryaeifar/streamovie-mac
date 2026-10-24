//! Movie and series detail pages.

use gpui_kit::component::{
    ActiveTheme as _, IconName, Selectable as _, Sizable as _, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    empty::{Empty, EmptyDescription, EmptyHeader, EmptyTitle},
    h_flex,
    spinner::Spinner,
    v_flex,
};
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, SharedString, Styled as _, div, hsla,
    linear_color_stop, linear_gradient, prelude::FluentBuilder as _, px, relative, white,
};

use crate::{
    app::StreamovieApp,
    download::DownloadPhase,
    i18n::{directional_row, format_count, is_persian_text},
    models::{MediaCategory, best_download_source, best_stream_source},
    state::DetailsState,
    ui::image_cache::render_remote_image,
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
    let title = media.display_title();
    let title_rtl = is_persian_text(&title) || rtl;
    let bookmarked = app.is_bookmarked(media.id);
    let parsed = media.parsed_description();
    let overview = parsed.overview.clone().or_else(|| {
        details
            .omdb_details
            .as_ref()
            .and_then(|item| item.plot.clone())
    });
    let year = media.release_year().or_else(|| {
        details
            .omdb_details
            .as_ref()
            .and_then(|item| item.year.clone())
    });
    let runtime = media.duration_label().or_else(|| {
        details
            .omdb_details
            .as_ref()
            .and_then(|item| item.runtime.clone())
    });
    let rating = media.rating_value();
    let genres = media.genre_list();
    let stage = hsla(0.0, 0.0, 0.03, 1.0);
    let poster = media.poster_image();
    let cover = media.hero_image().or(poster.clone());

    let media_for_watch = media.clone();
    let media_for_bookmark = media.clone();

    v_flex()
        .w_full()
        .pb_16()
        .child(
            div()
                .relative()
                .w_full()
                .h(px(460.))
                .overflow_hidden()
                .bg(stage)
                .child(div().absolute().inset_0().child(render_remote_image(
                    app,
                    cover.as_deref(),
                    poster.as_deref(),
                    px(0.),
                    IconName::Frame,
                    None,
                    cx,
                )))
                .child(div().absolute().inset_0().bg(linear_gradient(
                    0.0,
                    linear_color_stop(stage, 0.0),
                    linear_color_stop(stage.opacity(0.05), 0.7),
                )))
                .child(div().absolute().inset_0().bg(linear_gradient(
                    if rtl { 270.0 } else { 90.0 },
                    linear_color_stop(stage.opacity(0.94), 0.0),
                    linear_color_stop(stage.opacity(0.1), 0.75),
                )))
                .child(
                    v_flex()
                        .absolute()
                        .left(px(40.))
                        .right(px(40.))
                        .bottom(px(28.))
                        .gap_3()
                        .max_w(px(760.))
                        .when(rtl, |col| col.items_end().text_right())
                        .child(
                            Button::new("details-back")
                                .outline()
                                .small()
                                .icon(if rtl {
                                    IconName::ChevronRight
                                } else {
                                    IconName::ChevronLeft
                                })
                                .label(dict.details.back)
                                .on_click(cx.listener(|this, _, _, cx| this.close_details(cx))),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(white().opacity(0.7))
                                .child(if category == MediaCategory::Series {
                                    dict.details.series
                                } else {
                                    dict.details.movie
                                }),
                        )
                        .child(
                            div()
                                .text_size(px(42.))
                                .font_bold()
                                .line_height(relative(1.05))
                                .text_color(white())
                                .when(title_rtl, |el| el.text_right())
                                .child(title.clone()),
                        )
                        .child(
                            directional_row(h_flex(), rtl)
                                .gap_3()
                                .text_sm()
                                .text_color(white().opacity(0.8))
                                .when_some(year, |row, year| row.child(year))
                                .when_some(runtime, |row, runtime| row.child(runtime))
                                .when(rating > 0.0, |row| row.child(format!("{rating:.1}")))
                                .when(!genres.is_empty(), |row| {
                                    row.child(
                                        genres.into_iter().take(3).collect::<Vec<_>>().join(" · "),
                                    )
                                }),
                        )
                        .when_some(overview, |col, text| {
                            let shown: String = text.chars().take(280).collect();
                            let text_rtl = is_persian_text(&shown) || rtl;
                            col.child(
                                div()
                                    .max_w(px(620.))
                                    .text_sm()
                                    .line_height(relative(1.5))
                                    .text_color(hsla(0.0, 0.0, 0.92, 0.92))
                                    .when(text_rtl, |el| el.text_right())
                                    .child(shown),
                            )
                        })
                        .child(
                            directional_row(h_flex(), rtl)
                                .pt_1()
                                .gap_3()
                                .child(
                                    Button::new("details-watch")
                                        .primary()
                                        .large()
                                        .icon(IconName::Play)
                                        .label(dict.actions.watch_now)
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.watch_title(media_for_watch.clone(), category, cx);
                                        })),
                                )
                                .child(
                                    Button::new("details-bookmark")
                                        .large()
                                        .outline()
                                        .selected(bookmarked)
                                        .icon(if bookmarked {
                                            IconName::Check
                                        } else {
                                            IconName::Plus
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
                                            window.push_notification(
                                                if added {
                                                    this.locale().dict().bookmarks.add
                                                } else {
                                                    this.locale().dict().bookmarks.remove
                                                },
                                                cx,
                                            );
                                        })),
                                ),
                        ),
                ),
        )
        .child(if category == MediaCategory::Series {
            render_episodes(app, details, cx)
        } else {
            render_movie_note(app, details, cx)
        })
        .into_any_element()
}

fn render_movie_note(
    app: &StreamovieApp,
    details: &DetailsState,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = app.locale().dict();
    let sources = details.media.source_links();
    let body = best_stream_source(&sources)
        .map(|source| source.quality.clone())
        .unwrap_or_else(|| dict.player.unavailable_body.to_string());
    div()
        .px_10()
        .pt_6()
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .child(body)
        .into_any_element()
}

fn render_episodes(
    app: &StreamovieApp,
    details: &DetailsState,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();

    if details.seasons_loading {
        return directional_row(h_flex(), rtl)
            .px_10()
            .pt_6()
            .gap_2()
            .child(Spinner::new().small())
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(dict.common.loading),
            )
            .into_any_element();
    }

    if details.seasons.is_empty() {
        return div()
            .px_10()
            .pt_6()
            .child(
                Empty::new().header(
                    EmptyHeader::new()
                        .title(EmptyTitle::new().child(dict.details.no_seasons))
                        .description(EmptyDescription::new().child(dict.details.no_seasons_hint)),
                ),
            )
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

    let rows: Vec<_> = episodes
        .iter()
        .enumerate()
        .map(|(index, episode)| {
            let number = index + 1;
            let omdb = omdb_episodes.iter().find(|item| {
                item.episode
                    .as_deref()
                    .and_then(|n| n.parse::<usize>().ok())
                    == Some(number)
            });
            let title = omdb
                .and_then(|item| item.title.clone())
                .filter(|title| !title.is_empty() && title != "N/A")
                .unwrap_or_else(|| episode.title.clone());
            let key = format!("{series_id}-s{}-e{number}", selected + 1);
            let sources = episode.sources.clone();
            let can_download = best_download_source(&sources).is_some();
            let job = app.download_snapshot(&key);
            let watch_sources = sources.clone();
            let download_sources = sources.clone();
            let watch_title = format!("{series_title} · {title}");
            let download_title = watch_title.clone();

            directional_row(h_flex(), rtl)
                .w_full()
                .py_3()
                .gap_4()
                .border_b_1()
                .border_color(cx.theme().border)
                .justify_between()
                .items_center()
                .child(
                    directional_row(h_flex(), rtl)
                        .gap_4()
                        .min_w_0()
                        .items_center()
                        .child(
                            div()
                                .w(px(36.))
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("{number:02}")),
                        )
                        .child(
                            v_flex()
                                .min_w_0()
                                .gap_0p5()
                                .when(rtl, |col| col.items_end().text_right())
                                .child(div().text_sm().font_semibold().child(title))
                                .child(download_status_line(app, &key, cx)),
                        ),
                )
                .child(
                    directional_row(h_flex(), rtl)
                        .gap_2()
                        .flex_shrink_0()
                        .child(
                            Button::new(SharedString::from(format!("ep-watch-{key}")))
                                .small()
                                .primary()
                                .icon(IconName::Play)
                                .label(dict.actions.watch_now)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.watch_sources(
                                        watch_title.clone(),
                                        watch_sources.clone(),
                                        cx,
                                    );
                                })),
                        )
                        .child(
                            if job
                                .as_ref()
                                .is_some_and(|job| job.phase == DownloadPhase::Running)
                            {
                                Button::new(SharedString::from(format!("ep-cancel-{key}")))
                                    .small()
                                    .outline()
                                    .label(dict.player.cancel)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.cancel_download(&key, cx);
                                    }))
                                    .into_any_element()
                            } else if job
                                .as_ref()
                                .is_some_and(|job| job.phase == DownloadPhase::Finished)
                            {
                                Button::new(SharedString::from(format!("ep-reveal-{key}")))
                                    .small()
                                    .outline()
                                    .label(dict.player.show_in_finder)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.reveal_download(&key, cx);
                                    }))
                                    .into_any_element()
                            } else {
                                Button::new(SharedString::from(format!("ep-download-{key}")))
                                    .small()
                                    .outline()
                                    .icon(IconName::ArrowDown)
                                    .label(if can_download {
                                        dict.actions.download
                                    } else {
                                        dict.player.no_download
                                    })
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        if !can_download {
                                            window.push_notification(
                                                this.locale().dict().player.no_download,
                                                cx,
                                            );
                                            return;
                                        }
                                        this.start_episode_download(
                                            key.clone(),
                                            download_title.clone(),
                                            download_sources.clone(),
                                            window,
                                            cx,
                                        );
                                    }))
                                    .into_any_element()
                            },
                        ),
                )
                .into_any_element()
        })
        .collect();

    v_flex()
        .w_full()
        .px_10()
        .pt_6()
        .gap_4()
        .child(
            directional_row(h_flex(), rtl)
                .w_full()
                .justify_between()
                .items_center()
                .child(div().text_lg().font_semibold().child(dict.player.episodes))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(format_count(
                            dict.details.episodes_count,
                            rows.len(),
                            locale,
                        )),
                ),
        )
        .child(directional_row(h_flex(), rtl).gap_2().flex_wrap().children(
            details.seasons.iter().enumerate().map(|(index, season)| {
                let selected_season = index == selected;
                Button::new(SharedString::from(format!("season-{series_id}-{index}")))
                    .small()
                    .selected(selected_season)
                    .when(selected_season, |btn| btn.primary())
                    .when(!selected_season, |btn| btn.outline())
                    .label(season.display_title(index))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.select_season(index, cx);
                    }))
            }),
        ))
        .child(if rows.is_empty() {
            Empty::new()
                .header(
                    EmptyHeader::new()
                        .title(EmptyTitle::new().child(dict.details.no_episodes))
                        .description(
                            EmptyDescription::new().child(dict.details.select_season_first),
                        ),
                )
                .into_any_element()
        } else {
            v_flex().w_full().children(rows).into_any_element()
        })
        .into_any_element()
}

fn download_status_line(
    app: &StreamovieApp,
    key: &str,
    cx: &mut Context<StreamovieApp>,
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
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .child(label)
        .into_any_element()
}
