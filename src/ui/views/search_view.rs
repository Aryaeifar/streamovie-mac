//! Search across the live catalog.

use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Selectable as _, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::Input,
    v_flex,
};
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, Styled as _, div,
    prelude::FluentBuilder as _, px,
};

use crate::{
    app::StreamovieApp,
    i18n::{directional_row, format_count},
    state::MediaFilterTab,
    ui::components::{empty_state, network_error_state, render_poster_card, skeleton_grid},
};

pub fn render_search_view(app: &StreamovieApp, cx: &mut Context<StreamovieApp>) -> AnyElement {
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

    let cards: Vec<_> = filtered
        .iter()
        .map(|media| {
            let kind = media.inferred_category();
            render_poster_card(
                app,
                media,
                kind,
                app.is_bookmarked(media.id),
                locale,
                "search",
                cx,
            )
        })
        .collect();

    v_flex()
        .w_full()
        .px(px(32.))
        .pt(px(24.))
        .pb(px(48.))
        .gap_6()
        .child(
            v_flex()
                .w_full()
                .max_w(px(760.))
                .gap_4()
                .when(rtl, |col| col.items_end())
                .child(
                    div()
                        .text_size(px(28.))
                        .font_semibold()
                        .child(dict.nav.search),
                )
                .child(div().w_full().child(
                    Input::new(app.search_input()).cleanable(true).prefix(
                        Icon::new(IconName::Search).text_color(cx.theme().muted_foreground),
                    ),
                ))
                .child(
                    directional_row(h_flex(), rtl)
                        .gap_2()
                        .child(filter_button(
                            app,
                            "search-all",
                            dict.browse.all_types,
                            MediaFilterTab::All,
                            search.filter_tab,
                            cx,
                        ))
                        .child(filter_button(
                            app,
                            "search-movies",
                            dict.browse.movies,
                            MediaFilterTab::Movies,
                            search.filter_tab,
                            cx,
                        ))
                        .child(filter_button(
                            app,
                            "search-series",
                            dict.browse.series,
                            MediaFilterTab::Series,
                            search.filter_tab,
                            cx,
                        )),
                ),
        )
        .child(if search.query.trim().is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(dict.search.hint)
                .into_any_element()
        } else if search.loading {
            skeleton_grid(8, rtl)
        } else if let Some(err) = search.error.clone() {
            network_error_state(
                dict.common.network_title,
                dict.search.failed,
                dict.common.network_body,
                Some(err),
                Some(
                    Button::new("search-retry")
                        .primary()
                        .rounded(px(13.))
                        .icon(IconName::RefreshCw)
                        .label(dict.actions.retry)
                        .on_click(cx.listener(|this, _, _, cx| this.trigger_search_now(cx)))
                        .into_any_element(),
                ),
                cx,
            )
        } else if cards.is_empty() {
            empty_state(
                IconName::Search,
                dict.search.empty,
                dict.search.hint,
                None,
                cx,
            )
        } else {
            v_flex()
                .gap_4()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(format_count(dict.browse.results, cards.len(), locale)),
                )
                .child(
                    h_flex()
                        .flex_wrap()
                        .gap(px(22.))
                        .py(px(12.))
                        .when(rtl, |row| row.flex_row_reverse())
                        .children(cards),
                )
                .into_any_element()
        })
        .into_any_element()
}

fn filter_button(
    _app: &StreamovieApp,
    id: &'static str,
    label: &'static str,
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
            this.set_search_filter_tab(tab, cx);
        }))
}
