//! The calendar under the clock: the time large, the month, and how busy the machine is.

use crate::theme::{TextStyle, colors, elevation};
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui_md3e::components::{Date, DatePickerStyle, circular_wavy_progress, date_picker};
use anti_webui_md3e::icons::{self, Symbol};
use std::time::Duration;

pub fn panel() -> Div {
    div()
        .flex()
        .flex_col()
        .gap(10.0)
        .p(12.0)
        .rounded(32.0)
        .shadows(elevation(3))
        .dyn_style(|s| {
            let c = colors();
            s.bg(c.surface_container).text_color(c.on_surface)
        })
        .child(header())
        .child(
            div()
                .rounded(24.0)
                .overflow_hidden()
                .flex()
                .justify_center()
                .dyn_style(|s| s.bg(colors().surface_container_high))
                .child(date_picker(
                    || {
                        let t = local_time(Every::Minute);
                        Some(Date {
                            year: t.year,
                            month: t.month as u32,
                            day: t.day as u32,
                        })
                    },
                    |_| {},
                    DatePickerStyle::new().no_header(),
                )),
        )
        .child(gauges())
}

/// The time with its seconds rolling, and the day.
fn header() -> Div {
    let big = TextStyle {
        size: 52.0,
        line_height: 60.0,
        letter_spacing: -0.5,
        weight: 400,
    };
    let small = TextStyle {
        size: 22.0,
        line_height: 28.0,
        letter_spacing: 0.0,
        weight: 500,
    };
    let width = |size: f32| {
        move |c: char| match c {
            '0'..='9' => size * 0.6,
            ':' => size * 0.3,
            _ => size * 0.5,
        }
    };
    let clock = move |f: &'static str| move || local_time(Every::Second).format(f);
    div()
        .flex()
        .flex_col()
        .px(14.0)
        .pt(6.0)
        .child(
            div()
                .flex()
                .flex_row()
                .items_end()
                .gap(6.0)
                .child(
                    rolling_text(clock("%H:%M"), big, width(52.0))
                        .dyn_style(|s| s.text_color(colors().primary)),
                )
                .child(
                    rolling_text(clock("%S"), small, width(22.0))
                        .mb(10.0)
                        .dyn_style(|s| s.text_color(colors().on_surface_variant)),
                ),
        )
        .child(
            txt(TextStyle::TITLE_MEDIUM, || {
                local_time(Every::Minute).format("%A, %-d %B %Y")
            })
            .dyn_style(|s| s.text_color(colors().on_surface_variant)),
        )
}

/// CPU, memory and temperature as wavy rings.
fn gauges() -> Div {
    let sys = std::rc::Rc::new(system_every(Duration::from_secs(2)));
    let cpu = memo({
        let sys = sys.clone();
        move || sys.cpu()
    });
    let memory = memo({
        let sys = sys.clone();
        move || {
            let m = sys.memory();
            (m.used, m.total)
        }
    });
    let heat = memo(move || sys.cpu_temperature());
    let gauge = |symbol: &'static Symbol,
                 label: &'static str,
                 value: Box<dyn Fn() -> f32>,
                 shown: Box<dyn Fn() -> String>| {
        let value: std::rc::Rc<dyn Fn() -> f32> = value.into();
        div()
            .flex_1()
            .flex()
            .flex_row()
            .items_center()
            .gap(10.0)
            .p(10.0)
            .rounded(20.0)
            .dyn_style(|s| s.bg(colors().surface_container_high))
            .child(
                div()
                    .size(48.0)
                    .relative()
                    .flex_none()
                    .child(
                        circular_wavy_progress(move || value())
                            .absolute()
                            .inset(0.0),
                    )
                    .child(
                        div()
                            .absolute()
                            .inset(0.0)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(sym(symbol, 18.0, |c| c.primary)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(txt(TextStyle::TITLE_SMALL, shown))
                    .child(
                        static_txt(TextStyle::LABEL_SMALL, label)
                            .dyn_style(|s| s.text_color(colors().on_surface_variant)),
                    ),
            )
    };
    let gib = |b: u64| b as f64 / (1u64 << 30) as f64;
    div()
        .flex()
        .flex_row()
        .gap(8.0)
        .child(gauge(
            icons::SPEED,
            "CPU",
            Box::new(move || cpu.get() as f32),
            Box::new(move || format!("{:.0}%", cpu.get() * 100.0)),
        ))
        .child(gauge(
            icons::MEMORY,
            "Memory",
            Box::new(move || {
                let (used, total) = memory.get();
                (used as f64 / total.max(1) as f64) as f32
            }),
            Box::new(move || format!("{:.1} GiB", gib(memory.get().0))),
        ))
        .child(gauge(
            icons::THERMOSTAT,
            "Heat",
            Box::new(move || ((heat.get().unwrap_or(0.0) - 30.0) / 70.0).clamp(0.0, 1.0) as f32),
            Box::new(move || {
                heat.get()
                    .map(|t| format!("{t:.0}°C"))
                    .unwrap_or_else(|| "—".into())
            }),
        ))
}
