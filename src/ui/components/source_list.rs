//! Infuse / Stremio / Linear inspired stream and download source cards.

use gpui_kit::{
    AnyElement, ClipboardItem, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, Styled as _, div, hsla, prelude::FluentBuilder as _, px,
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
};

pub fn render_source_links(
    sources: &[SourceLink],
    context_title: &str,
    locale: Locale,
    id_prefix: &str,
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

    let surface_bg = if is_dark {
        hsla(240.0 / 360.0, 0.20, 0.09, 0.85)
    } else {
        cx.theme().popover
    };
    let border_col = if is_dark {
        hsla(0.0, 0.0, 1.0, 0.09)
    } else {
        cx.theme().border
    };

    let mut source_cards = Vec::with_capacity(downloads.len());
    for (ix, source) in downloads.into_iter().enumerate() {
        let quality = source.quality.clone();
        let format_badge = source
            .format_type
            .clone()
            .unwrap_or_else(|| "Direct Stream / MKV".to_string());
        let is_playable = source.is_playable_video();
        let url_for_open = source.url.clone();
        let url_for_copy = source.url.clone();
        let dialog_title = format!("{context_title} — {quality}");
        let copy_notice = dict.actions.copied.to_string();

        let open_btn_id = SharedString::from(format!("src-open-{id_prefix}-{}-{ix}", source.id));
        let copy_btn_id = SharedString::from(format!("src-copy-{id_prefix}-{}-{ix}", source.id));

        source_cards.push(
            v_flex()
                .w(px(300.))
                .p_4()
                .gap_3()
                .rounded(px(14.))
                .bg(surface_bg)
                .border_1()
                .border_color(border_col)
                .hover(move |s| s.border_color(primary_color.opacity(0.65)))
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
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size_7()
                                        .rounded(px(8.))
                                        .bg(primary_color.opacity(0.16))
                                        .text_color(primary_color)
                                        .child(
                                            Icon::new(if is_playable {
                                                IconName::Play
                                            } else {
                                                IconName::ArrowDown
                                            })
                                            .xsmall(),
                                        ),
                                )
                                .child(div().text_sm().font_bold().child(quality.clone())),
                        )
                        .child(Tag::secondary().small().outline().child(format_badge)),
                )
                .child(
                    directional_row(h_flex(), rtl)
                        .w_full()
                        .gap_2()
                        .child(
                            Button::new(open_btn_id)
                                .primary()
                                .small()
                                .icon(IconName::Play)
                                .label(dict.actions.watch_online)
                                .on_click(move |_, window, cx| {
                                    let url = url_for_open.clone();
                                    let heading = dialog_title.clone();
                                    let _ = std::process::Command::new("open").arg(&url).spawn();
                                    window.open_dialog(cx, move |dialog, _, cx| {
                                        dialog.title(heading.clone()).child(
                                            v_flex().gap_3().child(
                                                div()
                                                    .text_sm()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(url.clone()),
                                            ),
                                        )
                                    });
                                }),
                        )
                        .child(
                            Button::new(copy_btn_id)
                                .outline()
                                .small()
                                .icon(IconName::Copy)
                                .label(dict.actions.copy_link)
                                .on_click(move |_, window, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(
                                        url_for_copy.clone(),
                                    ));
                                    window.push_notification(copy_notice.clone(), cx);
                                }),
                        ),
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
