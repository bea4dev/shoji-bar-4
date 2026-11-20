use std::rc::Rc;

use crate::bar::island::Island;
use crate::context::Part;
use crate::layout::{by_placement, row};
use crate::panels::{Panel, is_open, toggle};
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

/// The time, its digits rolling over as they change; hovering makes it heavier. A vertical bar
/// stacks the hour and the minute (and the seconds and the meridian, when they are shown).
pub fn time() -> Part {
    Rc::new(|cx| {
        let hovered = cx.island.hovered;
        let weight = follow(
            move || if hovered.get() { 760.0 } else { 520.0 },
            fast_spatial(),
        );
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
        let time = if cx.placement.is_vertical() {
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
        };
        time.dyn_style(move |s| s.font_axis(b"wght", weight.get()))
            .into_view()
    })
}

/// The date, which turns darker while the calendar is open.
pub fn date() -> Part {
    Rc::new(|cx| {
        let active = cx.island.active;
        txt(TextStyle::LABEL_SMALL, || {
            local_time(Every::Minute).format("%a %-d %b")
        })
        .dyn_style(move |s| {
            let c = colors();
            s.text_color(if active.get() {
                c.on_primary_container
            } else {
                c.on_surface_variant
            })
        })
        .into_view()
    })
}

/// The clock, as an island of the bar.
pub fn island() -> Island {
    let horizontal = row(8.0, [time(), date()]);
    let vertical = time();
    Island::new("clock", by_placement(horizontal, vertical))
        .padding(14.0, 10.0)
        .active(|cx| is_open(&cx.output, Panel::Calendar))
        .on_click(|cx| toggle(&cx.output, Panel::Calendar))
}
