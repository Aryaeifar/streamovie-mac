//! Local watchlist.

use gpui_kit::component::{
    ActiveTheme as _, IconName, Selectable as _, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, Styled as _, div,
    prelude::FluentBuilder as _, px,
};

use crate::{
    app::StreamovieApp,
    i18n::{directional_row, format_count},
    state::{MediaFilterTab, NavSection},
    ui::components::{empty_state, render_poster_card},
};

pub fn render_bookmarks_view(app: &StreamovieApp, cx: &mut Context<StreamovieApp>) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let filter_tab = app.bookmarks_filter_tab();
    let filtered: Vec<_> = app
        .ordered_bookmarks()
        .into_iter()
        .filter(|entry| filter_tab.matches(entry.kind))
        .collect();

    let cards: Vec<_> = filtered
        .iter()
        .map(|entry| {
            render_poster_card(app, &entry.media, entry.kind, true, locale, "watchlist", cx)
        })
        .collect();

    v_flex()
        .w_full()
        .px(px(32.))
        .pt(px(24.))
        .pb(px(48.))
        .gap_5()
        .child(
            directional_row(h_flex(), rtl)
                .w_full()
                .justify_between()
                .items_end()
                .child(
                    v_flex()
                        .gap_1()
                        .when(rtl, |col| col.items_end().text_right())
                        .child(
                            div()
                                .text_size(px(28.))
                                .font_semibold()
                                .child(dict.bookmarks.title),
                        )
                        .child(
                            div()
                                .text_size(px(13.))
                                .text_color(cx.theme().muted_foreground)
                                .child(dict.bookmarks.subtitle),
                        ),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(format_count(
                            dict.bookmarks.saved_count,
                            cards.len(),
                            locale,
                        )),
                ),
        )
        .child(
            directional_row(h_flex(), rtl)
                .gap_2()
                .child(tab_button(
                    dict.browse.all_types,
                    "bm-all",
                    MediaFilterTab::All,
                    filter_tab,
                    cx,
                ))
                .child(tab_button(
                    dict.browse.movies,
                    "bm-movies",
                    MediaFilterTab::Movies,
                    filter_tab,
                    cx,
                ))
                .child(tab_button(
                    dict.browse.series,
                    "bm-series",
                    MediaFilterTab::Series,
                    filter_tab,
                    cx,
                )),
        )
        .child(if cards.is_empty() {
            empty_state(
                IconName::BookOpen,
                dict.bookmarks.empty_title,
                dict.bookmarks.empty_body,
                Some(
                    Button::new("watchlist-browse")
                        .primary()
                        .rounded(px(13.))
                        .label(dict.actions.browse)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.select_section(NavSection::Home, cx);
                        }))
                        .into_any_element(),
                ),
                cx,
            )
        } else {
            h_flex()
                .flex_wrap()
                .gap(px(22.))
                .py(px(12.))
                .when(rtl, |row| row.flex_row_reverse())
                .children(cards)
                .into_any_element()
        })
        .into_any_element()
}

fn tab_button(
    label: &'static str,
    id: &'static str,
    tab: MediaFilterTab,
    selected: MediaFilterTab,
    cx: &mut Context<StreamovieApp>,
) -> impl IntoElement {
    Button::new(id)
        .small()
        .rounded(px(13.))
        .selected(selected == tab)
        .when(selected == tab, |btn| btn.primary())
        .when(selected != tab, |btn| btn.secondary())
        .label(label)
        .on_click(cx.listener(move |this, _, _, cx| {
            this.set_bookmarks_filter_tab(tab, cx);
        }))
}
