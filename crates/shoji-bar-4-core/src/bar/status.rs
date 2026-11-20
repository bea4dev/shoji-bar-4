use std::rc::Rc;

use crate::bar::island::Island;
use crate::context::Part;
use crate::layout::{by_placement, column, row};
use crate::panels::{Panel, is_open, toggle};
use crate::theme::TextStyle;
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui::shell::{BatteryState, Connectivity};
use anti_webui_md3e::icons;

/// How the network is connected: Wi-Fi with its strength, the cable, or why there is none.
pub fn wifi() -> Part {
    Rc::new(|_cx| {
        let net = network();
        dyn_sym(
            move || match net.wifi() {
                Some(ap) => wifi_icon(ap.strength),
                None if net.wired().is_some_and(|w| w.connected) => icons::LAN,
                None if net.connectivity() == Connectivity::None => icons::SIGNAL_WIFI_OFF,
                None => icons::WIFI_OFF,
            },
            16.0,
            |c: &anti_webui_md3e::ColorScheme| c.on_surface_variant,
        )
        .into_view()
    })
}

/// Bluetooth, while the adapter is on: it shows whether a device is connected.
pub fn bluetooth() -> Part {
    Rc::new(|_cx| {
        let bt = anti_webui::shell::bluetooth();
        show(
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
                    |c: &anti_webui_md3e::ColorScheme| c.on_surface_variant,
                )
            },
        )
        .into_view()
    })
}

/// The volume of the default output: muted, low or high.
pub fn volume() -> Part {
    Rc::new(|_cx| {
        let a = audio();
        dyn_sym(
            move || match a.default_sink() {
                Some(s) if s.muted || s.volume <= 0.001 => icons::VOLUME_OFF,
                Some(s) if s.volume < 0.5 => icons::VOLUME_DOWN,
                _ => icons::VOLUME_UP,
            },
            16.0,
            |c: &anti_webui_md3e::ColorScheme| c.on_surface_variant,
        )
        .into_view()
    })
}

/// The battery, while there is one: its icon and its percentage, stacked on a vertical bar.
pub fn battery() -> Part {
    Rc::new(|cx| {
        let vertical = cx.placement.is_vertical();
        let b = anti_webui::shell::battery();
        show(
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
                        |c: &anti_webui_md3e::ColorScheme| c.on_surface_variant,
                    ))
                    .child(txt(TextStyle::LABEL_MEDIUM, move || {
                        format!("{:.0}%", b.percentage())
                    }))
            },
        )
        .into_view()
    })
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

/// Network, Bluetooth, sound and battery at a glance; a click opens quick settings.
pub fn island() -> Island {
    let parts = [wifi(), bluetooth(), volume(), battery()];
    Island::new(
        "status",
        by_placement(row(9.0, parts.clone()), column(7.0, parts)),
    )
    .padding(12.0, 10.0)
    .active(|cx| is_open(&cx.output, Panel::QuickSettings))
    .on_click(|cx| toggle(&cx.output, Panel::QuickSettings))
}
