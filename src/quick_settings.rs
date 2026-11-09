//! Quick settings: who is here and the power buttons, toggles as tiles, the volume and the
//! brightness, what is playing, and the colors of the shell.

use crate::bar::wifi_icon;
use crate::settings::{Palette, ThemeMode, settings};
use crate::theme::{SEEDS, TextStyle, colors, elevation, scheme_for, wallpaper_seed};
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui_md3e::components::{
    ButtonStyle, MaterialShape, ShapeStyle, SliderStyle, connected_button_group, filter_chip,
    sized_icon, styled_shape, styled_slider, wavy_progress,
};
use anti_webui_md3e::icons::{self, Symbol};
use anti_webui_md3e::motion::{default_effects, fast_spatial};

const WIDTH: f32 = 412.0;

pub fn panel(output: Output) -> Div {
    let _ = output;
    div()
        .w(WIDTH)
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
        .child(tiles())
        .child(sliders())
        .child(show(|| media().current().is_some(), media_card))
        .child(style_section())
}

/// A section of the panel on a card of its own.
fn section() -> Div {
    div()
        .flex()
        .flex_col()
        .gap(10.0)
        .p(14.0)
        .rounded(24.0)
        .dyn_style(|s| s.bg(colors().surface_container_high))
}

fn greeting() -> &'static str {
    match local_time(Every::Minute).hour {
        5..=10 => "Good morning",
        11..=16 => "Good afternoon",
        17..=21 => "Good evening",
        _ => "Good night",
    }
}

/// The user, the battery, and the power buttons, which open into a row of what they can do.
fn header() -> Div {
    let user = std::env::var("USER").unwrap_or_else(|_| "you".into());
    let initial = user.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
    let power_open = signal(false);
    let b = battery();
    let name = user.clone();
    div()
        .flex()
        .flex_col()
        .gap(10.0)
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(12.0)
                .px(6.0)
                .pt(4.0)
                .child(avatar(initial))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w(0.0)
                        .child(txt(TextStyle::TITLE_MEDIUM, move || format!("{}, {name}", greeting())))
                        .child(
                            txt(TextStyle::BODY_SMALL, move || {
                                let t = local_time(Every::Minute);
                                let mut s = t.format("%A, %-d %B");
                                if b.available() {
                                    s.push_str(&format!(" · {:.0}%", b.percentage()));
                                    if b.charging() {
                                        s.push_str(" charging");
                                    }
                                }
                                s
                            })
                            .dyn_style(|s| s.text_color(colors().on_surface_variant)),
                        ),
                )
                .child(
                    icon_btn(icons::SETTINGS, 40.0, |c| c.on_surface_variant).on_click(|_| {
                        let _ = std::process::Command::new("systemsettings").spawn();
                        crate::panels::close();
                    }),
                )
                .child(power_button(power_open)),
        )
        .child(
            show(move || power_open.get(), move || {
                let s = session();
                let actions: [(&'static Symbol, &str, Power); 4] = [
                    (icons::LOCK, "Lock", Power::Lock),
                    (icons::BEDTIME, "Sleep", Power::Sleep),
                    (icons::RESTART_ALT, "Restart", Power::Restart),
                    (icons::POWER_SETTINGS_NEW, "Shut down", Power::Off),
                ];
                let mut row = div().flex().flex_row().gap(8.0);
                for (i, (symbol, label, action)) in actions.into_iter().enumerate() {
                    let appear = animated(0.0f32);
                    set_timeout(std::time::Duration::from_millis(40 * i as u64), move || {
                        appear.spring_to(1.0, fast_spatial())
                    });
                    row = row.child(pressable(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(4.0)
                            .py(10.0)
                            .rounded(20.0)
                            .dyn_style(move |st| {
                                let c = colors();
                                let a = appear.get();
                                let danger = matches!(action, Power::Off | Power::Restart);
                                st.bg(if danger { c.error_container } else { c.secondary_container })
                                    .text_color(if danger { c.on_error_container } else { c.on_secondary_container })
                                    .opacity(a.clamp(0.0, 1.0))
                                    .scale(0.6 + 0.4 * a)
                            })
                            .on_click(move |_| {
                                crate::panels::close();
                                match action {
                                    Power::Lock => drop(s.lock()),
                                    Power::Sleep => drop(s.suspend()),
                                    Power::Restart => drop(s.reboot()),
                                    Power::Off => drop(s.power_off()),
                                }
                            })
                            .child(sized_icon(symbol, 22.0))
                            .child(static_txt(TextStyle::LABEL_MEDIUM, label)),
                        20.0,
                        |c| c.on_secondary_container,
                    ));
                }
                row
            })
            .enter(|s| s.opacity(0.0).h(0.0))
            .exit(|s| s.opacity(0.0).h(0.0))
            .transition(emphasized(300)),
        )
}

#[derive(Clone, Copy)]
enum Power {
    Lock,
    Sleep,
    Restart,
    Off,
}

/// The first letter of the user on a sunny shape that turns when the pointer is on it.
fn avatar(initial: String) -> Div {
    let hovered = signal(false);
    div()
        .size(46.0)
        .relative()
        .flex_none()
        .on_hover(move |h| hovered.set(h))
        .child(
            styled_shape(
                move || if hovered.get() { MaterialShape::SoftBurst } else { MaterialShape::Cookie12Sided },
                ShapeStyle::new()
                    .color(|c| c.tertiary_container)
                    .rotation(move || if hovered.get() { 60.0 } else { 0.0 }),
            )
            .absolute()
            .inset(0.0),
        )
        .child(
            TextStyle::TITLE_LARGE
                .apply(div())
                .absolute()
                .inset(0.0)
                .flex()
                .items_center()
                .justify_center()
                .dyn_style(|s| s.text_color(colors().on_tertiary_container))
                .child(initial),
        )
}

/// The power button: it turns into a close button while the actions are shown.
fn power_button(open: Signal<bool>) -> Div {
    let turn = follow(move || if open.get() { 1.0 } else { 0.0 }, fast_spatial());
    pressable(
        div()
            .size(40.0)
            .flex()
            .items_center()
            .justify_center()
            .flex_none()
            .on_click(move |_| open.update(|o| *o = !*o))
            .dyn_style(move |s| {
                let c = colors();
                let t = turn.get();
                s.rounded(20.0 - 8.0 * t)
                    .bg(c.error_container.with_alpha(t.clamp(0.0, 1.0)))
                    .rotate(90.0 * t)
            })
            .child(dyn_sym(
                move || if open.get() { icons::CLOSE } else { icons::POWER_SETTINGS_NEW },
                22.0,
                |c| c.on_surface_variant,
            )),
        20.0,
        |c| c.on_surface,
    )
}

/// A toggle as a tile: it grows rounder and takes the primary color when on.
fn tile(
    icon: impl Fn() -> &'static Symbol + 'static,
    title: &'static str,
    subtitle: impl Fn() -> String + 'static,
    on: impl Fn() -> bool + Copy + 'static,
    available: impl Fn() -> bool + Copy + 'static,
    click: impl Fn() + 'static,
) -> Div {
    let round = follow(move || if on() { 1.0 } else { 0.0 }, fast_spatial());
    let pressed = signal(false);
    let squash = follow(move || if pressed.get() { 0.95 } else { 1.0 }, fast_spatial());
    let content = move |c: &anti_webui_md3e::ColorScheme| if on() { c.on_primary } else { c.on_surface };
    pressable(
        div()
            .flex_1()
            .min_w(0.0)
            .h(68.0)
            .flex()
            .flex_row()
            .items_center()
            .gap(10.0)
            .px(12.0)
            .on_active(move |a| pressed.set(a))
            .on_click(move |_| {
                if available() {
                    click()
                }
            })
            .transition_of([Prop::Background, Prop::TextColor], Transition::ms(180).ease_out())
            .dyn_style(move |s| {
                let c = colors();
                let r = round.get();
                s.rounded(18.0 + 16.0 * r)
                    .scale(squash.get())
                    .bg(if on() { c.primary } else { c.surface_container_highest })
                    .text_color(content(&c))
                    .opacity(if available() { 1.0 } else { 0.45 })
            })
            .child(
                div()
                    .size(40.0)
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_none()
                    .transition_of([Prop::Background], Transition::ms(180).ease_out())
                    .dyn_style(move |s| {
                        let c = colors();
                        let r = round.get();
                        s.rounded(12.0 + 8.0 * r).bg(if on() {
                            c.on_primary.with_alpha(0.16)
                        } else {
                            c.secondary_container
                        })
                    })
                    .child(
                        div()
                            .transition(Transition::ms(200).ease_out())
                            .dyn_style(move |s| s.font_axis(b"FILL", if on() { 1.0 } else { 0.0 }))
                            .child(dyn_view(move || sized_icon(icon(), 22.0))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .min_w(0.0)
                    .overflow_hidden()
                    .child(static_txt(TextStyle::LABEL_LARGE, title))
                    .child(
                        txt(TextStyle::BODY_SMALL, subtitle)
                            .overflow_hidden()
                            .dyn_style(move |s| s.opacity(0.8)),
                    ),
            ),
        34.0,
        content,
    )
}

fn tiles() -> Div {
    let net = network();
    let bt = bluetooth();
    let a = audio();
    let p = power_profiles();
    let server = crate::notify::server();
    let has_server = server.is_some();
    let dnd = memo({
        let server = server.clone();
        move || server.as_ref().is_some_and(|s| s.do_not_disturb())
    });
    let profile_label = |name: &str| match name {
        "power-saver" => "Power saver".to_owned(),
        "performance" => "Performance".to_owned(),
        "balanced" => "Balanced".to_owned(),
        other => other.to_owned(),
    };
    let row = || div().flex().flex_row().gap(8.0);
    let dark = move || crate::theme::is_dark();
    div()
        .flex()
        .flex_col()
        .gap(8.0)
        .child(
            row()
                .child(tile(
                    move || match net.wifi() {
                        Some(ap) => wifi_icon(ap.strength),
                        None if net.wired().is_some_and(|w| w.connected) => icons::LAN,
                        None => icons::WIFI_OFF,
                    },
                    "Internet",
                    move || match net.wifi() {
                        Some(ap) => ap.ssid,
                        None if net.wired().is_some_and(|w| w.connected) => "Ethernet".into(),
                        None if net.wifi_enabled() => "Not connected".into(),
                        None => "Wi-Fi off".into(),
                    },
                    move || net.wifi_enabled(),
                    move || net.available(),
                    move || drop(net.set_wifi_enabled(!net.wifi_enabled())),
                ))
                .child(tile(
                    move || {
                        if bt.devices().iter().any(|d| d.connected) {
                            icons::BLUETOOTH_CONNECTED
                        } else if bt.adapter().is_some_and(|a| a.powered) {
                            icons::BLUETOOTH
                        } else {
                            icons::BLUETOOTH_DISABLED
                        }
                    },
                    "Bluetooth",
                    move || {
                        let connected: Vec<String> =
                            bt.devices().into_iter().filter(|d| d.connected).map(|d| d.name).collect();
                        match connected.len() {
                            0 if bt.adapter().is_some_and(|a| a.powered) => "On".into(),
                            0 => "Off".into(),
                            1 => connected[0].clone(),
                            n => format!("{n} devices"),
                        }
                    },
                    move || bt.adapter().is_some_and(|a| a.powered),
                    move || bt.available(),
                    move || {
                        let on = bt.adapter().is_some_and(|a| a.powered);
                        drop(bt.set_powered(!on));
                    },
                )),
        )
        .child(
            row()
                .child(tile(
                    move || if dark() { icons::DARK_MODE } else { icons::LIGHT_MODE },
                    "Dark theme",
                    move || match settings().with(|s| s.mode) {
                        ThemeMode::Auto => "By time of day".into(),
                        _ if dark() => "On".into(),
                        _ => "Off".into(),
                    },
                    dark,
                    || true,
                    move || {
                        settings().update(|s| s.mode = if dark() { ThemeMode::Light } else { ThemeMode::Dark })
                    },
                ))
                .child(tile(
                    move || if dnd.get() { icons::NOTIFICATIONS_OFF } else { icons::NOTIFICATIONS },
                    "Do not disturb",
                    move || if dnd.get() { "On".into() } else { "Off".into() },
                    move || dnd.get(),
                    move || has_server,
                    move || {
                        if let Some(s) = &server {
                            s.set_do_not_disturb(!s.do_not_disturb());
                        }
                    },
                )),
        )
        .child(
            row()
                .child(tile(
                    move || match p.active().as_str() {
                        "power-saver" => icons::ENERGY_SAVINGS_LEAF,
                        "performance" => icons::SPEED,
                        _ => icons::BOLT,
                    },
                    "Power mode",
                    move || profile_label(&p.active()),
                    move || p.active() != "balanced",
                    move || p.available(),
                    move || {
                        let all = p.profiles();
                        let at = all.iter().position(|x| *x == p.active()).unwrap_or(0);
                        if let Some(next) = all.get((at + 1) % all.len().max(1)) {
                            drop(p.set_active(next));
                        }
                    },
                ))
                .child(tile(
                    move || {
                        if a.default_source().is_some_and(|s| s.muted) {
                            icons::MIC_OFF
                        } else {
                            icons::MIC
                        }
                    },
                    "Microphone",
                    move || match a.default_source() {
                        Some(s) if s.muted => "Muted".into(),
                        Some(s) => s.description,
                        None => "None".into(),
                    },
                    move || a.default_source().is_some_and(|s| !s.muted),
                    move || a.default_source().is_some(),
                    move || {
                        if let Some(s) = a.default_source() {
                            drop(s.set_muted(!s.muted));
                        }
                    },
                )),
        )
}

/// The volume and the brightness.
fn sliders() -> Div {
    let a = audio();
    let s = session();
    let style = || SliderStyle::new().track(40.0).corner(14.0).handle_height(52.0).no_stops();
    let row = |icon: Div, slider: Div| {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(6.0)
            .child(icon)
            .child(slider.flex_1().min_w(0.0))
    };
    let volume = Binding::new(
        move || a.default_sink().map_or(0.0, |s| s.volume as f32),
        move |v| {
            if let Some(sink) = untrack(|| a.default_sink()) {
                drop(sink.set_volume(v as f64));
            }
        },
    );
    let brightness = Binding::new(
        move || s.brightness().unwrap_or(0.0) as f32,
        move |v| drop(s.set_brightness(v.max(0.02) as f64)),
    );
    section()
        .child(row(
            pressable(
                div()
                    .size(40.0)
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_none()
                    .rounded(20.0)
                    .on_click(move |_| {
                        if let Some(sink) = a.default_sink() {
                            drop(sink.set_muted(!sink.muted));
                        }
                    })
                    .child(dyn_sym(
                        move || match a.default_sink() {
                            Some(s) if s.muted || s.volume <= 0.001 => icons::VOLUME_OFF,
                            Some(s) if s.volume < 0.5 => icons::VOLUME_DOWN,
                            _ => icons::VOLUME_UP,
                        },
                        22.0,
                        |c| c.on_surface_variant,
                    )),
                20.0,
                |c| c.on_surface,
            ),
            styled_slider(volume, style()),
        ))
        .child(show(
            move || s.brightness().is_some(),
            move || {
                row(
                    div()
                        .size(40.0)
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_none()
                        .child(sym(icons::BRIGHTNESS_6, 22.0, |c| c.on_surface_variant)),
                    styled_slider(brightness.clone(), style()),
                )
            },
        ))
}

/// What is playing: its art behind it, the title, how far along it is on a wave, and the buttons.
fn media_card() -> Div {
    let m = media();
    let current = memo(move || m.current());
    let art = memo(move || {
        current.with(|p| {
            p.as_ref()
                .and_then(|p| p.art_url.clone())
                .and_then(|u| u.strip_prefix("file://").map(str::to_owned))
        })
    });
    let playing = memo(move || current.with(|p| p.as_ref().is_some_and(|p| p.status == PlaybackStatus::Playing)));
    let progress = move || {
        local_time(Every::Second);
        current.with(|p| match p.as_ref() {
            Some(p) => match p.length {
                Some(len) if !len.is_zero() => (p.position().as_secs_f32() / len.as_secs_f32()).clamp(0.0, 1.0),
                _ => 0.0,
            },
            None => 0.0,
        })
    };
    let has_art = move || art.with(Option::is_some);
    let fg = move |c: &anti_webui_md3e::ColorScheme| if has_art() { rgb(0xffffff) } else { c.on_secondary_container };
    let morph = follow(move || if playing.get() { 1.0 } else { 0.0 }, fast_spatial());
    div()
        .relative()
        .flex()
        .flex_col()
        .gap(8.0)
        .p(16.0)
        .rounded(24.0)
        .overflow_hidden()
        .dyn_style(move |s| {
            let c = colors();
            let s = s.bg(c.secondary_container).text_color(fg(&c));
            match art.get() {
                Some(path) => s.bg_image(path).bg_size(BackgroundSize::Cover),
                None => s,
            }
        })
        // Keeps the text readable on any art.
        .child(show(has_art, || {
            div()
                .absolute()
                .inset(0.0)
                .pointer_events_none()
                .bg(rgba(0x000000).with_alpha(0.5))
        }))
        .child(
            div()
                .relative()
                .flex()
                .flex_row()
                .items_center()
                .gap(12.0)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w(0.0)
                        .child(
                            txt(TextStyle::TITLE_MEDIUM, move || {
                                current.with(|p| p.as_ref().map(|p| p.title.clone()).unwrap_or_default())
                            })
                            .overflow_hidden(),
                        )
                        .child(
                            txt(TextStyle::BODY_MEDIUM, move || {
                                current.with(|p| {
                                    p.as_ref()
                                        .map(|p| {
                                            let who = p.artists.join(", ");
                                            if who.is_empty() { p.identity.clone() } else { who }
                                        })
                                        .unwrap_or_default()
                                })
                            })
                            .overflow_hidden()
                            .opacity(0.8),
                        ),
                )
                .child(
                    icon_btn(icons::SKIP_PREVIOUS, 40.0, fg).on_click(move |_| {
                        if let Some(p) = current.get() {
                            drop(p.previous());
                        }
                    }),
                )
                .child(
                    pressable(
                        div()
                            .size(52.0)
                            .flex()
                            .items_center()
                            .justify_center()
                            .flex_none()
                            .on_click(move |_| {
                                if let Some(p) = current.get() {
                                    drop(p.play_pause());
                                }
                            })
                            .dyn_style(move |s| {
                                let c = colors();
                                let t = morph.get();
                                s.rounded(14.0 + 12.0 * t).bg(c.primary).text_color(c.on_primary)
                            })
                            .child(dyn_view(move || {
                                sized_icon(if playing.get() { icons::PAUSE } else { icons::PLAY_ARROW }, 26.0)
                            })),
                        26.0,
                        |c| c.on_primary,
                    ),
                )
                .child(
                    icon_btn(icons::SKIP_NEXT, 40.0, fg).on_click(move |_| {
                        if let Some(p) = current.get() {
                            drop(p.next());
                        }
                    }),
                ),
        )
        .child(
            div()
                .relative()
                .child(wavy_progress(progress).w_full())
                .dyn_style(move |s| s),
        )
}

/// The colors: seeds to choose from, how colorful a scheme is made from them, and light or dark.
fn style_section() -> Div {
    let mut swatches = div().flex().flex_row().justify_between();
    for (seed, name) in SEEDS {
        swatches = swatches.child(swatch(Some(seed), name));
    }
    let from_wallpaper = show(
        || wallpaper_seed().is_some(),
        || swatch(None, "Wallpaper"),
    );
    let chip = |p: Palette| {
        filter_chip(
            p.label(),
            Binding::new(
                move || settings().with(|s| s.palette == p),
                move |on| {
                    if on {
                        settings().update(|s| s.palette = p)
                    }
                },
            ),
        )
    };
    let mut chips_a = div().flex().flex_row().gap(6.0);
    let mut chips_b = div().flex().flex_row().gap(6.0);
    for (i, p) in Palette::ALL.into_iter().enumerate() {
        if i < 3 {
            chips_a = chips_a.child(chip(p));
        } else {
            chips_b = chips_b.child(chip(p));
        }
    }
    let modes = [ThemeMode::Light, ThemeMode::Auto, ThemeMode::Dark];
    section()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(8.0)
                .child(sym(icons::PALETTE, 20.0, |c| c.primary))
                .child(static_txt(TextStyle::TITLE_SMALL, "Style & color")),
        )
        .child(swatches.child(from_wallpaper))
        .child(div().flex().flex_col().gap(6.0).child(chips_a).child(chips_b))
        .child(connected_button_group(
            ["Light", "Auto", "Dark"],
            ButtonStyle::tonal(),
            move |i| settings().with(|s| s.mode == modes[i]),
            move |i| settings().update(|s| s.mode = modes[i]),
        ))
}

/// A seed as a circle of its primary color with its two other accents in the middle; the chosen
/// one blooms into a cookie and shows a check. `None` is the wallpaper's.
fn swatch(seed: Option<u32>, name: &'static str) -> Div {
    let chosen = memo(move || {
        settings().with(|s| match seed {
            Some(seed) => !s.seed_from_wallpaper && s.seed == seed,
            None => s.seed_from_wallpaper,
        })
    });
    let hovered = signal(false);
    let lift = follow(move || if hovered.get() { 1.0 } else { 0.0 }, fast_spatial());
    let scheme = memo(move || {
        let s = settings().get();
        let seed = seed.or_else(wallpaper_seed).unwrap_or(s.seed);
        scheme_for(seed, s.palette, crate::theme::is_dark())
    });
    let check = follow(move || if chosen.get() { 1.0 } else { 0.0 }, default_effects());
    div()
        .size(38.0)
        .relative()
        .flex_none()
        .on_hover(move |h| hovered.set(h))
        .on_click(move |_| {
            settings().update(|s| match seed {
                Some(seed) => {
                    s.seed = seed;
                    s.seed_from_wallpaper = false;
                }
                None => s.seed_from_wallpaper = true,
            })
        })
        .access(move |a| a.label(name))
        .dyn_style(move |s| s.scale(1.0 + 0.1 * lift.get()))
        .child(
            styled_shape(
                move || if chosen.get() { MaterialShape::Cookie9Sided } else { MaterialShape::Circle },
                ShapeStyle::new()
                    .color(move |_| scheme.get().primary)
                    .spring(fast_spatial())
                    .rotation(move || if chosen.get() { 40.0 } else { 0.0 }),
            )
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
                .pointer_events_none()
                .child(
                    div()
                        .size(18.0)
                        .rounded(9.0)
                        .overflow_hidden()
                        .flex()
                        .flex_row()
                        .dyn_style(move |s| s.opacity(1.0 - check.get()).scale(1.0 - 0.5 * check.get()))
                        .child(div().w(9.0).h_full().dyn_style(move |s| s.bg(scheme.get().secondary_container)))
                        .child(div().w(9.0).h_full().dyn_style(move |s| s.bg(scheme.get().tertiary))),
                )
                .child(
                    div()
                        .absolute()
                        .inset(0.0)
                        .flex()
                        .items_center()
                        .justify_center()
                        .dyn_style(move |s| {
                            let t = check.get();
                            s.text_color(scheme.get().on_primary).opacity(t).scale(0.4 + 0.6 * t)
                        })
                        .child(sized_icon(if seed.is_some() { icons::CHECK } else { icons::WALLPAPER }, 20.0)),
                ),
        )
}
