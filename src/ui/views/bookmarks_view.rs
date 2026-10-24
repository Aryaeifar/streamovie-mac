//! Local watchlist.

use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Selectable as _, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    empty::{
        Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant,
        EmptyTitle,
    },
    h_flex, v_flex,
};
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, Styled as _, div,
    prelude::FluentBuilder as _,
};

use crate::{
    app::StreamovieApp,
    i18n::{directional_row, format_count},
    state::{MediaFilterTab, NavSection},
    ui::components::render_poster_card,
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
        .px_8()
        .pt_6()
        .pb_12()
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
                        .child(div().text_2xl().font_semibold().child(dict.bookmarks.title))
                        .child(
                            div()
                                .text_sm()
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
            Empty::new()
                .header(
                    EmptyHeader::new()
                        .media(
                            EmptyMedia::new()
                                .with_variant(EmptyMediaVariant::Icon)
                                .child(Icon::new(IconName::BookOpen)),
                        )
                        .title(EmptyTitle::new().child(dict.bookmarks.empty_title))
                        .description(EmptyDescription::new().child(dict.bookmarks.empty_body)),
                )
                .content(
                    EmptyContent::new().child(
                        Button::new("watchlist-browse")
                            .primary()
                            .label(dict.actions.browse)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.select_section(NavSection::Home, cx);
                            })),
                    ),
                )
                .into_any_element()
        } else {
            h_flex()
                .flex_wrap()
                .gap_x_4()
                .gap_y_6()
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
        .selected(selected == tab)
        .when(selected == tab, |btn| btn.primary())
        .when(selected != tab, |btn| btn.outline())
        .label(label)
        .on_click(cx.listener(move |this, _, _, cx| {
            this.set_bookmarks_filter_tab(tab, cx);
        }))
}
