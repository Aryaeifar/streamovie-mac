//! Search screen completely redesigned to match Streamovie Search UI/UX.
//!
//! Features:
//! - Streamovie high-impact search hero header with category filter pills
//! - Sleek dark glass input with Streamovie blue focus glow and clean button
//! - Instant results counter and responsive multi-column grid of Streamovie poster cards

use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, Styled as _, div,
    prelude::FluentBuilder as _, px,
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _,
    alert::Alert,
    button::{Button, ButtonVariants as _},
    empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle},
    h_flex,
    input::Input,
    spinner::Spinner,
    tab::{Tab, TabBar},
    tag::Tag,
    v_flex,
};

use crate::{
    app::StreamovieApp,
    i18n::{directional_row, format_count},
    state::MediaFilterTab,
    ui::components::render_poster_card,
};

pub fn render_search_view(
    app: &StreamovieApp,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let search = app.search_state().clone();

    let filtered: Vec<_> = search
        .results
        .iter()
        .filter(|media| search.filter_tab.matches(media.inferred_category()))
        .cloned()
        .collect();

    let mut cards = Vec::with_capacity(filtered.len());
    for media in &filtered {
        let kind = media.inferred_category();
        cards.push(render_poster_card(
            app,
            media,
            kind,
            app.is_bookmarked(media.id),
            locale,
            "streamovie-search-results",
            cx,
        ));
    }

    v_flex()
        .w_full()
        .gap_6()
        .pb_12()
        // Streamovie Search Panel
        .child(
            v_flex()
                .w_full()
                .p_6()
                .gap_4()
                .rounded(px(10.))
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().popover)
                .child(
                    directional_row(h_flex(), rtl)
                        .w_full()
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
                                        .w(px(3.5))
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
                                                        .font_extrabold()
                                                        .child("SEARCH"),
                                                )
                                                .when(!search.query.trim().is_empty(), |row| {
                                                    row.child(
                                                        Tag::primary()
                                                            .small()
                                                            .outline()
                                                            .child(format_count(
                                                                dict.browse.results,
                                                                filtered.len(),
                                                                locale,
                                                            )),
                                                    )
                                                }),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("Search by title, character, actor, or genre across the Streamovie catalog."),
                                        ),
                                ),
                        )
                        // Filter Pills: ALL | MOVIES | TV SHOWS
                        .child(
                            TabBar::new("streamovie-search-filter-tabs")
                                .pill()
                                .selected_index(search.filter_tab.index())
                                .child(Tab::new().label("ALL"))
                                .child(Tab::new().label("MOVIES"))
                                .child(Tab::new().label("TV SHOWS"))
                                .on_click(cx.listener(|this, ix: &usize, _, cx| {
                                    this.set_search_filter_tab(MediaFilterTab::from_index(*ix), cx);
                                })),
                        ),
                )
                // Search Input Box & Submit Button
                .child(
                    directional_row(h_flex(), rtl)
                        .w_full()
                        .gap_3()
                        .child(
                            div().flex_1().child(
                                Input::new(app.search_input())
                                    .large()
                                    .cleanable(true)
                                    .prefix(
                                        Icon::new(IconName::Search)
                                            .small()
                                            .text_color(cx.theme().primary),
                                    ),
                            ),
                        )
                        .child(
                            Button::new("streamovie-search-submit-btn")
                                .primary()
                                .large()
                                .loading(search.loading)
                                .icon(IconName::Search)
                                .label("SEARCH")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.trigger_search_now(cx);
                                })),
                        ),
                ),
        )
        // Error state
        .when_some(search.error.clone(), |col, err| {
            col.child(
                v_flex()
                    .w_full()
                    .gap_2()
                    .child(Alert::error("search-error-banner", err).title(dict.search.failed))
                    .child(
                        directional_row(h_flex(), rtl).child(
                            Button::new("streamovie-search-retry-btn")
                                .small()
                                .outline()
                                .icon(IconName::RefreshCw)
                                .label(dict.actions.retry)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.trigger_search_now(cx);
                                })),
                        ),
                    ),
            )
        })
        // Results / Loading / Empty
        .child(if search.loading {
            h_flex()
                .w_full()
                .py_12()
                .items_center()
                .justify_center()
                .gap_3()
                .child(Spinner::new().large())
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Searching the Streamovie catalog…"),
                )
                .into_any_element()
        } else if search.query.trim().is_empty() {
            Empty::new()
                .header(
                    EmptyHeader::new()
                        .media(
                            EmptyMedia::new()
                                .with_variant(EmptyMediaVariant::Icon)
                                .child(Icon::new(IconName::Search)),
                        )
                        .title(EmptyTitle::new().child("Find Your Next Obsession"))
                        .description(EmptyDescription::new().child(
                            "Type in the search box above to instantly discover blockbuster films and original series.",
                        )),
                )
                .into_any_element()
        } else if cards.is_empty() {
            Empty::new()
                .header(
                    EmptyHeader::new()
                        .media(
                            EmptyMedia::new()
                                .with_variant(EmptyMediaVariant::Icon)
                                .child(Icon::new(IconName::Search)),
                        )
                        .title(EmptyTitle::new().child("No Matching Titles Found"))
                        .description(EmptyDescription::new().child(
                            "Check your spelling or try searching for a different movie, series, or actor.",
                        )),
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
