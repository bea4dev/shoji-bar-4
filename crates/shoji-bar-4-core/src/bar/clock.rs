use crate::bar::island::{Island, bar_island};
use crate::panels::{Panel, is_open, toggle};
use crate::placement::BarPlacement;
use crate::settings::settings;
use crate::theme::{TextStyle, colors};
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui_md3e::motion::fast_spatial;

#[derive(Clone, Copy)]
enum ClockPart {
    Full,
    Hour,
    Minute,
    Second,
    Meridian,
}

fn clock_text(part: ClockPart) -> String {
    let s = settings().get();
    let t = local_time(if s.clock_seconds {
        Every::Second
    } else {
        Every::Minute
    });
    match part {
        ClockPart::Full => {
            let hm = if s.clock_24h { "%H:%M" } else { "%I:%M" };
            t.format(&if s.clock_seconds {
                format!("{hm}:%S")
            } else {
                hm.to_owned()
            })
        }
        ClockPart::Hour => t.format(if s.clock_24h { "%H" } else { "%I" }),
        ClockPart::Minute => t.format("%M"),
        ClockPart::Second => t.format("%S"),
        ClockPart::Meridian => t.format("%p"),
    }
}

/// The time, its digits rolling over as they change, and the date. Hovering makes the time
/// heavier; a click opens the calendar. A vertical bar stacks the parts instead of showing the date.
fn clock_island(output: Output, at: BarPlacement) -> Div {
    let vertical = at.is_vertical();
    let hovered = signal(false);
    let weight = follow(
        move || if hovered.get() { 760.0 } else { 520.0 },
        fast_spatial(),
    );
    let open = memo({
        let output = output.clone();
        move || is_open(&output, Panel::Calendar)
    });
    let size = 15.0;
    let style = TextStyle {
        size,
        line_height: 20.0,
        letter_spacing: 0.0,
        weight: 500,
    };
    let digit_width = move |c| match c {
        '0'..='9' => size * 0.6,
        ':' => size * 0.32,
        _ => size * 0.5,
    };
    let time = if vertical {
        div()
            .flex()
            .flex_col()
            .items_center()
            .child(rolling_text(
                || clock_text(ClockPart::Hour),
                style,
                digit_width,
            ))
            .child(rolling_text(
                || clock_text(ClockPart::Minute),
                style,
                digit_width,
            ))
            .child(show(
                || settings().with(|s| s.clock_seconds),
                || {
                    rolling_text(
                        || clock_text(ClockPart::Second),
                        TextStyle::LABEL_SMALL,
                        |c| if c.is_ascii_digit() { 7.0 } else { 5.0 },
                    )
                },
            ))
            .child(show(
                || !settings().with(|s| s.clock_24h),
                || txt(TextStyle::LABEL_SMALL, || clock_text(ClockPart::Meridian)),
            ))
    } else {
        rolling_text(|| clock_text(ClockPart::Full), style, digit_width)
    }
    .dyn_style(move |s| s.font_axis(b"wght", weight.get()));
    let content = if vertical {
        bar_island(at).py(10.0_f32).gap(2.0_f32)
    } else {
        bar_island(at).px(14.0_f32).gap(8.0_f32)
    };
    let content = content
        .on_hover(move |h| hovered.set(h))
        .on_click(move |_| toggle(&output, Panel::Calendar))
        .dyn_style(move |s| {
            let c = colors();
            if open.get() {
                s.bg(c.primary_container).text_color(c.on_primary_container)
            } else {
                s
            }
        })
        .child(time);
    let content = if vertical {
        content
    } else {
        content.child(
            txt(TextStyle::LABEL_SMALL, || {
                local_time(Every::Minute).format("%a %-d %b")
            })
            .dyn_style(move |s| {
                let c = colors();
                s.text_color(if open.get() {
                    c.on_primary_container
                } else {
                    c.on_surface_variant
                })
            }),
        )
    };
    pressable(content, |c| c.on_surface)
}

/// The clock, as an island of the bar.
pub fn island() -> Island {
    Island::new("clock", |cx| {
        clock_island(cx.output.clone(), cx.placement).into_view()
    })
}
