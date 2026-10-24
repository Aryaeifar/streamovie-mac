//! Shared loading, empty, and network-error surfaces.

use gpui_kit::component::{
    Icon, IconName, Sizable as _, StyledExt as _, h_flex, skeleton::Skeleton, v_flex,
};
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, SharedString, Styled as _, div,
    prelude::FluentBuilder as _, px,
};

use crate::{
    app::StreamovieApp,
    ui::{apple::ApplePaint, components::PosterCardStyle},
};

pub fn skeleton_card(style: PosterCardStyle) -> AnyElement {
    let (width, height) = style.frame();
    v_flex()
        .flex_none()
        .w(width)
        .gap(px(8.))
        .child(Skeleton::new().w(width).h(height).rounded(px(14.)))
        .child(Skeleton::new().w(width * 0.72).h(px(12.)).rounded(px(6.)))
        .child(
            Skeleton::new()
                .secondary()
                .w(width * 0.4)
                .h(px(10.))
                .rounded(px(6.)),
        )
        .into_any_element()
}

pub fn skeleton_row(style: PosterCardStyle, count: usize, rtl: bool) -> AnyElement {
    div()
        .w_full()
        .min_w_0()
        .overflow_hidden()
        .child(
            h_flex()
                .gap(px(20.))
                .py(px(8.))
                .when(rtl, |row| row.flex_row_reverse())
                .children((0..count).map(|_| skeleton_card(style))),
        )
        .into_any_element()
}

pub fn skeleton_grid(count: usize, rtl: bool) -> AnyElement {
    h_flex()
        .w_full()
        .flex_wrap()
        .gap(px(22.))
        .py(px(12.))
        .when(rtl, |row| row.flex_row_reverse())
        .children((0..count).map(|_| skeleton_card(PosterCardStyle::Poster)))
        .into_any_element()
}

pub fn home_skeleton(rtl: bool) -> AnyElement {
    v_flex()
        .w_full()
        .min_w_0()
        .overflow_hidden()
        .gap(px(36.))
        .pt(px(8.))
        .pb(px(56.))
        .child(rail_skeleton(PosterCardStyle::Widescreen, 4, rtl))
        .child(rail_skeleton(PosterCardStyle::Poster, 6, rtl))
        .child(rail_skeleton(PosterCardStyle::Poster, 6, rtl))
        .child(rail_skeleton(PosterCardStyle::Poster, 6, rtl))
        .into_any_element()
}

pub fn episode_skeleton_row(rtl: bool) -> AnyElement {
    div()
        .w_full()
        .min_w_0()
        .overflow_hidden()
        .child(
            h_flex()
                .gap(px(18.))
                .py(px(8.))
                .when(rtl, |row| row.flex_row_reverse())
                .children((0..4).map(|_| {
                    v_flex()
                        .flex_none()
                        .w(px(292.))
                        .gap(px(10.))
                        .child(Skeleton::new().w(px(292.)).h(px(164.)).rounded(px(16.)))
                        .child(Skeleton::new().w(px(180.)).h(px(14.)).rounded(px(6.)))
                        .child(
                            Skeleton::new()
                                .secondary()
                                .w(px(96.))
                                .h(px(12.))
                                .rounded(px(6.)),
                        )
                        .into_any_element()
                })),
        )
        .into_any_element()
}

pub fn cast_skeleton_row(rtl: bool) -> AnyElement {
    div()
        .w_full()
        .min_w_0()
        .overflow_hidden()
        .child(
            h_flex()
                .gap(px(18.))
                .py(px(4.))
                .when(rtl, |row| row.flex_row_reverse())
                .children((0..8).map(|_| {
                    v_flex()
                        .flex_none()
                        .w(px(88.))
                        .items_center()
                        .gap(px(8.))
                        .child(Skeleton::new().size(px(64.)).rounded_full())
                        .child(Skeleton::new().w(px(72.)).h(px(12.)).rounded(px(6.)))
                        .into_any_element()
                })),
        )
        .into_any_element()
}

pub fn empty_state(
    icon: IconName,
    title: impl Into<SharedString>,
    body: impl Into<SharedString>,
    action: Option<AnyElement>,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let paint = ApplePaint::from_cx(cx);
    state_card(
        icon,
        paint.secondary_text,
        SharedString::default(),
        title.into(),
        body.into(),
        None,
        action,
        &paint,
    )
}

pub fn network_error_state(
    eyebrow: impl Into<SharedString>,
    title: impl Into<SharedString>,
    body: impl Into<SharedString>,
    detail: Option<String>,
    action: Option<AnyElement>,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let paint = ApplePaint::from_cx(cx);
    state_card(
        IconName::TriangleAlert,
        paint.danger,
        eyebrow.into(),
        title.into(),
        body.into(),
        detail.map(|text| brief(&text)),
        action,
        &paint,
    )
}

fn rail_skeleton(style: PosterCardStyle, count: usize, rtl: bool) -> AnyElement {
    v_flex()
        .w_full()
        .min_w_0()
        .px(px(32.))
        .gap(px(12.))
        .child(Skeleton::new().w(px(180.)).h(px(22.)).rounded(px(8.)))
        .child(skeleton_row(style, count, rtl))
        .into_any_element()
}

fn state_card(
    icon: IconName,
    icon_color: gpui_kit::Hsla,
    eyebrow: SharedString,
    title: SharedString,
    body: SharedString,
    detail: Option<String>,
    action: Option<AnyElement>,
    paint: &ApplePaint,
) -> AnyElement {
    v_flex()
        .w_full()
        .items_center()
        .py(px(12.))
        .child(
            v_flex()
                .w_full()
                .max_w(px(440.))
                .items_center()
                .gap(px(10.))
                .px(px(24.))
                .py(px(28.))
                .bg(paint.surface)
                .rounded(px(16.))
                .border_1()
                .border_color(paint.hairline)
                .shadow(paint.poster_shadow(false))
                .child(
                    h_flex()
                        .size(px(56.))
                        .rounded_full()
                        .items_center()
                        .justify_center()
                        .bg(paint.chip)
                        .child(Icon::new(icon).large().text_color(icon_color)),
                )
                .when(!eyebrow.is_empty(), |card| {
                    card.child(
                        div()
                            .w_full()
                            .text_center()
                            .text_size(px(12.))
                            .font_semibold()
                            .text_color(paint.danger)
                            .child(eyebrow),
                    )
                })
                .child(
                    div()
                        .w_full()
                        .text_center()
                        .text_size(px(18.))
                        .font_semibold()
                        .text_color(paint.ink)
                        .child(title),
                )
                .when(!body.is_empty(), |card| {
                    card.child(
                        div()
                            .w_full()
                            .text_center()
                            .text_size(px(14.))
                            .line_height(gpui_kit::relative(1.4))
                            .text_color(paint.secondary_text)
                            .child(body),
                    )
                })
                .when_some(detail, |card, detail| {
                    card.child(
                        div()
                            .w_full()
                            .text_center()
                            .text_size(px(12.))
                            .text_color(paint.secondary_text)
                            .child(detail),
                    )
                })
                .when_some(action, |card, action| {
                    card.child(div().pt(px(6.)).child(action))
                }),
        )
        .into_any_element()
}

fn brief(detail: &str) -> String {
    let mut chars = detail.chars();
    let short: String = chars.by_ref().take(160).collect();
    if chars.next().is_some() {
        format!("{short}…")
    } else {
        short
    }
}
