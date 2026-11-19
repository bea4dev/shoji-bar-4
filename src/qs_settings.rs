//! The settings of the shell, a page of quick settings behind its gear: who is here and their
//! picture, the colors, and the clock.

use crate::qs_pages::header;
use crate::quick_settings::{Page, open_at, section};
use crate::settings::{BarPosition, Palette, ShellPadding, ThemeMode, settings};
use crate::theme::{SEEDS, TextStyle, colors, scheme_for, wallpaper_seed};
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui_md3e::components::{
    ButtonSize, ButtonStyle, MaterialShape, ShapeStyle, TextFieldStyle, connected_button_group,
    divider, filter_chip, sized_icon, styled_button, styled_shape, styled_slider, switch,
    text_field,
};
use anti_webui_md3e::icons;
use anti_webui_md3e::motion::{default_effects, fast_spatial};

pub fn settings_page() -> Div {
    div()
        .flex()
        .flex_col()
        .gap(12.0_f32)
        .child(header("Settings", div()))
        .child(settle_in(1, account_section()))
        .child(settle_in(2, style_section()))
        .child(settle_in(3, desktop_layout_section()))
        .child(settle_in(4, clock_section()))
}

/// The heading of a section: an icon and its name.
fn heading(symbol: &'static icons::Symbol, title: &'static str) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(8.0_f32)
        .child(sym(symbol, 18.0, |c| c.primary))
        .child(static_txt(TextStyle::TITLE_SMALL, title))
}

/// The user's picture, and the ways to change it.
fn account_section() -> Div {
    let chosen = memo(|| settings().with(|s| s.avatar.is_some()));
    let said = move || {
        if chosen.get() {
            "Your picture"
        } else if crate::user::picture().is_some() {
            "The desktop's picture"
        } else {
            "No picture yet"
        }
        .to_owned()
    };
    section().child(
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(14.0_f32)
            .child(crate::user::avatar(64.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w(0.0)
                    .gap(8.0_f32)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(static_txt(TextStyle::TITLE_MEDIUM, crate::user::name()))
                            .child(
                                txt(TextStyle::BODY_SMALL, said)
                                    .dyn_style(|s| s.text_color(colors().on_surface_variant)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(6.0_f32)
                            .child(
                                styled_button(
                                    "Change picture",
                                    ButtonStyle::tonal()
                                        .size(ButtonSize::ExtraSmall)
                                        .icon(icons::ADD_PHOTO_ALTERNATE),
                                )
                                // The chooser is a window, which the panel would cover; the
                                // panel comes back here once it is closed.
                                .on_click(|_| {
                                    crate::panels::close();
                                    crate::user::choose_picture(|| open_at(Page::Settings));
                                }),
                            )
                            .child(
                                show(
                                    move || chosen.get(),
                                    || {
                                        styled_button(
                                            "Remove",
                                            ButtonStyle::text().size(ButtonSize::ExtraSmall),
                                        )
                                        .on_click(|_| crate::user::forget_picture())
                                    },
                                )
                                .enter(|s| s.opacity(0.0_f32).scale(0.8_f32))
                                .exit(|s| s.opacity(0.0_f32).scale(0.8_f32))
                                .transition(overshoot(320)),
                            ),
                    ),
            ),
    )
}

fn format_edge_input(value: Option<u16>) -> String {
    value.map_or_else(String::new, |v| v.to_string())
}

fn parse_edge_input(raw: &str, label: &str) -> Result<Option<u16>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("auto") {
        return Ok(None);
    }
    trimmed
        .parse::<u16>()
        .ok()
        .filter(|&n| n <= ShellPadding::MAX_EDGE)
        .map(Some)
        .ok_or_else(|| format!("{label} must be 0–{} or auto.", ShellPadding::MAX_EDGE))
}

fn setting_switch(label: &'static str, on: Binding<bool>) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .px(4.0_f32)
        .child(static_txt(TextStyle::BODY_MEDIUM, label).flex_1())
        .child(switch(on))
}

/// Bar edge, optional desktop components, and shell padding/gaps alignment.
fn desktop_layout_section() -> Div {
    let positions = BarPosition::ALL;
    let overrides_open = signal(settings().with_untracked(|s| s.padding.has_overrides()));
    let top_text = signal(String::new());
    let right_text = signal(String::new());
    let bottom_text = signal(String::new());
    let left_text = signal(String::new());
    let error = signal(String::new());

    let saved_edges = memo(|| {
        settings().with(|s| {
            let p = s.padding.normalized();
            (p.top, p.right, p.bottom, p.left)
        })
    });
    effect(move || {
        let (top, right, bottom, left) = saved_edges.get();
        top_text.set(format_edge_input(top));
        right_text.set(format_edge_input(right));
        bottom_text.set(format_edge_input(bottom));
        left_text.set(format_edge_input(left));
        error.set(String::new());
    });

    let stepper_btn = |symbol: &'static icons::Symbol, label: &'static str, delta: i32| {
        pressable(
            div()
                .size(28.0)
                .rounded(14.0_f32)
                .flex()
                .items_center()
                .justify_center()
                .flex_none()
                .cursor(CursorIcon::Pointer)
                .focusable()
                .dyn_style(|s| s.bg(colors().surface_container_highest))
                .access(move |a| a.role(anti_webui::prelude::Role::Button).label(label))
                .on_click(move |_| {
                    settings().update(|s| {
                        let cur = s.padding.gaps.min(ShellPadding::MAX_GAPS) as i32;
                        s.padding.gaps =
                            (cur + delta).clamp(0, ShellPadding::MAX_GAPS as i32) as u16;
                    });
                })
                .child(sym(symbol, 16.0, |c| c.on_surface_variant)),
            |c| c.on_surface_variant,
        )
    };

    let mut presets = div().flex().flex_row().gap(4.0_f32);
    for preset in [0u16, 8, 12, 14, 16, 24] {
        let pill = pressable(
            div()
                .flex_1()
                .h(30.0)
                .flex()
                .items_center()
                .justify_center()
                .cursor(CursorIcon::Pointer)
                .focusable()
                .transition_of(
                    [Prop::Background, Prop::TextColor],
                    Transition::ms(160).ease_out(),
                )
                .dyn_style(move |s| {
                    let c = colors();
                    let active = settings().with(|cfg| cfg.padding.gaps == preset);
                    s.rounded(if active { 15.0_f32 } else { 10.0_f32 })
                        .bg(if active {
                            c.primary
                        } else {
                            c.surface_container_highest
                        })
                        .text_color(if active {
                            c.on_primary
                        } else {
                            c.on_surface
                        })
                })
                .access(move |a| {
                    a.role(anti_webui::prelude::Role::Button)
                        .label(format!("Gaps {preset} px"))
                })
                .on_click(move |_| settings().update(|cfg| cfg.padding.gaps = preset))
                .child(static_txt(TextStyle::LABEL_MEDIUM, preset.to_string())),
            move |c| {
                if settings().with(|cfg| cfg.padding.gaps == preset) {
                    c.on_primary
                } else {
                    c.on_surface
                }
            },
        );
        presets = presets.child(pill);
    }

    let gaps_binding = Binding::new(
        || {
            settings().with(|s| {
                (s.padding.gaps.min(ShellPadding::MAX_GAPS) as f32)
                    / (ShellPadding::MAX_GAPS as f32)
            })
        },
        |v| {
            let next = (v * ShellPadding::MAX_GAPS as f32)
                .round()
                .clamp(0.0, ShellPadding::MAX_GAPS as f32) as u16;
            settings().update(|s| s.padding.gaps = next);
        },
    );

    let apply_edges = move || {
        let parsed = (|| {
            Ok::<_, String>((
                parse_edge_input(&top_text.get_untracked(), "Top")?,
                parse_edge_input(&right_text.get_untracked(), "Right")?,
                parse_edge_input(&bottom_text.get_untracked(), "Bottom")?,
                parse_edge_input(&left_text.get_untracked(), "Left")?,
            ))
        })();
        match parsed {
            Ok((top, right, bottom, left)) => {
                settings().update(|s| {
                    let mut next = s.padding.normalized();
                    next.top = top;
                    next.right = right;
                    next.bottom = bottom;
                    next.left = left;
                    s.padding = next;
                });
                top_text.set(format_edge_input(top));
                right_text.set(format_edge_input(right));
                bottom_text.set(format_edge_input(bottom));
                left_text.set(format_edge_input(left));
                error.set(String::new());
            }
            Err(message) => error.set(message),
        }
    };

    let clear_edges = move |_: &PointerEvent| {
        settings().update(|s| {
            let mut next = s.padding.normalized();
            next.top = None;
            next.right = None;
            next.bottom = None;
            next.left = None;
            s.padding = next;
        });
        top_text.set(String::new());
        right_text.set(String::new());
        bottom_text.set(String::new());
        left_text.set(String::new());
        error.set(String::new());
    };

    let edge_field = move |label: &'static str, value: Signal<String>| {
        text_field(
            value,
            TextFieldStyle::new()
                .label(label)
                .placeholder("auto")
                .on_submit(move |_| apply_edges()),
        )
        .flex_1()
        .min_w(0.0)
    };

    let toggle_overrides = move |icon| {
        styled_button(
            "Edge overrides",
            ButtonStyle::text().size(ButtonSize::ExtraSmall).icon(icon),
        )
        .on_click(move |_| overrides_open.update(|open| *open = !*open))
    };

    section()
        .child(heading(icons::SPACE_DASHBOARD, "Desktop & layout"))
        .child(static_txt(TextStyle::LABEL_LARGE, "Bar position"))
        .child(connected_button_group(
            positions.map(|p| p.label()),
            ButtonStyle::tonal(),
            move |i| settings().with(|s| s.bar_position == positions[i]),
            move |i| settings().update(|s| s.bar_position = positions[i]),
        ))
        .child(setting_switch(
            "Launcher",
            Binding::new(
                || settings().with(|s| s.launcher_enabled),
                |on| settings().update(|s| s.launcher_enabled = on),
            ),
        ))
        .child(setting_switch(
            "Dock",
            Binding::new(
                || settings().with(|s| s.dock_enabled),
                |on| settings().update(|s| s.dock_enabled = on),
            ),
        ))
        .child(divider())
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(4.0_f32)
                .px(2.0_f32)
                .child(static_txt(TextStyle::LABEL_LARGE, "Gaps").flex_1())
                .child(show(
                    || settings().with(|s| s.padding.normalized() != ShellPadding::DEFAULT),
                    || {
                        pressable(
                            div()
                                .h(24.0)
                                .px(10.0_f32)
                                .mr(2.0_f32)
                                .rounded(12.0_f32)
                                .flex()
                                .items_center()
                                .justify_center()
                                .flex_none()
                                .cursor(CursorIcon::Pointer)
                                .focusable()
                                .dyn_style(|s| {
                                    let c = colors();
                                    s.bg(c.surface_container_highest).text_color(c.primary)
                                })
                                .access(|a| {
                                    a.role(anti_webui::prelude::Role::Button)
                                        .label("Reset padding and gaps")
                                })
                                .on_click(|_| {
                                    settings().update(|s| s.padding = ShellPadding::DEFAULT)
                                })
                                .child(static_txt(TextStyle::LABEL_MEDIUM, "Reset")),
                            |c| c.primary,
                        )
                    },
                ))
                .child(stepper_btn(icons::CHEVRON_LEFT, "Decrease gaps", -1))
                .child(
                    txt(TextStyle::LABEL_LARGE, || {
                        format!(
                            "{} px",
                            settings().with(|s| s.padding.gaps.min(ShellPadding::MAX_GAPS))
                        )
                    })
                    .min_w(44.0)
                    .text_align(TextAlign::Center)
                    .dyn_style(|s| s.text_color(colors().primary)),
                )
                .child(stepper_btn(icons::CHEVRON_RIGHT, "Increase gaps", 1)),
        )
        .child(
            TextStyle::BODY_SMALL
                .apply(div().child(text(|| {
                    let p = settings().with(|s| s.padding.normalized());
                    if !p.has_overrides() {
                        format!("Gaps {} px · all edges follow gaps", p.gaps)
                    } else {
                        let edge = |name: &str, v: Option<u16>| match v {
                            Some(n) => format!("{name} {n}"),
                            None => format!("{name} auto ({})", p.gaps),
                        };
                        format!(
                            "Gaps {} px · {} · {} · {} · {}",
                            p.gaps,
                            edge("Top", p.top),
                            edge("Right", p.right),
                            edge("Bottom", p.bottom),
                            edge("Left", p.left),
                        )
                    }
                })))
                .dyn_style(|s| s.text_color(colors().on_surface_variant)),
        )
        .child(presets)
        .child(
            styled_slider(gaps_binding, crate::quick_settings::slider_style())
                .w_full()
                .my(-4.0),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .min_h(36.0)
                .child(show(
                    move || !overrides_open.get(),
                    move || toggle_overrides(icons::EXPAND_MORE),
                ))
                .child(show(
                    move || overrides_open.get(),
                    move || toggle_overrides(icons::EXPAND_LESS),
                )),
        )
        .child(show(
            move || overrides_open.get(),
            move || {
                div()
                    .flex()
                    .flex_col()
                    .gap(10.0_f32)
                    .child(
                        TextStyle::BODY_SMALL
                            .apply(div().child(
                                "Blank or 'auto' follows gaps. Enter 0–512 px to match Niri struts.",
                            ))
                            .dyn_style(|s| s.text_color(colors().on_surface_variant)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(8.0_f32)
                            .child(edge_field("Top", top_text))
                            .child(edge_field("Right", right_text)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(8.0_f32)
                            .child(edge_field("Bottom", bottom_text))
                            .child(edge_field("Left", left_text)),
                    )
                    .child(show(
                        move || !error.with(String::is_empty),
                        move || {
                            TextStyle::BODY_SMALL
                                .apply(div().child(text(move || error.get())))
                                .dyn_style(|s| s.text_color(colors().error))
                        },
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_end()
                            .gap(8.0_f32)
                            .child(
                                styled_button(
                                    "Clear overrides",
                                    ButtonStyle::text().size(ButtonSize::Small),
                                )
                                .on_click(clear_edges),
                            )
                            .child(
                                styled_button(
                                    "Apply",
                                    ButtonStyle::filled().size(ButtonSize::Small),
                                )
                                .on_click(move |_| apply_edges()),
                            ),
                    )
            },
        ))
}

/// How the clock in the bar reads.
fn clock_section() -> Div {
    section()
        .child(heading(icons::SCHEDULE, "Clock"))
        .child(setting_switch(
            "24-hour time",
            Binding::new(
                || settings().with(|s| s.clock_24h),
                |on| settings().update(|s| s.clock_24h = on),
            ),
        ))
        .child(setting_switch(
            "Show seconds",
            Binding::new(
                || settings().with(|s| s.clock_seconds),
                |on| settings().update(|s| s.clock_seconds = on),
            ),
        ))
}

/// The colors: seeds to choose from, how colorful a scheme is made from them, and light or dark.
fn style_section() -> Div {
    let mut swatches = div().flex().flex_row().justify_between();
    for (seed, name) in SEEDS {
        swatches = swatches.child(swatch(Some(seed), name));
    }
    let from_wallpaper = show(|| wallpaper_seed().is_some(), || swatch(None, "Wallpaper"));
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
    let mut chips_a = div().flex().flex_row().gap(6.0_f32);
    let mut chips_b = div().flex().flex_row().gap(6.0_f32);
    for (i, p) in Palette::ALL.into_iter().enumerate() {
        if i < 3 {
            chips_a = chips_a.child(chip(p));
        } else {
            chips_b = chips_b.child(chip(p));
        }
    }
    let modes = [ThemeMode::Light, ThemeMode::Auto, ThemeMode::Dark];
    section()
        .child(heading(icons::PALETTE, "Style & color"))
        .child(wallpapers())
        .child(swatches.child(from_wallpaper))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(6.0_f32)
                .child(chips_a)
                .child(chips_b),
        )
        .child(connected_button_group(
            ["Light", "Auto", "Dark"],
            ButtonStyle::tonal(),
            move |i| settings().with(|s| s.mode == modes[i]),
            move |i| settings().update(|s| s.mode = modes[i]),
        ))
}

/// The pictures to choose a wallpaper from. Choosing one takes the colors from it too.
fn wallpapers() -> Div {
    let pictures = crate::wallpaper::pictures();
    let mut row = div().flex().flex_row().gap(8.0_f32).pb(4.0_f32).overflow_x_scroll();
    for path in pictures {
        let chosen = memo({
            let path = path.clone();
            move || settings().with(|s| s.wallpaper.as_ref() == Some(&path))
        });
        let round = follow(move || if chosen.get() { 1.0 } else { 0.0 }, fast_spatial());
        let hovered = signal(false);
        let lift = follow(
            move || if hovered.get() { 1.0 } else { 0.0 },
            fast_spatial(),
        );
        let picked = path.clone();
        row = row.child(
            div()
                .w(80.0)
                .h(50.0)
                .flex_none()
                .relative()
                .on_hover(move |h| hovered.set(h))
                .on_click(move |_| {
                    let path = picked.clone();
                    settings().update(move |s| {
                        s.wallpaper = Some(path);
                        s.seed_from_wallpaper = true;
                    })
                })
                .dyn_style(move |s| s.scale(1.0 + 0.05 * lift.get()))
                .child(
                    div()
                        .size_full()
                        .overflow_hidden()
                        .dyn_style(move |s| s.rounded(10.0 + 18.0 * round.get()))
                        .child(img(path).size_full().object_fit(ObjectFit::Cover)),
                )
                .child(
                    div()
                        .absolute()
                        .inset(0.0)
                        .pointer_events_none()
                        .border(3.0_f32)
                        .dyn_style(move |s| {
                            let t = round.get();
                            s.rounded(10.0 + 18.0 * t)
                                .border_color(colors().primary.with_alpha(t.clamp(0.0, 1.0)))
                        }),
                ),
        );
    }
    row
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
    let lift = follow(
        move || if hovered.get() { 1.0 } else { 0.0 },
        fast_spatial(),
    );
    let scheme = memo(move || {
        let s = settings().get();
        let seed = seed.or_else(wallpaper_seed).unwrap_or(s.seed);
        scheme_for(seed, s.palette, crate::theme::is_dark())
    });
    let check = follow(
        move || if chosen.get() { 1.0 } else { 0.0 },
        default_effects(),
    );
    div()
        .size(30.0)
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
                move || {
                    if chosen.get() {
                        MaterialShape::Cookie9Sided
                    } else {
                        MaterialShape::Circle
                    }
                },
                ShapeStyle::new()
                    .color(move |_| scheme.get().primary)
                    .spring(fast_spatial())
                    .rotation(move || if chosen.get() { 40.0 } else { 0.0 }),
            )
            .absolute()
            .inset(0.0)
            .size_full(),
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
                        .size(14.0)
                        .rounded(7.0_f32)
                        .overflow_hidden()
                        .flex()
                        .flex_row()
                        .dyn_style(move |s| {
                            s.opacity(1.0 - check.get()).scale(1.0 - 0.5 * check.get())
                        })
                        .child(
                            div()
                                .w(7.0)
                                .h_full()
                                .dyn_style(move |s| s.bg(scheme.get().secondary_container)),
                        )
                        .child(
                            div()
                                .w(7.0)
                                .h_full()
                                .dyn_style(move |s| s.bg(scheme.get().tertiary)),
                        ),
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
                            s.text_color(scheme.get().on_primary)
                                .opacity(t)
                                .scale(0.4 + 0.6 * t)
                        })
                        .child(sized_icon(
                            if seed.is_some() {
                                icons::CHECK
                            } else {
                                icons::WALLPAPER
                            },
                            16.0,
                        )),
                ),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Settings, ShellPadding};
    use anti_webui::core::{
        Ui,
        event::WindowEvent,
        geometry::{Scale, SizeI},
    };

    #[test]
    fn edge_input_parsing_accepts_blank_auto_and_bounds() {
        assert_eq!(parse_edge_input("", "Top"), Ok(None));
        assert_eq!(parse_edge_input("  auto  ", "Top"), Ok(None));
        assert_eq!(parse_edge_input("AUTO", "Top"), Ok(None));
        assert_eq!(parse_edge_input("0", "Top"), Ok(Some(0)));
        assert_eq!(parse_edge_input("512", "Top"), Ok(Some(512)));
        assert!(parse_edge_input("513", "Top").is_err());
        assert!(parse_edge_input("-1", "Top").is_err());
        assert!(parse_edge_input("px", "Top").is_err());
    }

    #[test]
    fn settings_page_renders_desktop_layout_card_with_overrides() {
        crate::settings::never_save();
        settings().set(Settings {
            padding: ShellPadding {
                gaps: 16,
                top: Some(8),
                right: Some(64),
                bottom: None,
                left: Some(64),
            },
            ..Settings::default()
        });
        let mut ui = Ui::new(|| div().w(396.0).p(16.0_f32).child(settings_page()));
        ui.handle_event(WindowEvent::Resized {
            size: SizeI::new(396, 1600),
            scale: Scale::from_f64(1.0),
        });
        ui.update();
        drop(ui);
        settings().set(Settings::default());
    }
}
