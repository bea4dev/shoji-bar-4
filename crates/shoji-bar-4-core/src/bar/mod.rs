//! The bar along an edge of every output: the launcher and the workspaces at the start, the window
//! with the keyboard and the clock in the middle, and what is playing, the tray and the state of the
//! system at the end. Each group floats in an island of its own, on a pane of frosted glass that is
//! all of the bar. Vertical bars use compact icons and stack the clock.

use crate::appinfo::tray_icon_image;
use crate::bar::island::{Island, VERTICAL_ISLAND, bar_island};
use crate::bar::window::window_island;
use crate::bar::workspaces::workspaces_of;
use crate::bar::launcher::launcher_island;
use crate::panels::{Panel, is_open, toggle};
use crate::placement::{AnchorPoint, BarPlacement, PanelSide};
use crate::settings::settings;
use crate::theme::{TextStyle, colors, hex_of, target_scheme};
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui::shell::{BatteryState, Connectivity, MenuToggle, TrayMenuItem};
use anti_webui_md3e::components::{MenuItem, MenuStyle, menu_item, styled_menu};
use anti_webui_md3e::icons;
use anti_webui_md3e::motion::{fast_spatial, slow_spatial};
use std::rc::Rc;

pub mod island;
pub mod launcher;
pub mod window;
pub mod workspaces;

pub use window::{Focused, focused_island};
pub use workspaces::{Ws, workspaces_island};

/// The size of the words in the bar: smaller than Material's titles, as a bar is.
pub(crate) const BAR_TEXT: TextStyle = TextStyle {
    size: 13.0,
    line_height: 18.0,
    letter_spacing: 0.1,
    weight: 500,
};

#[derive(Clone, Default)]
pub struct Bar {
    pub left: Vec<Island>,
    pub center: Vec<Island>,
    pub right: Vec<Island>,
}

impl Bar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn left(mut self, islands: impl IntoIterator<Item = Island>) -> Self {
        self.left = islands.into_iter().collect();
        self
    }

    pub fn center(mut self, islands: impl IntoIterator<Item = Island>) -> Self {
        self.center = islands.into_iter().collect();
        self
    }

    pub fn right(mut self, islands: impl IntoIterator<Item = Island>) -> Self {
        self.right = islands.into_iter().collect();
        self
    }
}

pub fn view(output: Output, at: BarPlacement) -> Div {
    let vertical = at.is_vertical();
    let mut root = div()
        .select_none()
        .size_full()
        .relative()
        // Its own stacking context, for the glass to go behind the islands but not behind this.
        .z_index(0)
        .flex()
        .items_center()
        .gap(10.0_f32);
    root = if vertical {
        root.flex_col().px(6.0_f32).py(14.0_f32)
    } else {
        root.flex_row().px(14.0_f32)
    };
    let leading = div().flex().items_center().gap(10.0_f32).flex_1();
    let leading = if vertical {
        leading.flex_col().min_h(0.0)
    } else {
        leading.flex_row().min_w(0.0)
    };
    let trailing = div()
        .flex()
        .items_center()
        .justify_end()
        .gap(10.0_f32)
        .flex_1();
    let trailing = if vertical {
        trailing.flex_col().min_h(0.0)
    } else {
        trailing.flex_row().min_w(0.0)
    };
    root.dyn_style(|s| s.text_color(colors().on_surface))
        // Faint, but enough for the compositor to blur what is behind it: it blurs behind what is
        // at least a quarter opaque.
        .child(
            div()
                .absolute()
                .z_index(-1)
                .inset(0.0)
                .pointer_events_none()
                .dyn_style(|s| s.bg(colors().surface.with_alpha(0.34))),
        )
        .child(
            leading
                .child(launcher_island(output.clone(), at))
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
                    at,
                ))
                .child(window_island(output.clone(), at)),
        )
        .child(clock_island(output.clone(), at))
        .child(
            trailing
                .child(media_island(output.clone(), at))
                .child(tray_island(at))
                .child(status_island(output, at)),
        )
}

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

/// What is playing, while something plays: a little equalizer and, on the top bar, its title.
/// A click opens the player beside it; the middle button pauses it or plays it again.
fn media_island(output: Output, at: BarPlacement) -> Show {
    let vertical = at.is_vertical();
    let m = media();
    let current = memo(move || m.current());
    let open = memo({
        let output = output.clone();
        move || is_open(&output, Panel::Media)
    });
    show(
        move || current.with(|p| p.as_ref().is_some_and(|p| !p.title.is_empty())),
        move || {
            let output = output.clone();
            let playing = memo(move || {
                current.with(|p| {
                    p.as_ref()
                        .is_some_and(|p| p.status == PlaybackStatus::Playing)
                })
            });
            let here = node_ref();
            let content = if vertical {
                bar_island(at).size(VERTICAL_ISLAND).justify_center()
            } else {
                bar_island(at).px(12.0_f32).gap(8.0_f32).max_w(260.0)
            };
            let content = content
                .node_ref(here)
                .on_click(move |_| {
                    if let Some(r) = here.logical_rect() {
                        crate::panels::toggle_at(
                            &output,
                            Panel::Media,
                            AnchorPoint {
                                x: r.x + r.width / 2.0,
                                y: r.y + r.height / 2.0,
                            },
                        );
                    } else {
                        toggle(&output, Panel::Media);
                    }
                })
                .on_aux_click(move |e| {
                    if e.button == Some(PointerButton::Middle)
                        && let Some(p) = untrack(|| current.get())
                    {
                        p.play_pause();
                    }
                })
                .dyn_style(move |s| {
                    let c = colors();
                    if open.get() {
                        s.bg(c.primary_container).text_color(c.on_primary_container)
                    } else {
                        s
                    }
                })
                .child(equalizer(playing));
            let content = if vertical {
                content
            } else {
                content.child(
                    txt(BAR_TEXT, move || {
                        current.with(|p| {
                            p.as_ref()
                                .map(|p| p.title.chars().take(40).collect())
                                .unwrap_or_default()
                        })
                    })
                    .min_w(0.0)
                    .overflow_hidden(),
                )
            };
            pressable(content, |c| c.on_surface)
        },
    )
    .enter(|s| s.opacity(0.0_f32).scale(0.6_f32))
    .exit(|s| s.opacity(0.0_f32).scale(0.6_f32))
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
        .gap(2.0_f32)
        .h(13.0)
        .flex_none();
    for i in 0..4 {
        row = row.child(div().w(2.5).rounded(1.25_f32).dyn_style(move |s| {
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

const TRAY_ICON: f32 = 24.0;

/// The icons of the tray.
fn tray_island(at: BarPlacement) -> Show {
    show(
        || !tray().items().is_empty(),
        move || {
            let content = if at.is_vertical() {
                // Center the end icon's circle in the island's semicircle.
                bar_island(at).py((VERTICAL_ISLAND - TRAY_ICON) / 2.0)
            } else {
                bar_island(at).px(5.0_f32)
            };
            content.gap(2.0_f32).child(
                for_each(
                    || tray().items(),
                    |item| item.key(),
                    move |item| tray_icon(item, at),
                )
                .enter(|s| s.opacity(0.0_f32).scale(0.4_f32))
                .exit(|s| s.opacity(0.0_f32).scale(0.4_f32))
                .transition(overshoot(300)),
            )
        },
    )
    .enter(|s| s.opacity(0.0_f32).scale(0.8_f32))
    .exit(|s| s.opacity(0.0_f32).scale(0.8_f32))
    .transition(overshoot(300))
}

fn tray_icon(item: TrayItem, at: BarPlacement) -> Div {
    let open = signal(false);
    let key = item.key();
    let now = move || tray().items().into_iter().find(|i| i.key() == key);
    let shown = now.clone();
    let icon = memo(move || now().map(|item| item.icon));
    // A symbolic icon is a picture made in a color: in the one the theme is going to, made again
    // when that changes rather than at each step of the way there.
    let symbolic_color = memo(|| hex_of(target_scheme().on_surface));
    let clicked = item.clone();
    pressable(
        div()
            .size(TRAY_ICON)
            .rounded(TRAY_ICON / 2.0)
            .flex()
            .items_center()
            .justify_center()
            .child(dyn_view(move || {
                let icon = icon.get();
                let picture = icon
                    .as_ref()
                    .and_then(|icon| tray_icon_image(icon, 48, symbolic_color.get()));
                if let Some(picture) = picture {
                    return img(picture)
                        .size(16.0)
                        .object_fit(ObjectFit::Contain)
                        .into_view();
                }
                match icon.and_then(|icon| icon.best(48)) {
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
            .child(tray_menu(&item, open, at)),
        |c| c.on_surface,
    )
}

/// The item's menu, in the theme's menus: the separators part it into groups. It follows the item's
/// menu as that changes.
fn tray_menu(item: &TrayItem, open: Signal<bool>, at: BarPlacement) -> View {
    let item = item.clone();
    effect({
        let item = item.clone();
        move || {
            if open.get() {
                // Programs fill some menus only when told they are about to be shown.
                let _ = item.menu_about_to_show(0);
            }
        }
    });
    let placement = match at.side() {
        PanelSide::Below => Placement::BelowCenter,
        PanelSide::Left => Placement::RightStart,
        PanelSide::Right => Placement::LeftStart,
    };
    dyn_view(move || {
        styled_menu(
            move || open.get(),
            move || open.set(false),
            tray_groups(&item, &item.menu()),
            MenuStyle::new().placement(placement),
        )
    })
}

fn tray_groups(item: &TrayItem, list: &[TrayMenuItem]) -> Vec<Vec<MenuItem>> {
    let mut groups = vec![Vec::new()];
    for m in list.iter().filter(|m| m.visible) {
        if m.separator {
            groups.push(Vec::new());
            continue;
        }
        let mut row = menu_item(m.label.clone());
        if !m.children.is_empty() {
            row = row.submenu(tray_groups(item, &m.children));
        } else {
            if let Some(MenuToggle::Check(on) | MenuToggle::Radio(on)) = m.toggle {
                row = row.selected_when(move || on);
            }
            let (item, id) = (item.clone(), m.id);
            row = row.on_click(move || {
                let _ = item.menu_clicked(id);
            });
        }
        if !m.enabled {
            row = row.disabled_when(|| true);
        }
        if let Some(keys) = &m.shortcut {
            row = row.trailing(keys.clone());
        }
        groups.last_mut().expect("never empty").push(row);
    }
    groups.retain(|g| !g.is_empty());
    groups
}

/// Network, Bluetooth, sound and battery at a glance; a click opens quick settings.
fn status_island(output: Output, at: BarPlacement) -> Div {
    let vertical = at.is_vertical();
    let net = network();
    let bt = bluetooth();
    let a = audio();
    let b = battery();
    let open = memo({
        let output = output.clone();
        move || is_open(&output, Panel::QuickSettings)
    });
    let color = move |c: &anti_webui_md3e::ColorScheme| c.on_surface_variant;
    let content = if vertical {
        bar_island(at).py(10.0_f32).gap(7.0_f32)
    } else {
        bar_island(at).px(12.0_f32).gap(9.0_f32)
    };
    let content = content
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
                let battery = div().flex().items_center().gap(2.0_f32);
                let battery = if vertical {
                    battery.flex_col()
                } else {
                    battery.flex_row()
                };
                battery
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
        });
    pressable(content, |c| c.on_surface)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    fn island(id: &str) -> Island {
        Island {
            id: id.into(),
            build: Rc::new(|_, _| unreachable!()),
        }
    }

    #[test]
    fn the_order_of_the_calls_does_not_matter() {
        let bar = Bar::new()
            .right([island("c")])
            .left([island("a")])
            .center([island("b")]);

        assert_eq!(bar.left[0].id, "a");
        assert_eq!(bar.center[0].id, "b");
        assert_eq!(bar.right[0].id, "c");
    }
}
