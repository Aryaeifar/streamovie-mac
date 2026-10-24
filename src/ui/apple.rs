//! Apple TV light-theme surfaces, type, and pressable controls.
//!
//! Dark mode keeps the same layout with inverted ink so the appearance toggle
//! still reads clearly. Light mode is the Apple TV page: white cards, ink
//! buttons, and a soft gray canvas.

use gpui_kit::component::{ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _, h_flex};
use gpui_kit::{
    AnimationExt as _, AnyElement, App, BoxShadow, ClickEvent, Context, Hsla,
    InteractiveElement as _, IntoElement, MouseButton, MouseDownEvent, MouseUpEvent,
    ParentElement as _, SharedString, SpringAnimation, SpringConfig,
    StatefulInteractiveElement as _, Styled as _, Window, div, hsla, point,
    prelude::FluentBuilder as _, px, white,
};

use crate::app::StreamovieApp;

#[derive(Clone, Copy)]
pub struct ApplePaint {
    pub canvas: Hsla,
    pub surface: Hsla,
    pub ink: Hsla,
    pub body: Hsla,
    pub secondary_text: Hsla,
    pub hairline: Hsla,
    pub chip: Hsla,
    pub track: Hsla,
    pub primary_fill: Hsla,
    pub primary_hover: Hsla,
    pub primary_pressed: Hsla,
    pub on_primary: Hsla,
    pub secondary_fill: Hsla,
    pub secondary_hover: Hsla,
    pub secondary_pressed: Hsla,
    pub on_secondary: Hsla,
    pub star: Hsla,
    pub success: Hsla,
    pub danger: Hsla,
    pub dark: bool,
}

impl ApplePaint {
    pub fn resolve(dark: bool) -> Self {
        if dark {
            Self {
                canvas: hsla(240.0 / 360.0, 0.02, 0.07, 1.0),
                surface: hsla(240.0 / 360.0, 0.02, 0.12, 1.0),
                ink: hsla(0.0, 0.0, 1.0, 0.94),
                body: hsla(0.0, 0.0, 1.0, 0.82),
                secondary_text: hsla(0.0, 0.0, 1.0, 0.62),
                hairline: hsla(0.0, 0.0, 1.0, 0.12),
                chip: hsla(0.0, 0.0, 1.0, 0.12),
                track: hsla(0.0, 0.0, 1.0, 0.14),
                primary_fill: white(),
                primary_hover: hsla(0.0, 0.0, 0.92, 1.0),
                primary_pressed: hsla(0.0, 0.0, 0.82, 1.0),
                on_primary: hsla(0.0, 0.0, 0.08, 1.0),
                secondary_fill: hsla(0.0, 0.0, 1.0, 0.14),
                secondary_hover: hsla(0.0, 0.0, 1.0, 0.22),
                secondary_pressed: hsla(0.0, 0.0, 1.0, 0.10),
                on_secondary: white(),
                star: hsla(36.0 / 360.0, 1.0, 0.58, 1.0),
                success: hsla(135.0 / 360.0, 0.64, 0.58, 1.0),
                danger: hsla(3.0 / 360.0, 0.90, 0.64, 1.0),
                dark: true,
            }
        } else {
            Self {
                // #F5F5F7 canvas, #FFFFFF cards, #1D1D1F ink.
                canvas: hsla(240.0 / 360.0, 0.11, 0.965, 1.0),
                surface: white(),
                ink: hsla(240.0 / 360.0, 0.03, 0.118, 1.0),
                body: hsla(240.0 / 360.0, 0.02, 0.22, 1.0),
                secondary_text: hsla(240.0 / 360.0, 0.02, 0.44, 1.0),
                hairline: hsla(0.0, 0.0, 0.0, 0.08),
                chip: hsla(240.0 / 360.0, 0.08, 0.94, 1.0),
                track: hsla(240.0 / 360.0, 0.06, 0.90, 1.0),
                primary_fill: hsla(240.0 / 360.0, 0.03, 0.118, 1.0),
                primary_hover: hsla(240.0 / 360.0, 0.02, 0.20, 1.0),
                primary_pressed: hsla(0.0, 0.0, 0.0, 1.0),
                on_primary: white(),
                secondary_fill: white(),
                secondary_hover: hsla(240.0 / 360.0, 0.08, 0.97, 1.0),
                secondary_pressed: hsla(240.0 / 360.0, 0.06, 0.93, 1.0),
                on_secondary: hsla(240.0 / 360.0, 0.03, 0.118, 1.0),
                star: hsla(36.0 / 360.0, 1.0, 0.50, 1.0),
                success: hsla(135.0 / 360.0, 0.59, 0.40, 1.0),
                danger: hsla(3.0 / 360.0, 0.86, 0.50, 1.0),
                dark: false,
            }
        }
    }

    pub fn from_cx(cx: &gpui_kit::App) -> Self {
        Self::resolve(cx.theme().is_dark())
    }

    pub fn poster_shadow(self, lifted: bool) -> Vec<BoxShadow> {
        let alpha = if self.dark {
            if lifted { 0.55 } else { 0.38 }
        } else if lifted {
            0.16
        } else {
            0.10
        };
        vec![
            BoxShadow {
                offset: point(px(0.), px(1.)),
                blur_radius: px(2.),
                spread_radius: px(0.),
                inset: false,
                color: hsla(0.0, 0.0, 0.0, if self.dark { 0.35 } else { 0.05 }),
            },
            BoxShadow {
                offset: point(px(0.), px(if lifted { 18. } else { 8. })),
                blur_radius: px(if lifted { 34. } else { 22. }),
                spread_radius: px(-8.),
                inset: false,
                color: hsla(0.0, 0.0, 0.0, alpha),
            },
        ]
    }

    pub fn button_shadow(self, primary: bool) -> Vec<BoxShadow> {
        let alpha = if self.dark {
            0.35
        } else if primary {
            0.16
        } else {
            0.08
        };
        vec![BoxShadow {
            offset: point(px(0.), px(if primary { 8. } else { 4. })),
            blur_radius: px(if primary { 18. } else { 12. }),
            spread_radius: px(if primary { -8. } else { -6. }),
            inset: false,
            color: hsla(0.0, 0.0, 0.0, alpha),
        }]
    }
}

#[derive(Clone, Copy)]
pub enum AppleButtonKind {
    Primary,
    Secondary,
    Glass,
}

pub fn apple_section_title(label: &str, paint: &ApplePaint) -> AnyElement {
    div()
        .text_size(px(22.))
        .font_semibold()
        .line_height(gpui_kit::relative(1.15))
        .text_color(paint.ink)
        .child(label.to_string())
        .into_any_element()
}

pub fn apple_progress(fraction: f32, color: Hsla, track: Hsla) -> AnyElement {
    let clamped = fraction.clamp(0.04, 1.0);
    div()
        .w_full()
        .h(px(4.))
        .rounded_full()
        .overflow_hidden()
        .bg(track)
        .child(
            div()
                .h_full()
                .w(gpui_kit::relative(clamped))
                .rounded_full()
                .bg(color),
        )
        .into_any_element()
}

pub fn apple_button(
    app: &StreamovieApp,
    id: impl Into<SharedString>,
    label: impl Into<String>,
    icon: Option<IconName>,
    kind: AppleButtonKind,
    compact: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let id = id.into();
    let label = label.into();
    let paint = ApplePaint::from_cx(cx);
    let hovered = app.control_is_hovered(&id);
    let pressed = app.control_is_pressed(&id);
    let glass = matches!(kind, AppleButtonKind::Glass);
    let primary = matches!(kind, AppleButtonKind::Primary);
    let height = if glass {
        36.
    } else if compact {
        32.
    } else {
        40.
    };
    let radius = if glass {
        18.
    } else if compact {
        12.
    } else {
        13.
    };
    let text_size = if compact { 12. } else { 14. };
    let pad_x = if glass {
        0.
    } else if compact {
        12.
    } else {
        16.
    };
    let (fill, fg) = match kind {
        AppleButtonKind::Primary if pressed => (paint.primary_pressed, paint.on_primary),
        AppleButtonKind::Primary if hovered => (paint.primary_hover, paint.on_primary),
        AppleButtonKind::Primary => (paint.primary_fill, paint.on_primary),
        AppleButtonKind::Secondary if pressed => (paint.secondary_pressed, paint.on_secondary),
        AppleButtonKind::Secondary if hovered => (paint.secondary_hover, paint.on_secondary),
        AppleButtonKind::Secondary => (paint.secondary_fill, paint.on_secondary),
        AppleButtonKind::Glass if pressed => (paint.secondary_pressed, paint.ink),
        AppleButtonKind::Glass if hovered => (paint.surface, paint.ink),
        AppleButtonKind::Glass => (paint.surface.opacity(0.88), paint.ink),
    };
    let scale_to = if pressed {
        0.96
    } else if hovered {
        1.03
    } else {
        1.0
    };
    let hover_id = id.clone();
    let press_id = id.clone();
    let release_id = id.clone();
    let spring_id = SharedString::from(format!("press-{id}"));

    let spacer = if glass {
        div().size(px(height)).into_any_element()
    } else {
        h_flex()
            .h(px(height))
            .px(px(pad_x))
            .gap(px(8.))
            .items_center()
            .opacity(0.)
            .when_some(icon.clone(), |row, icon| row.child(Icon::new(icon).small()))
            .child(
                div()
                    .text_size(px(text_size))
                    .font_semibold()
                    .child(label.clone()),
            )
            .into_any_element()
    };

    let face = h_flex()
        .id(id)
        .absolute()
        .top_0()
        .bottom_0()
        .left_0()
        .right_0()
        .gap(px(8.))
        .rounded(px(radius))
        .items_center()
        .justify_center()
        .cursor_pointer()
        .bg(fill)
        .text_color(fg)
        .when(!primary, |el| el.border_1().border_color(paint.hairline))
        .when(!glass, |el| el.px(px(pad_x)))
        .shadow(paint.button_shadow(primary))
        .when_some(icon, |row, icon| {
            row.child(Icon::new(icon).small().text_color(fg))
        })
        .when(!glass && !label.is_empty(), |row| {
            row.child(
                div()
                    .text_size(px(text_size))
                    .font_semibold()
                    .line_height(gpui_kit::relative(1.0))
                    .text_color(fg)
                    .child(label),
            )
        })
        .on_hover(cx.listener(move |this, hovered, _, cx| {
            this.set_control_hovered(&hover_id, *hovered, cx);
        }))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                this.set_control_pressed(&press_id, true, cx);
            }),
        )
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |this, _: &MouseUpEvent, _, cx| {
                this.set_control_pressed(&release_id, false, cx);
            }),
        )
        .on_click(on_click)
        .with_spring(
            spring_id,
            SpringAnimation::new(SpringConfig::new(420.0, 34.0, 1.0))
                .to(scale_to)
                .from(1.0),
            move |face, scale| {
                let inset_y = px((height * 0.5) * (1.0 - scale));
                let inset_x = px((if glass { height * 0.5 } else { 12. }) * (1.0 - scale));
                face.top(inset_y)
                    .bottom(inset_y)
                    .left(inset_x)
                    .right(inset_x)
            },
        );

    h_flex()
        .relative()
        .flex_shrink_0()
        .h(px(height))
        .child(spacer)
        .child(face)
        .into_any_element()
}
