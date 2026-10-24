//! Streamovie stream source actions: primary Play, secondary Download, tertiary Copy.

use gpui_kit::{
    AnyElement, BoxShadow, ClipboardItem, Context, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, Styled as _, div, hsla, point, prelude::FluentBuilder as _,
    px, white,
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle},
    h_flex,
    tag::Tag,
    v_flex,
};

use crate::{
    app::StreamovieApp,
    i18n::{Locale, directional_row},
    models::SourceLink,
    storage::DownloadStatus,
    ui::theme_tokens,
};

/// Render quality source cards with Play / Download / Copy actions.
///
/// `media_id` and `episode_key` identify the title (and optional episode) for player + downloads.
#[allow(clippy::too_many_arguments)]
pub fn render_source_links(
    app: &StreamovieApp,
    sources: &[SourceLink],
    context_title: &str,
    locale: Locale,
    id_prefix: &str,
    media_id: u64,
    episode_key: Option<&str>,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let primary_color = cx.theme().primary;
    let is_dark = cx.theme().is_dark();

    let downloads: Vec<SourceLink> = sources
        .iter()
        .filter(|s| !s.is_trailer)
        .cloned()
        .collect();

    if downloads.is_empty() {
        return Empty::new()
            .header(
                EmptyHeader::new()
                    .media(
                        EmptyMedia::new()
                            .with_variant(EmptyMediaVariant::Icon)
                            .child(Icon::new(IconName::ArrowDown)),
                    )
                    .title(EmptyTitle::new().child(dict.details.no_links))
                    .description(EmptyDescription::new().child(dict.details.no_links_hint)),
            )
            .into_any_element();
    }

    let surface_bg = theme_tokens::elevated_surface(is_dark, cx.theme().popover);
    let border_col = theme_tokens::quiet_border(is_dark, cx.theme().border);

    let mut source_cards = Vec::with_capacity(downloads.len());
    for (ix, source) in downloads.into_iter().enumerate() {
        let quality = source.quality.clone();
        let format_badge = source
            .format_type
            .clone()
            .unwrap_or_else(|| "Direct stream".to_string());
        let is_playable = source.is_playable_video();
        let is_hls = source.is_hls();
        let download_key = format!(
            "{}:{}:{}",
            media_id,
            episode_key.unwrap_or("movie"),
            source.id
        );
        let dl_status = app.download_status(&download_key);
        let url_for_copy = source.url.clone();
        let copy_notice = dict.actions.copied.to_string();
        let source_for_play = source.clone();
        let source_for_dl = source.clone();
        let title_for_play = context_title.to_string();
        let title_for_dl = context_title.to_string();
        let ep_key_owned = episode_key.map(|s| s.to_string());
        let dl_key_owned = download_key.clone();

        let play_btn_id = SharedString::from(format!("src-play-{id_prefix}-{}-{ix}", source.id));
        let dl_btn_id = SharedString::from(format!("src-dl-{id_prefix}-{}-{ix}", source.id));
        let copy_btn_id = SharedString::from(format!("src-copy-{id_prefix}-{}-{ix}", source.id));
        let ext_btn_id = SharedString::from(format!("src-ext-{id_prefix}-{}-{ix}", source.id));

        let status_label = match &dl_status {
            DownloadStatus::Downloading { progress } => {
                if *progress > 0.0 {
                    format!("{} {:.0}%", dict.actions.download_in_progress, progress * 100.0)
                } else {
                    dict.actions.download_in_progress.to_string()
                }
            }
            DownloadStatus::Done { .. } => dict.actions.download_complete.to_string(),
            DownloadStatus::Failed { message } => message.clone(),
            DownloadStatus::Idle => String::new(),
        };

        source_cards.push(
            v_flex()
                .w(px(300.))
                .p_3p5()
                .gap_3()
                .rounded(theme_tokens::RADIUS)
                .bg(surface_bg)
                .border_1()
                .border_color(border_col)
                .shadow(vec![BoxShadow {
                    offset: point(px(0.), px(2.)),
                    blur_radius: px(8.),
                    spread_radius: px(0.),
                    inset: false,
                    color: hsla(222.0 / 360.0, 0.50, 0.02, 0.40),
                }])
                .hover(move |s| s.border_color(primary_color))
                .when(rtl, |this| this.items_end().text_right())
                .child(
                    directional_row(h_flex(), rtl)
                        .w_full()
                        .justify_between()
                        .items_center()
                        .gap_2()
                        .child(
                            directional_row(h_flex(), rtl)
                                .gap_2()
                                .items_center()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size_7()
                                        .rounded(px(6.))
                                        .bg(primary_color)
                                        .text_color(white())
                                        .child(
                                            Icon::new(if is_playable {
                                                IconName::Play
                                            } else if is_hls {
                                                IconName::Info
                                            } else {
                                                IconName::ArrowDown
                                            })
                                            .xsmall(),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_bold()
                                        .text_color(white())
                                        .child(quality.clone()),
                                ),
                        )
                        .child(Tag::primary().small().outline().child(format_badge)),
                )
                .when(!status_label.is_empty(), |col| {
                    col.child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(status_label),
                    )
                })
                .child(
                    directional_row(h_flex(), rtl)
                        .w_full()
                        .gap_2()
                        .flex_wrap()
                        .child(
                            Button::new(play_btn_id)
                                .primary()
                                .small()
                                .icon(IconName::Play)
                                .label(dict.actions.play.to_uppercase())
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.play_source(
                                        title_for_play.clone(),
                                        source_for_play.clone(),
                                        media_id,
                                        ep_key_owned.clone(),
                                        window,
                                        cx,
                                    );
                                })),
                        )
                        .child(
                            Button::new(dl_btn_id)
                                .outline()
                                .small()
                                .icon(IconName::ArrowDown)
                                .label(dict.actions.download.to_uppercase())
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.start_download(
                                        dl_key_owned.clone(),
                                        &source_for_dl,
                                        title_for_dl.clone(),
                                        window,
                                        cx,
                                    );
                                })),
                        )
                        .child(
                            Button::new(copy_btn_id)
                                .ghost()
                                .small()
                                .icon(IconName::Copy)
                                .label(dict.actions.copy_link)
                                .on_click(move |_, window, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(
                                        url_for_copy.clone(),
                                    ));
                                    window.push_notification(copy_notice.clone(), cx);
                                }),
                        )
                        .when(!is_playable, |row| {
                            let url = source.url.clone();
                            row.child(
                                Button::new(ext_btn_id)
                                    .ghost()
                                    .small()
                                    .icon(IconName::Globe)
                                    .label(dict.actions.open_externally)
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.open_source_externally(&url, window, cx);
                                    })),
                            )
                        }),
                ),
        );
    }

    h_flex()
        .w_full()
        .flex_wrap()
        .gap_3()
        .when(rtl, |this| this.flex_row_reverse())
        .children(source_cards)
        .into_any_element()
}

/// Compact Play + Download actions for an episode row (best source).
pub fn render_episode_actions(
    sources: &[SourceLink],
    context_title: &str,
    locale: Locale,
    id_prefix: &str,
    media_id: u64,
    episode_key: &str,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let best = crate::player::best_playable_source(sources).cloned();
    let any_source = sources.iter().find(|s| !s.is_trailer).cloned();
    let download_source = best.clone().or(any_source.clone());

    let play_id = SharedString::from(format!("ep-play-{id_prefix}"));
    let dl_id = SharedString::from(format!("ep-dl-{id_prefix}"));
    let title = context_title.to_string();
    let ep_key = episode_key.to_string();
    let dl_key = format!("{media_id}:{episode_key}");

    directional_row(h_flex(), rtl)
        .gap_2()
        .items_center()
        .child(
            Button::new(play_id)
                .primary()
                .small()
                .icon(IconName::Play)
                .label(dict.actions.play.to_uppercase())
                .on_click(cx.listener({
                    let title = title.clone();
                    let ep_key = ep_key.clone();
                    let sources: Vec<SourceLink> = sources.to_vec();
                    move |this, _, window, cx| {
                        this.play_best_source(
                            title.clone(),
                            &sources,
                            media_id,
                            Some(ep_key.clone()),
                            window,
                            cx,
                        );
                    }
                })),
        )
        .when_some(download_source, |row, source| {
            let title = title.clone();
            let dl_key = dl_key.clone();
            row.child(
                Button::new(dl_id)
                    .outline()
                    .small()
                    .icon(IconName::ArrowDown)
                    .label(dict.actions.download.to_uppercase())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.start_download(dl_key.clone(), &source, title.clone(), window, cx);
                    })),
            )
        })
        .into_any_element()
}
