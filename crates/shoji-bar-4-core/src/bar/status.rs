use crate::bar::island::{bar_island};
use crate::panels::{Panel, is_open, toggle};
use crate::placement::BarPlacement;
use crate::theme::{TextStyle, colors};
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui::shell::{BatteryState, Connectivity};
use anti_webui_md3e::icons;

/// Network, Bluetooth, sound and battery at a glance; a click opens quick settings.
pub fn status_island(output: Output, at: BarPlacement) -> Div {
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
