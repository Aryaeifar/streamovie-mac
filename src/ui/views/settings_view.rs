//! Appearance, language, and local storage. Catalog credentials stay in the environment.

use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    setting::{SettingField, SettingGroup, SettingItem, SettingPage, Settings},
    v_flex,
};
use gpui_kit::{
    App, Context, IntoElement, ParentElement as _, SharedString, Styled as _, div,
    prelude::FluentBuilder as _,
};

use crate::{
    app::StreamovieApp,
    i18n::{Locale, directional_row},
    storage::{AccentPalette, ThemeModePreference},
};

pub fn render_settings_view(
    app: &StreamovieApp,
    cx: &mut Context<StreamovieApp>,
) -> impl IntoElement + use<> {
    let locale = app.locale();
    let dict = locale.dict();
    let rtl = locale.is_rtl();
    let view = cx.entity();

    let mode_options: Vec<(SharedString, SharedString)> = ThemeModePreference::ALL
        .iter()
        .map(|mode| (mode.code().into(), mode.label(dict).into()))
        .collect();
    let mode_field = {
        let read = view.clone();
        let write = view.clone();
        SettingField::dropdown(
            mode_options,
            move |cx: &App| read.read(cx).settings().theme_mode.code().into(),
            move |val: SharedString, cx: &mut App| {
                write.update(cx, |this, cx| {
                    this.set_theme_mode(ThemeModePreference::from_code(val.as_ref()), None, cx);
                });
            },
        )
    };

    let accent_options: Vec<(SharedString, SharedString)> = AccentPalette::ALL
        .iter()
        .map(|palette| (palette.code().into(), palette.label(dict).into()))
        .collect();
    let accent_field = {
        let read = view.clone();
        let write = view.clone();
        SettingField::dropdown(
            accent_options,
            move |cx: &App| read.read(cx).settings().accent_palette.code().into(),
            move |val: SharedString, cx: &mut App| {
                write.update(cx, |this, cx| {
                    this.set_accent_palette(AccentPalette::from_code(val.as_ref()), None, cx);
                });
            },
        )
    };

    let lang_options: Vec<(SharedString, SharedString)> = vec![
        (Locale::En.code().into(), Locale::En.label().into()),
        (Locale::Fa.code().into(), Locale::Fa.label().into()),
    ];
    let lang_field = {
        let read = view.clone();
        let write = view.clone();
        SettingField::dropdown(
            lang_options,
            move |cx: &App| read.read(cx).locale().code().into(),
            move |val: SharedString, cx: &mut App| {
                write.update(cx, |this, cx| {
                    let next = if val.as_ref() == "fa" {
                        Locale::Fa
                    } else {
                        Locale::En
                    };
                    this.set_locale(next, None, cx);
                });
            },
        )
    };

    let storage_dir = app.storage_dir_display();
    let pages = vec![
        SettingPage::new(dict.settings.title)
            .icon(Icon::new(IconName::Settings2))
            .default_open(true)
            .groups(vec![
                SettingGroup::new()
                    .title(dict.settings.appearance_group)
                    .items(vec![
                        SettingItem::new(dict.settings.mode_label, mode_field)
                            .description(dict.settings.mode_description),
                        SettingItem::new(dict.settings.accent_theme, accent_field)
                            .description(dict.settings.accent_description),
                    ]),
                SettingGroup::new()
                    .title(dict.settings.language_group)
                    .items(vec![
                        SettingItem::new(dict.common.language, lang_field)
                            .description(dict.settings.language_description),
                    ]),
                SettingGroup::new()
                    .title(dict.settings.storage_group)
                    .items(vec![SettingItem::render(move |_, _, _| {
                        directional_row(h_flex(), rtl)
                            .w_full()
                            .justify_between()
                            .gap_3()
                            .child(div().text_sm().child(dict.settings.storage_path_label))
                            .child(div().text_xs().child(storage_dir.clone()))
                            .into_any_element()
                    })]),
            ]),
    ];

    v_flex()
        .size_full()
        .gap_4()
        .child(
            directional_row(h_flex(), rtl)
                .w_full()
                .justify_between()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .when(rtl, |col| col.items_end().text_right())
                        .child(div().text_2xl().font_semibold().child(dict.settings.title))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(dict.settings.description),
                        ),
                )
                .child(
                    directional_row(h_flex(), rtl)
                        .gap_2()
                        .child(
                            Button::new("settings-reset")
                                .outline()
                                .label(dict.settings.reset)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.reset_settings(Some(window), cx);
                                })),
                        )
                        .child(
                            Button::new("settings-save")
                                .primary()
                                .label(dict.settings.save)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.save_settings_now(cx);
                                    window.push_notification(
                                        this.locale().dict().settings.saved_notice,
                                        cx,
                                    );
                                })),
                        ),
                ),
        )
        .child(
            div()
                .min_h_0()
                .flex_1()
                .child(Settings::new("streamovie-settings").pages(pages)),
        )
}
