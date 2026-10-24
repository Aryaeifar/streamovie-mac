//! Cinematic in-app player chrome. Video itself is a child WebKit view.

use crate::{app::StreamovieApp, i18n::directional_row, playback::PlaybackSession};
use gpui_kit::component::{
    ActiveTheme as _, IconName, StyledExt as _, button::Button, h_flex, v_flex,
};
use gpui_kit::{AnyElement, Context, IntoElement, ParentElement as _, Styled as _, div, px, white};

pub fn render_player_view(
    app: &StreamovieApp,
    session: &PlaybackSession,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();

    v_flex()
        .size_full()
        .bg(cx.theme().background)
        .child(
            directional_row(h_flex(), rtl)
                .w_full()
                .px_5()
                .h(px(56.))
                .flex_shrink_0()
                .items_center()
                .justify_between()
                .gap_4()
                .child(
                    Button::new("player-back")
                        .outline()
                        .icon(if rtl {
                            IconName::ChevronRight
                        } else {
                            IconName::ChevronLeft
                        })
                        .label(dict.player.back)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.close_playback(cx);
                        })),
                )
                .child(
                    v_flex()
                        .min_w_0()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(dict.player.now_playing),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_semibold()
                                .text_color(white())
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(session.heading.clone()),
                        ),
                )
                .child(div().w(px(88.))),
        )
        .child(match (&session.stream_url, session.webview.as_ref()) {
            (None, _) => unavailable(dict.player.unavailable_title, &session.message, cx),
            (Some(_), None) if session.mount_error.is_some() => unavailable(
                dict.player.unavailable_title,
                session
                    .mount_error
                    .as_deref()
                    .unwrap_or(dict.player.unavailable_body),
                cx,
            ),
            (Some(_), None) => unavailable(dict.player.preparing, dict.player.preparing, cx),
            (Some(_), Some(webview)) => div()
                .flex_1()
                .min_h_0()
                .w_full()
                .bg(cx.theme().background)
                .child(webview.clone())
                .into_any_element(),
        })
        .into_any_element()
}

fn unavailable(title: &str, body: &str, cx: &mut Context<StreamovieApp>) -> AnyElement {
    v_flex()
        .flex_1()
        .items_center()
        .justify_center()
        .gap_2()
        .px_8()
        .child(
            div()
                .text_xl()
                .font_semibold()
                .text_color(cx.theme().foreground)
                .child(title.to_string()),
        )
        .child(
            div()
                .max_w(px(460.))
                .text_sm()
                .text_center()
                .text_color(cx.theme().muted_foreground)
                .child(body.to_string()),
        )
        .into_any_element()
}
