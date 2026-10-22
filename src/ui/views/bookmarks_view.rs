//! Local Bookmarks / Watchlist view redesigned with an Infuse library header and
//! cinema poster grid.

use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, Styled as _, div,
    prelude::FluentBuilder as _, px,
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    empty::{
        Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant,
        EmptyTitle,
    },
    h_flex,
    tab::{Tab, TabBar},
    tag::Tag,
    v_flex,
};

use crate::{
    app::StreamovieApp,
    i18n::{directional_row, format_count},
    state::{MediaFilterTab, NavSection},
    ui::components::render_poster_card,
};

pub fn render_bookmarks_view(
    app: &StreamovieApp,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let filter_tab = app.bookmarks_filter_tab();

    let all_bookmarks = app.ordered_bookmarks();
    let filtered: Vec<_> = all_bookmarks
        .into_iter()
        .filter(|entry| filter_tab.matches(entry.kind))
        .collect();

    let mut cards = Vec::with_capacity(filtered.len());
    for entry in &filtered {
        cards.push(render_poster_card(
            app,
            &entry.media,
            entry.kind,
            true,
            locale,
            "bookmarks-grid",
            cx,
        ));
    }

    v_flex()
        .w_full()
        .gap_6()
        .pb_10()
        // Elevated Watchlist Header Bar
        .child(
            directional_row(h_flex(), rtl)
                .w_full()
                .p_5()
                .rounded(cx.theme().radius_lg)
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().popover)
                .justify_between()
                .items_center()
                .flex_wrap()
                .gap_4()
                .child(
                    directional_row(h_flex(), rtl)
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .w(px(4.))
                                .h(px(26.))
                                .rounded_full()
                                .bg(cx.theme().primary),
                        )
                        .child(
                            v_flex()
                                .gap_0p5()
                                .when(rtl, |this| this.items_end().text_right())
                                .child(
                                    directional_row(h_flex(), rtl)
                                        .items_center()
                                        .gap_3()
                                        .child(
                                            div()
                                                .text_2xl()
                                                .font_bold()
                                                .child(dict.bookmarks.title),
                                        )
                                        .child(
                                            Tag::primary().small().outline().child(format_count(
                                                dict.bookmarks.saved_count,
                                                filtered.len(),
                                                locale,
                                            )),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(dict.bookmarks.subtitle),
                                ),
                        ),
                )
                .child(
                    TabBar::new("bookmarks-filter-tabs")
                        .segmented()
                        .selected_index(filter_tab.index())
                        .child(Tab::new().label(dict.browse.all_types))
                        .child(Tab::new().label(dict.browse.movies))
                        .child(Tab::new().label(dict.browse.series))
                        .on_click(cx.listener(|this, ix: &usize, _, cx| {
                            this.set_bookmarks_filter_tab(MediaFilterTab::from_index(*ix), cx);
                        })),
                ),
        )
        // Grid or Empty State
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
                        Button::new("bookmarks-browse-cta")
                            .primary()
                            .icon(IconName::LayoutDashboard)
                            .label(dict.actions.browse)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.select_section(NavSection::Home, cx);
                            })),
                    ),
                )
                .into_any_element()
        } else {
            h_flex()
                .w_full()
                .flex_wrap()
                .gap_4()
                .pt_2()
                .pb_2()
                .when(rtl, |this| this.flex_row_reverse())
                .children(cards)
                .into_any_element()
        })
        .into_any_element()
}
