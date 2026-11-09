//! The bar along the top of every output: the launcher and the workspaces on the left, the window
//! with the keyboard, the clock in the middle, and what is playing, the tray and the state of the
//! system on the right. Each group floats in an island of its own.

use crate::appinfo::window_icon;
use crate::panels::{Panel, is_open, toggle};
use crate::settings::settings;
use crate::theme::{TextStyle, colors};
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui::shell::{BatteryState, Connectivity};
use anti_webui_md3e::icons;
use anti_webui_md3e::motion::{fast_spatial, slow_spatial};
use std::cell::Cell;
use std::rc::Rc;

/// How tall the layer is; the islands are [`ISLAND`] of it, with room above and below.
pub const HEIGHT: i32 = 48;

pub fn open(output: Output) {
    open_layer(
        LayerOptions::bar(Anchor::TOP, HEIGHT).namespace("shoji-bar-4-bar"),
        move || view(output),
    );
}

pub fn view(output: Output) -> Div {
    div()
        .size_full()
        .flex()
        .flex_row()
        .items_center()
        .px(14.0)
        .gap(10.0)
        .dyn_style(|s| s.text_color(colors().on_surface))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(10.0)
                .flex_1()
                .min_w(0.0)
                .child(launcher_island(output.clone()))
                .child(workspaces_island(
                    workspaces_of(output.clone()),
                    Rc::new({
                        let output = output.clone();
                        move |id: &str| {
                            if let Some(w) =
                                workspaces().on(&output).into_iter().find(|w| w.id() == id)
                            {
                                w.activate();
                            }
                        }
                    }),
                ))
                .child(window_island(output.clone())),
        )
        .child(clock_island(output.clone()))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_end()
                .gap(10.0)
                .flex_1()
                .min_w(0.0)
                .child(media_island())
                .child(tray_island())
                .child(status_island(output)),
        )
}

/// The button that opens the launcher: an icon on a shape that turns and blooms on hover.
fn launcher_island(output: Output) -> Div {
    let hovered = signal(false);
    let open = memo({
        let output = output.clone();
        move || is_open(&output, Panel::Launcher)
    });
    island()
        .w(ISLAND)
        .justify_center()
        .on_hover(move |h| hovered.set(h))
        .on_click(move |_| toggle(&output, Panel::Launcher))
        .child(launcher_glyph(open, hovered, || 15.0).size(26.0))
}

/// A workspace as the bar shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct Ws {
    pub id: String,
    pub name: String,
    pub active: bool,
    /// Has no windows.
    pub empty: bool,
    pub urgent: bool,
}

/// The workspaces of an output, in order.
fn workspaces_of(output: Output) -> Memo<Vec<Ws>> {
    memo(move || {
        let mut all = workspaces().on(&output);
        all.sort_by(|a, b| a.coordinates().cmp(b.coordinates()));
        all.iter()
            .map(|w| Ws {
                id: w.id().to_owned(),
                name: w.name().to_owned(),
                active: w.is_active(),
                empty: w.is_hidden(),
                urgent: w.is_urgent(),
            })
            .collect()
    })
}

const SLOT: f32 = 18.0;
const ACTIVE: f32 = 32.0;
/// How tall the pill under the active workspace is.
const PILL: f32 = 22.0;

/// The workspaces of the output as dots, with a pill under the active one that stretches as it
/// goes from one to another: its front edge leads on a quick spring, its back follows on a slow
/// one.
pub fn workspaces_island(list: Memo<Vec<Ws>>, activate: Rc<dyn Fn(&str)>) -> Div {
    let active = memo(move || list.with(|l| l.iter().position(|w| w.active)).unwrap_or(0));
    let scrolled = activate.clone();
    let left = animated(0.0f32);
    let right = animated(ACTIVE);
    let first = Cell::new(true);
    effect(move || {
        let at = active.get() as f32 * SLOT;
        let (to_left, to_right) = (at, at + ACTIVE);
        untrack(|| {
            if first.replace(false) {
                left.set(to_left);
                right.set(to_right);
            } else if to_left >= left.get_untracked() {
                right.spring_to(to_right, Spring::from_ratio(900.0, 0.7));
                left.spring_to(to_left, Spring::from_ratio(260.0, 0.85));
            } else {
                left.spring_to(to_left, Spring::from_ratio(900.0, 0.7));
                right.spring_to(to_right, Spring::from_ratio(260.0, 0.85));
            }
        })
    });
    let pill = div()
        .absolute()
        .top(0.0)
        .h(PILL)
        .rounded(PILL / 2.0)
        .dyn_style(move |s| {
            let (l, r) = (left.get(), right.get());
            s.left(l).w((r - l).max(PILL)).bg(colors().primary)
        });
    let slots =
        for_each(
            move || list.get(),
            |w| w.id.clone(),
            move |w| {
                let id = w.id.clone();
                let me = memo(move || list.with(|l| l.iter().find(|x| x.id == id).cloned()));
                let on = memo(move || me.with(|w| w.as_ref().is_some_and(|w| w.active)));
                let empty = memo(move || me.with(|w| w.as_ref().is_none_or(|w| w.empty)));
                let urgent = memo(move || me.with(|w| w.as_ref().is_some_and(|w| w.urgent)));
                let name = w.name.clone();
                let activate = activate.clone();
                div()
                    .relative()
                    .h(PILL)
                    .flex_none()
                    .transition_of([Prop::Width], emphasized(350))
                    .dyn_style(move |s| s.w(if on.get() { ACTIVE } else { SLOT }))
                    .on_click(move |_| activate(&w.id))
                    // The dot, which gives way to the number.
                    .child(
                        div()
                            .absolute()
                            .inset(0.0)
                            .flex()
                            .items_center()
                            .justify_center()
                            .pointer_events_none()
                            .child(div().rounded(3.0).transition(overshoot(350)).dyn_style(
                                move |s| {
                                    let c = colors();
                                    let size = if empty.get() { 5.0 } else { 6.0 };
                                    let color = if urgent.get() {
                                        c.error
                                    } else if empty.get() {
                                        c.outline_variant
                                    } else {
                                        c.on_surface_variant
                                    };
                                    s.size(size)
                                        .bg(color)
                                        .scale(if on.get() { 0.0 } else { 1.0 })
                                },
                            )),
                    )
                    .child(
                        TextStyle::LABEL_MEDIUM
                            .apply(div())
                            .absolute()
                            .inset(0.0)
                            .flex()
                            .items_center()
                            .justify_center()
                            .pointer_events_none()
                            .transition(overshoot(400))
                            .dyn_style(move |s| {
                                s.text_color(colors().on_primary)
                                    .opacity(if on.get() { 1.0 } else { 0.0 })
                                    .scale(if on.get() { 1.0 } else { 0.4 })
                            })
                            .child(name),
                    )
            },
        )
        .enter(|s| s.opacity(0.0).scale(0.3))
        .exit(|s| s.opacity(0.0).scale(0.3))
        .transition(overshoot(300));
    island()
        .px(5.0)
        .on_scroll(move |e| {
            let step = if e.delta_y > 0.0 {
                1
            } else if e.delta_y < 0.0 {
                -1
            } else {
                0
            };
            let all = untrack(|| list.get());
            let next = untrack(|| active.get()) as i32 + step;
            if step != 0
                && let Some(w) = usize::try_from(next).ok().and_then(|i| all.get(i))
            {
                scrolled(&w.id);
            }
        })
        .child(
            div()
                .relative()
                .flex()
                .flex_row()
                .items_center()
                .h(PILL)
                .child(pill)
                .child(slots),
        )
}

/// The window with the keyboard on this output: its icon and title, which slide over when another
/// window takes the keyboard.
fn window_island(output: Output) -> Show {
    let active = memo(move || {
        toplevels()
            .active()
            .filter(|t| t.outputs.is_empty() || t.outputs.contains(&output.name))
    });
    show(
        move || active.with(Option::is_some),
        move || {
            island()
                .px(5.0)
                .min_w(0.0)
                .flex_shrink(1.0)
                .overflow_hidden()
                .child(
                    for_each(
                        move || active.get().into_iter().map(|t| t.id()).collect::<Vec<_>>(),
                        |id| *id,
                        move |_| {
                            let shown = memo(move || active.get());
                            let icon = memo(move || {
                                shown.with(|t| t.as_ref().and_then(|t| window_icon(t, 64)))
                            });
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(8.0)
                                .pr(9.0)
                                .min_w(0.0)
                                .child(dyn_view(move || match icon.get() {
                                    Some(path) => img(path).size(22.0).flex_none().into_view(),
                                    None => div()
                                        .size(22.0)
                                        .rounded(11.0)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .flex_none()
                                        .dyn_style(|s| s.bg(colors().secondary_container))
                                        .child(sym(icons::DESKTOP_WINDOWS, 13.0, |c| {
                                            c.on_secondary_container
                                        }))
                                        .into_view(),
                                }))
                                .child(
                                    txt(BAR_TEXT, move || {
                                        shown.with(|t| {
                                            t.as_ref()
                                                .map(|t| t.title.chars().take(60).collect())
                                                .unwrap_or_default()
                                        })
                                    })
                                    .max_w(320.0)
                                    .overflow_hidden(),
                                )
                        },
                    )
                    .enter(|s| s.opacity(0.0).translate_y(16.0))
                    .exit(|s| s.opacity(0.0).translate_y(-16.0))
                    .transition(emphasized(350)),
                )
        },
    )
    .enter(|s| s.opacity(0.0).scale(0.8))
    .exit(|s| s.opacity(0.0).scale(0.8))
    .transition(overshoot(350))
}

/// The time, its digits rolling over as they change, and the date. Hovering makes the time
/// heavier; a click opens the calendar.
fn clock_island(output: Output) -> Div {
    let hovered = signal(false);
    let weight = follow(
        move || if hovered.get() { 760.0 } else { 520.0 },
        fast_spatial(),
    );
    let open = memo({
        let output = output.clone();
        move || is_open(&output, Panel::Calendar)
    });
    let time = move || {
        let s = settings().get();
        let t = local_time(if s.clock_seconds {
            Every::Second
        } else {
            Every::Minute
        });
        let hm = if s.clock_24h { "%H:%M" } else { "%I:%M" };
        t.format(&if s.clock_seconds {
            format!("{hm}:%S")
        } else {
            hm.to_owned()
        })
    };
    let size = 15.0;
    let style = TextStyle {
        size,
        line_height: 20.0,
        letter_spacing: 0.0,
        weight: 500,
    };
    pressable(
        island()
            .px(14.0)
            .gap(8.0)
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
            .child(
                rolling_text(time, style, move |c| match c {
                    '0'..='9' => size * 0.6,
                    ':' => size * 0.32,
                    _ => size * 0.5,
                })
                .dyn_style(move |s| s.font_axis(b"wght", weight.get())),
            )
            .child(
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
            ),
        |c| c.on_surface,
    )
}

/// What is playing, while something plays: a little equalizer and the title. A click pauses it or
/// plays it again.
fn media_island() -> Show {
    let m = media();
    let current = memo(move || m.current());
    show(
        move || current.with(|p| p.as_ref().is_some_and(|p| !p.title.is_empty())),
        move || {
            let playing = memo(move || {
                current.with(|p| {
                    p.as_ref()
                        .is_some_and(|p| p.status == PlaybackStatus::Playing)
                })
            });
            pressable(
                island()
                    .px(12.0)
                    .gap(8.0)
                    .max_w(260.0)
                    .on_click(move |_| {
                        if let Some(p) = untrack(|| current.get()) {
                            p.play_pause();
                        }
                    })
                    .child(equalizer(playing))
                    .child(
                        txt(BAR_TEXT, move || {
                            current.with(|p| {
                                p.as_ref()
                                    .map(|p| p.title.chars().take(40).collect())
                                    .unwrap_or_default()
                            })
                        })
                        .min_w(0.0)
                        .overflow_hidden(),
                    ),
                |c| c.on_surface,
            )
        },
    )
    .enter(|s| s.opacity(0.0).scale(0.6))
    .exit(|s| s.opacity(0.0).scale(0.6))
    .transition(overshoot(400))
}

/// Four bars that dance while `playing` and settle into dots when paused.
fn equalizer(playing: Memo<bool>) -> Div {
    let calming = follow(
        move || if playing.get() { 1.0 } else { 0.0 },
        slow_spatial(),
    );
    // Thirty times a second is plenty for bars that dance, and a quarter of the frames.
    let t = clock_at(30, move || calming.get() > 0.001);
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(2.0)
        .h(13.0)
        .flex_none();
    for i in 0..4 {
        row = row.child(div().w(2.5).rounded(1.25).dyn_style(move |s| {
            let level = calming.get();
            // Only asks for frames while it moves.
            let wave = if level > 0.001 {
                let x = t.get() * (5.0 + i as f32 * 1.3) + i as f32 * 1.7;
                (x.sin() * 0.5 + 0.5) * 0.7 + 0.3
            } else {
                0.0
            };
            s.h(2.5 + 10.5 * wave * level).bg(colors().primary)
        }));
    }
    row
}

/// The icons of the tray.
fn tray_island() -> Show {
    show(
        || !tray().items().is_empty(),
        || {
            island().px(5.0).gap(2.0).child(
                for_each(|| tray().items(), |item| item.key(), tray_icon)
                    .enter(|s| s.opacity(0.0).scale(0.4))
                    .exit(|s| s.opacity(0.0).scale(0.4))
                    .transition(overshoot(300)),
            )
        },
    )
    .enter(|s| s.opacity(0.0).scale(0.8))
    .exit(|s| s.opacity(0.0).scale(0.8))
    .transition(overshoot(300))
}

fn tray_icon(item: TrayItem) -> Div {
    let open = signal(false);
    let key = item.key();
    let now = move || tray().items().into_iter().find(|i| i.key() == key);
    let shown = now.clone();
    let clicked = item.clone();
    pressable(
        div()
            .size(24.0)
            .rounded(12.0)
            .flex()
            .items_center()
            .justify_center()
            .child(dyn_view(move || {
                match shown().and_then(|i| i.icon.best(48)) {
                    Some(picture) => bitmap_view(move || Some(picture.clone()))
                        .size(16.0)
                        .into_view(),
                    None => txt(TextStyle::LABEL_MEDIUM, {
                        let shown = shown.clone();
                        move || {
                            shown()
                                .and_then(|i| i.title.chars().next())
                                .map(String::from)
                                .unwrap_or_default()
                        }
                    })
                    .into_view(),
                }
            }))
            .on_pointer_down(move |e| match e.button {
                Some(PointerButton::Secondary) if clicked.has_menu => open.set(true),
                Some(PointerButton::Primary) if clicked.item_is_menu => open.set(true),
                Some(PointerButton::Primary) => {
                    clicked.activate(0, 0);
                }
                _ => {}
            })
            .child(tray_menu(&item, open)),
        |c| c.on_surface,
    )
}

/// The size of the words in the bar: smaller than Material's titles, as a bar is.
const BAR_TEXT: TextStyle = TextStyle {
    size: 13.0,
    line_height: 18.0,
    letter_spacing: 0.1,
    weight: 500,
};

/// Network, Bluetooth, sound and battery at a glance; a click opens quick settings.
fn status_island(output: Output) -> Div {
    let net = network();
    let bt = bluetooth();
    let a = audio();
    let b = battery();
    let open = memo({
        let output = output.clone();
        move || is_open(&output, Panel::QuickSettings)
    });
    let color = move |c: &anti_webui_md3e::ColorScheme| c.on_surface_variant;
    pressable(
        island()
            .px(12.0)
            .gap(9.0)
            .on_click(move |_| toggle(&output, Panel::QuickSettings))
            .dyn_style(move |s| {
                if open.get() {
                    s.bg(colors().primary_container)
                } else {
                    s
                }
            })
            .child(dyn_sym(
                move || match net.wifi() {
                    Some(ap) => wifi_icon(ap.strength),
                    None if net.wired().is_some_and(|w| w.connected) => icons::LAN,
                    None if net.connectivity() == Connectivity::None => icons::SIGNAL_WIFI_OFF,
                    None => icons::WIFI_OFF,
                },
                16.0,
                color,
            ))
            .child(show(
                move || bt.available() && bt.adapter().is_some_and(|a| a.powered),
                move || {
                    dyn_sym(
                        move || {
                            if bt.devices().iter().any(|d| d.connected) {
                                icons::BLUETOOTH_CONNECTED
                            } else {
                                icons::BLUETOOTH
                            }
                        },
                        16.0,
                        color,
                    )
                },
            ))
            .child(dyn_sym(
                move || match a.default_sink() {
                    Some(s) if s.muted || s.volume <= 0.001 => icons::VOLUME_OFF,
                    Some(s) if s.volume < 0.5 => icons::VOLUME_DOWN,
                    _ => icons::VOLUME_UP,
                },
                16.0,
                color,
            ))
            .child(show(
                move || b.available(),
                move || {
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(2.0)
                        .child(dyn_sym(
                            move || battery_icon(b.percentage(), b.state()),
                            16.0,
                            color,
                        ))
                        .child(txt(TextStyle::LABEL_MEDIUM, move || {
                            format!("{:.0}%", b.percentage())
                        }))
                },
            ))
            .dyn_style(move |s| {
                if open.get() {
                    s.text_color(colors().on_primary_container)
                } else {
                    s
                }
            }),
        |c| c.on_surface,
    )
}

pub fn wifi_icon(strength: u8) -> &'static anti_webui_md3e::icons::Symbol {
    match strength {
        75.. => icons::SIGNAL_WIFI_4_BAR,
        50.. => icons::NETWORK_WIFI_3_BAR,
        25.. => icons::NETWORK_WIFI_2_BAR,
        5.. => icons::NETWORK_WIFI_1_BAR,
        _ => icons::SIGNAL_WIFI_0_BAR,
    }
}

pub fn battery_icon(
    percentage: f64,
    state: BatteryState,
) -> &'static anti_webui_md3e::icons::Symbol {
    let charging = matches!(
        state,
        BatteryState::Charging | BatteryState::Full | BatteryState::PendingCharge
    );
    if charging {
        return match percentage as u32 {
            95.. => icons::BATTERY_CHARGING_FULL,
            65.. => icons::BATTERY_CHARGING_80,
            _ => icons::BATTERY_CHARGING_50,
        };
    }
    match percentage as u32 {
        95.. => icons::BATTERY_FULL,
        80.. => icons::BATTERY_6_BAR,
        65.. => icons::BATTERY_5_BAR,
        50.. => icons::BATTERY_4_BAR,
        35.. => icons::BATTERY_3_BAR,
        20.. => icons::BATTERY_2_BAR,
        8.. => icons::BATTERY_1_BAR,
        _ => icons::BATTERY_0_BAR,
    }
}
