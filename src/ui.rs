//! Small pieces the bar, the dock and the panels share.

use crate::theme::{TextStyle, colors, elevation};
use anti_webui::prelude::*;
use anti_webui_md3e::ColorScheme;
use anti_webui_md3e::components::sized_icon;
use anti_webui_md3e::icons::Symbol;

/// A color of the scheme, chosen by a function of it.
pub trait Role: Fn(&ColorScheme) -> Color + Copy + 'static {}
impl<F: Fn(&ColorScheme) -> Color + Copy + 'static> Role for F {}

/// Material's emphasized easing, for what moves in or out of view.
pub fn emphasized(ms: u64) -> Transition {
    Transition::ms(ms).cubic_bezier(0.2, 0.0, 0.0, 1.0)
}

/// An easing that goes a little past the end and comes back, for things that pop in.
pub fn overshoot(ms: u64) -> Transition {
    Transition::ms(ms).cubic_bezier(0.34, 1.45, 0.64, 1.0)
}

/// An icon of a size, in a color of the scheme.
pub fn sym(symbol: &'static Symbol, size: f32, color: impl Role) -> Div {
    sized_icon(symbol, size).dyn_style(move |s| s.text_color(color(&colors())))
}

/// An icon that changes with what `which` says.
pub fn dyn_sym(which: impl Fn() -> &'static Symbol + 'static, size: f32, color: impl Role) -> Div {
    let which = memo(move || SameSymbol(which()));
    div()
        .size(size)
        .flex_none()
        .child(dyn_view(move || sym(which.get().0, size, color)))
}

/// A symbol compared by which one it is.
#[derive(Clone, Copy)]
struct SameSymbol(&'static Symbol);

impl PartialEq for SameSymbol {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

/// Text in a style of the type scale, which follows what `f` says.
pub fn txt(style: TextStyle, f: impl Fn() -> String + 'static) -> Div {
    style.apply(div().whitespace_nowrap().child(text(f)))
}

/// Text that stays as it is.
pub fn static_txt(style: TextStyle, s: impl Into<String>) -> Div {
    style.apply(div().whitespace_nowrap().child(s.into()))
}

/// A floating container of the bar: a pill on `surface_container`.
pub fn island() -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .flex_none()
        .h(40.0)
        .rounded(20.0)
        .shadows(elevation(1))
        .dyn_style(|s| s.bg(colors().surface_container))
}

/// Gives `node` a state layer: a veil of `color` that shows on hover and deepens while pressed, in
/// a box of `radius`. `node` becomes `relative`.
pub fn pressable(node: Div, radius: f32, color: impl Role) -> Div {
    let hovered = signal(false);
    let pressed = signal(false);
    node.relative()
        .on_hover(move |h| hovered.set(h))
        .on_active(move |a| pressed.set(a))
        .child(
            div()
                .absolute()
                .inset(0.0)
                .rounded(radius)
                .pointer_events_none()
                .transition(Transition::ms(120).ease_out())
                .dyn_style(move |s| {
                    let level = if pressed.get() {
                        0.12
                    } else if hovered.get() {
                        0.08
                    } else {
                        0.0
                    };
                    s.bg(color(&colors()).with_alpha(level))
                }),
        )
}

/// A round button of an icon, `size` across, with its state layer.
pub fn icon_btn(symbol: &'static Symbol, size: f32, color: impl Role) -> Div {
    pressable(
        div()
            .size(size)
            .flex()
            .items_center()
            .justify_center()
            .flex_none()
            .rounded(size / 2.0)
            .child(sym(symbol, (size * 0.55).round(), color)),
        size / 2.0,
        color,
    )
}

/// Text whose characters roll into place when they change: the old one goes up and out, the new
/// one comes up from below. `width` says how wide each character's cell is.
pub fn rolling_text(
    f: impl Fn() -> String + 'static,
    style: TextStyle,
    width: impl Fn(char) -> f32 + Copy + 'static,
) -> Div {
    let chars = memo(move || f().chars().collect::<Vec<char>>());
    let height = style.line_height;
    div()
        .flex()
        .flex_row()
        .flex_none()
        .child(for_each(
            move || (0..chars.with(|c| c.len())).collect::<Vec<_>>(),
            |i| *i,
            move |i| {
                let ch = memo(move || chars.with(|c| c.get(i).copied().unwrap_or(' ')));
                div()
                    .relative()
                    .h(height)
                    .flex_none()
                    .overflow_hidden()
                    .transition_of([Prop::Width], emphasized(400))
                    .dyn_style(move |s| s.w(width(ch.get())))
                    .child(
                        for_each(
                            move || vec![ch.get()],
                            |c| *c,
                            move |c| {
                                style.apply(
                                    div()
                                        .absolute()
                                        .inset(0.0)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(c.to_string()),
                                )
                            },
                        )
                        .enter(move |s| s.translate_y(height * 0.9).opacity(0.0))
                        .exit(move |s| s.translate_y(-height * 0.9).opacity(0.0))
                        .transition(emphasized(550)),
                    )
            },
        ))
}
