//! Search across the live catalog.

use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Selectable as _, Sizable as _, StyledExt as _,
    alert::Alert,
    button::{Button, ButtonVariants as _},
    empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle},
    h_flex,
    input::Input,
    spinner::Spinner,
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
    ui::components::render_poster_card,
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
        .px_8()
        .pt_8()
        .pb_12()
        .gap_6()
        .child(
            v_flex()
                .w_full()
                .max_w(px(760.))
                .gap_4()
                .when(rtl, |col| col.items_end())
                .child(div().text_2xl().font_semibold().child(dict.nav.search))
                .child(
                    div().w_full().child(
                        Input::new(app.search_input())
                            .large()
                            .cleanable(true)
                            .prefix(
                                Icon::new(IconName::Search).text_color(cx.theme().muted_foreground),
                            ),
                    ),
                )
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
            directional_row(h_flex(), rtl)
                .gap_2()
                .items_center()
                .child(Spinner::new().small())
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(dict.search.searching),
                )
                .into_any_element()
        } else if let Some(err) = search.error.clone() {
            v_flex()
                .gap_3()
                .child(Alert::error("search-error", err).title(dict.search.failed))
                .child(
                    Button::new("search-retry")
                        .outline()
                        .label(dict.actions.retry)
                        .on_click(cx.listener(|this, _, _, cx| this.trigger_search_now(cx))),
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
                        .title(EmptyTitle::new().child(dict.search.empty))
                        .description(EmptyDescription::new().child(dict.search.hint)),
                )
                .into_any_element()
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
                        .gap_x_4()
                        .gap_y_6()
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
        .selected(selected == tab)
        .when(selected == tab, |btn| btn.primary())
        .when(selected != tab, |btn| btn.outline())
        .label(label)
        .on_click(cx.listener(move |this, _, _, cx| {
            this.set_search_filter_tab(tab, cx);
        }))
}
