//! Full-bleed home spotlight.

use gpui_kit::component::{
    IconName, Sizable as _, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, hsla, linear_color_stop, linear_gradient,
    prelude::FluentBuilder as _, px, relative, white,
};

use crate::{
    app::StreamovieApp,
    i18n::{Locale, directional_row, is_persian_text},
    models::{HeroSpotlightItem, MediaCategory},
    ui::image_cache::render_remote_image,
};

pub fn render_hero_banner(
    app: &StreamovieApp,
    spotlight: &[HeroSpotlightItem],
    active_index: usize,
    locale: Locale,
    bookmarked: bool,
    _liked: bool,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let Some(item) = spotlight.get(active_index) else {
        return div().into_any_element();
    };

    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let media = item.media.clone();
    let category = item.category;
    let title = if rtl {
        item.title_fa
            .clone()
            .unwrap_or_else(|| media.display_title())
    } else {
        media.display_title()
    };
    let title_rtl = is_persian_text(&title) || rtl;
    let overview = if rtl {
        item.description_fa.clone()
    } else {
        media.parsed_description().overview
    };
    let overview_rtl = overview.as_deref().is_some_and(is_persian_text) || rtl;
    let year = media.release_year();
    let rating = media.rating_value();
    let kind_label = if category == MediaCategory::Series {
        dict.home.series
    } else {
        dict.home.movie
    };

    let backdrop = render_remote_image(
        app,
        item.artwork.as_deref().or(media.hero_image().as_deref()),
        media.poster_image().as_deref(),
        px(0.),
        IconName::Frame,
        None,
        cx,
    );

    let media_for_watch = media.clone();
    let media_for_details = media.clone();
    let media_for_bookmark = media.clone();
    let stage = hsla(0.0, 0.0, 0.02, 1.0);

    div()
        .relative()
        .w_full()
        .h(px(560.))
        .overflow_hidden()
        .bg(stage)
        .child(div().absolute().inset_0().child(backdrop))
        .child(div().absolute().inset_0().bg(linear_gradient(
            0.0,
            linear_color_stop(stage, 0.0),
            linear_color_stop(stage.opacity(0.0), 0.72),
        )))
        .child(div().absolute().inset_0().bg(linear_gradient(
            if rtl { 270.0 } else { 90.0 },
            linear_color_stop(stage.opacity(0.92), 0.0),
            linear_color_stop(stage.opacity(0.0), 0.72),
        )))
        .child(
            v_flex()
                .absolute()
                .left(px(56.))
                .right(px(56.))
                .bottom(px(72.))
                .gap_3()
                .max_w(px(720.))
                .when(rtl, |col| col.items_end().text_right())
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(white().opacity(0.72))
                        .child(kind_label.to_uppercase()),
                )
                .child(
                    div()
                        .text_size(px(48.))
                        .font_bold()
                        .line_height(relative(1.05))
                        .text_color(white())
                        .when(title_rtl, |el| el.text_right())
                        .child(title),
                )
                .child(
                    directional_row(h_flex(), rtl)
                        .gap_2()
                        .text_sm()
                        .text_color(white().opacity(0.78))
                        .when_some(year, |row, year| row.child(year))
                        .when(rating > 0.0, |row| row.child(format!("{rating:.1}"))),
                )
                .when_some(overview, |col, text| {
                    let shown: String = text.chars().take(180).collect();
                    col.child(
                        div()
                            .max_w(px(560.))
                            .text_sm()
                            .line_height(relative(1.5))
                            .text_color(hsla(0.0, 0.0, 0.9, 0.92))
                            .when(overview_rtl, |el| el.text_right())
                            .child(shown),
                    )
                })
                .child(
                    directional_row(h_flex(), rtl)
                        .pt_2()
                        .gap_3()
                        .items_center()
                        .child(
                            Button::new("hero-watch")
                                .primary()
                                .large()
                                .icon(IconName::Play)
                                .label(dict.actions.watch_now)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.watch_title(media_for_watch.clone(), category, cx);
                                })),
                        )
                        .child(
                            Button::new("hero-list")
                                .large()
                                .outline()
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
                                    let notice = if added {
                                        this.locale().dict().bookmarks.add
                                    } else {
                                        this.locale().dict().bookmarks.remove
                                    };
                                    window.push_notification(notice, cx);
                                })),
                        )
                        .child(
                            Button::new("hero-details")
                                .large()
                                .outline()
                                .label(dict.actions.details)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.open_details(media_for_details.clone(), category, cx);
                                })),
                        ),
                ),
        )
        .child(
            directional_row(h_flex(), rtl)
                .absolute()
                .left(px(56.))
                .right(px(56.))
                .bottom(px(28.))
                .gap_2()
                .children(spotlight.iter().enumerate().map(|(index, _)| {
                    let selected = index == active_index;
                    let id = SharedString::from(format!("hero-dot-{index}"));
                    div()
                        .id(id)
                        .w(if selected { px(22.) } else { px(8.) })
                        .h(px(4.))
                        .rounded_full()
                        .cursor_pointer()
                        .bg(if selected {
                            white()
                        } else {
                            white().opacity(0.35)
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.set_spotlight_index(index, cx);
                        }))
                        .into_any_element()
                })),
        )
        .into_any_element()
}
