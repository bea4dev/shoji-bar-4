//! The dock along the bottom: a button for the launcher, the pinned apps and the other apps with
//! windows open. The icons near the pointer grow, an app that is starting bounces until its window
//! shows, and a dot under an app counts its windows.

use crate::appinfo::{app_by_id, app_for, app_icon, bare, icon_file};
use crate::panels::{Panel, is_open, toggle};
use crate::settings::settings;
use crate::theme::{TextStyle, colors, elevation};
use crate::ui::*;
use anti_webui::core::geometry::RectI;
use anti_webui::prelude::*;
use anti_webui::shell::Toplevel;
use anti_webui::shell::prelude::*;
use anti_webui_md3e::components::{MenuItem, MenuStyle, menu_item, styled_menu};
use anti_webui_md3e::icons;
use anti_webui_md3e::motion::{default_effects, default_spatial, fast_spatial};
use std::time::{Duration, Instant};

/// The square an icon has, unmagnified.
const CELL: f32 = 56.0;
const ICON: f32 = 44.0;
/// How much bigger the icon under the pointer gets, and how far the growth reaches.
const MAGNIFY: f32 = 0.5;
const REACH: f32 = 1.6 * CELL;
const PAD: f32 = 8.0;
/// How far an icon at rest stands above the bottom of its cell: in its middle.
const LIFT: f32 = (CELL - ICON) / 2.0;
const MARGIN: f32 = 8.0;
/// The body of the dock; what grows reaches above it, into the rest of the layer.
const BODY: f32 = CELL + 2.0 * PAD;
const LAYER: i32 = 150;

pub fn open(output: Output) {
    let region = signal(InputRegion::Empty);
    let layer = open_layer(
        LayerOptions::new()
            .anchor(Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT)
            .height(LAYER)
            .exclusive(Exclusive::Fixed((BODY + MARGIN) as i32))
            .output(&output)
            .namespace("shoji-bar-4-dock")
            .input_region(InputRegion::Empty),
        move || view(output, region),
    );
    effect(move || layer.set_input_region(region.get()));
}

/// Something the dock shows: an installed app, or windows of something that isn't one.
#[derive(Clone)]
struct Entry {
    key: String,
    app: Option<Application>,
    /// The `app_id` of its windows, when it isn't an installed app.
    app_id: String,
    pinned: bool,
}

impl Entry {
    fn owns(&self, window: &Toplevel) -> bool {
        match &self.app {
            Some(app) => app_for(&window.app_id).is_some_and(|a| a.id == app.id),
            None => window.app_id == self.app_id,
        }
    }

    fn name(&self) -> String {
        self.app
            .as_ref()
            .map_or_else(|| self.app_id.clone(), |a| a.name.clone())
    }
}

fn entries() -> Vec<Entry> {
    let pinned = settings().with(|s| s.pinned.clone());
    let mut list: Vec<Entry> = pinned
        .iter()
        .filter_map(|id| app_by_id(id))
        .map(|app| Entry {
            key: app.id.clone(),
            app_id: String::new(),
            app: Some(app),
            pinned: true,
        })
        .collect();
    for window in toplevels().all() {
        let entry = match app_for(&window.app_id) {
            Some(app) => Entry {
                key: app.id.clone(),
                app_id: String::new(),
                app: Some(app),
                pinned: false,
            },
            None => Entry {
                key: format!("window:{}", window.app_id),
                app_id: window.app_id.clone(),
                app: None,
                pinned: false,
            },
        };
        if !list.iter().any(|e| e.key == entry.key) {
            list.push(entry);
        }
    }
    list
}

pub fn view(output: Output, region: Signal<InputRegion>) -> Div {
    let list = memo(move || {
        entries()
            .into_iter()
            .map(|e| (e.key.clone(), e.pinned))
            .collect::<Vec<_>>()
    });
    // Where the pointer is over the layer, while it is over the dock.
    let pointer = signal(None::<f32>);
    let last_x = signal(0.0f32);
    let strength = follow(
        move || if pointer.get().is_some() { 1.0 } else { 0.0 },
        default_spatial(),
    );
    let x = follow(move || last_x.get(), Spring::from_ratio(1400.0, 1.0));
    let menu_open = signal(None::<String>);
    let layer_size = signal((0.0f32, 0.0f32));
    let body_size = signal((0.0f32, 0.0f32));
    // Clicks go through all of the layer but the dock, unless it is busy.
    effect(move || {
        let (lw, lh) = layer_size.get();
        let (bw, bh) = body_size.get();
        let busy = pointer.get().is_some() || menu_open.get().is_some();
        region.set(if busy {
            InputRegion::Full
        } else if bw > 0.0 {
            InputRegion::Rects(vec![RectI::new(
                ((lw - bw) / 2.0).floor() as i32,
                (lh - MARGIN - bh).floor() as i32,
                bw.ceil() as i32 + 1,
                bh.ceil() as i32 + 1,
            )])
        } else {
            InputRegion::Empty
        });
    });
    // The unmagnified middle of the `i`th cell, in the layer's coordinates.
    let center_of = move |i: usize| {
        let n = list.with(Vec::len) as f32 + 1.0;
        let has_divider = list.with(|l| l.iter().any(|(_, p)| *p) && l.iter().any(|(_, p)| !*p));
        let width = n * CELL + if has_divider { 17.0 } else { 0.0 };
        let left = (layer_size.get().0 - width) / 2.0;
        let mut at = left + (i as f32 + 0.5) * CELL;
        if has_divider && i > list.with(|l| l.iter().filter(|(_, p)| *p).count()) {
            at += 17.0;
        }
        at
    };
    let growth = move |i: usize| {
        let s = strength.get();
        if s < 0.001 {
            return 1.0;
        }
        let d = (x.get() - center_of(i)) / REACH;
        1.0 + MAGNIFY * s * (-d * d).exp()
    };

    let launcher_output = output.clone();
    let items = for_each(
        move || list.get(),
        |(key, _)| key.clone(),
        move |(key, _)| {
            let index = memo({
                let key = key.clone();
                move || {
                    list.with(|l| l.iter().position(|(k, _)| *k == key))
                        .unwrap_or(0)
                        + 1
                }
            });
            item(key, move || growth(index.get()), menu_open)
        },
    )
    .enter(|s| s.opacity(0.0).scale(0.2))
    .exit(|s| s.opacity(0.0).scale(0.2))
    .transition(overshoot(420));

    let divider_at = memo(move || {
        list.with(|l| {
            let pinned = l.iter().filter(|(_, p)| *p).count();
            (pinned > 0 && pinned < l.len()).then_some(pinned)
        })
    });

    div()
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .justify_end()
        .items_center()
        .pb(MARGIN)
        .on_resized(move |w, h| layer_size.set((w, h)))
        .child(
            div()
                .flex()
                .flex_row()
                .items_end()
                .h(BODY)
                .px(PAD)
                .py(PAD)
                .rounded(BODY / 2.0)
                .shadows(elevation(3))
                .dyn_style(|s| s.bg(colors().surface_container))
                .on_resized(move |w, h| body_size.set((w, h)))
                .on_pointer_move(move |e| {
                    let scale = window_scale();
                    let px = scale.to_logical(e.position.x);
                    pointer.set(Some(px));
                    last_x.set(px);
                })
                .on_hover(move |on| {
                    if !on {
                        pointer.set(None);
                    }
                })
                .child(launcher_button(launcher_output, move || growth(0)))
                .child(div().flex().flex_row().items_end().child(items).child(show(
                    move || divider_at.get().is_some(),
                    || {
                        div()
                            .w(1.0)
                            .h(32.0)
                            .mx(8.0)
                            .mb(12.0)
                            .dyn_style(|s| s.bg(colors().outline_variant))
                    },
                ))),
        )
}

/// The cell of an app: its icon, bigger near the pointer, its name above it while hovered, and
/// its windows as dots below.
fn item(
    key: String,
    growth: impl Fn() -> f32 + Copy + 'static,
    menu_open: Signal<Option<String>>,
) -> Div {
    let lookup = {
        let key = key.clone();
        move || entries().into_iter().find(|e| e.key == key)
    };
    let windows = memo({
        let lookup = lookup.clone();
        move || {
            let Some(e) = lookup() else { return Vec::new() };
            toplevels()
                .all()
                .into_iter()
                .filter(|t| e.owns(t))
                .collect::<Vec<_>>()
        }
    });
    let focused = memo(move || windows.with(|w| w.iter().any(|t| t.activated)));
    let hovered = signal(false);
    let pressed = signal(false);
    let squash = follow(
        move || if pressed.get() { 0.86 } else { 1.0 },
        fast_spatial(),
    );
    // Bounces from the click that starts it until a window of it shows, for at most 4 seconds.
    let launching = signal(None::<Instant>);
    let count = memo(move || windows.with(Vec::len));
    let before = std::cell::Cell::new(0usize);
    effect(move || {
        let n = count.get();
        if n > before.replace(n) {
            launching.set(None);
        }
    });
    let t = clock();
    let start = Instant::now();
    let icon_path = memo({
        let lookup = lookup.clone();
        move || {
            let e = lookup()?;
            match &e.app {
                Some(app) => app_icon(app, 128),
                None => {
                    icon_file(&e.app_id, 128).or_else(|| icon_file(&e.app_id.to_lowercase(), 128))
                }
            }
        }
    });
    let name = memo({
        let lookup = lookup.clone();
        move || lookup().map(|e| e.name()).unwrap_or_default()
    });

    let click = {
        let lookup = lookup.clone();
        move |_: &PointerEvent| {
            let Some(e) = lookup() else { return };
            let open = untrack(|| windows.get());
            if open.is_empty() {
                if let Some(app) = &e.app {
                    launching.set(Some(Instant::now()));
                    app.launch();
                    set_timeout(Duration::from_secs(4), move || launching.set(None));
                }
            } else if let Some(active) = open.iter().find(|t| t.activated) {
                // Goes round its windows.
                let at = open.iter().position(|t| t.id() == active.id()).unwrap_or(0);
                if open.len() > 1 {
                    open[(at + 1) % open.len()].activate();
                } else {
                    active.set_minimized(true);
                }
            } else if let Some(last) = open.last() {
                last.activate();
            }
        }
    };

    let menu_key = key.clone();
    let shown_key = key.clone();
    let groups = {
        let lookup = lookup.clone();
        move || menu_groups(lookup(), untrack(|| windows.get()), launching)
    };
    div()
        .relative()
        .flex_none()
        .h(CELL)
        .dyn_style(move |s| s.w(CELL * growth()))
        .on_hover(move |h| hovered.set(h))
        .on_active(move |a| pressed.set(a))
        .on_click(click)
        .on_pointer_down(move |e| {
            if e.button == Some(PointerButton::Secondary) {
                menu_open.set(Some(menu_key.clone()));
            }
        })
        // The name, above it.
        .child(
            show(
                move || hovered.get() && menu_open.get().is_none(),
                move || {
                    div()
                        .absolute()
                        .left(-60.0)
                        .right(-60.0)
                        .flex()
                        .justify_center()
                        .pointer_events_none()
                        .dyn_style(move |s| s.bottom(LIFT + ICON * growth() + 10.0))
                        .child(
                            txt(TextStyle::LABEL_LARGE, move || name.get())
                                .px(12.0)
                                .py(6.0)
                                .rounded(10.0)
                                .shadows(elevation(2))
                                .dyn_style(|s| {
                                    let c = colors();
                                    s.bg(c.inverse_surface).text_color(c.inverse_on_surface)
                                }),
                        )
                },
            )
            .enter(|s| s.opacity(0.0).translate_y(8.0).scale(0.8))
            .exit(|s| s.opacity(0.0).translate_y(4.0).scale(0.9))
            .transition(overshoot(260)),
        )
        // The icon, standing on the bottom of the cell.
        .child(
            div()
                .absolute()
                .bottom(LIFT)
                .left(0.0)
                .right(0.0)
                .flex()
                .justify_center()
                .pointer_events_none()
                .transform_origin(0.5, 1.0)
                .dyn_style(move |s| {
                    let g = growth();
                    let jump = match launching.get() {
                        Some(at) => {
                            let since = (t.get() - at.duration_since(start).as_secs_f32()).max(0.0);
                            -((since * std::f32::consts::PI * 2.2).sin().abs()) * 16.0
                        }
                        None => 0.0,
                    };
                    s.h(ICON * g)
                        .translate_y(jump - 4.0 * (g - 1.0))
                        .scale(squash.get())
                })
                .child(dyn_view(move || match icon_path.get() {
                    Some(path) => img(path)
                        .object_fit(ObjectFit::Contain)
                        .dyn_style(move |s| s.size(ICON * growth()))
                        .into_view(),
                    None => div()
                        .rounded(14.0)
                        .flex()
                        .items_center()
                        .justify_center()
                        .dyn_style(move |s| {
                            let c = colors();
                            s.size(ICON * growth())
                                .bg(c.tertiary_container)
                                .text_color(c.on_tertiary_container)
                        })
                        .child(
                            TextStyle::TITLE_LARGE.apply(div()).child(
                                name.get()
                                    .chars()
                                    .next()
                                    .map(|c| c.to_uppercase().to_string())
                                    .unwrap_or_default(),
                            ),
                        )
                        .into_view(),
                })),
        )
        // Its windows, under it.
        .child(
            div()
                .absolute()
                .bottom(-4.0)
                .left(0.0)
                .right(0.0)
                .flex()
                .justify_center()
                .gap(3.0)
                .pointer_events_none()
                .child(
                    div()
                        .h(4.0)
                        .rounded(2.0)
                        .transition_of(
                            [Prop::Width, Prop::Background, Prop::Opacity],
                            overshoot(380),
                        )
                        .dyn_style(move |s| {
                            let c = colors();
                            let n = count.get();
                            let w = match (n, focused.get()) {
                                (0, _) => 4.0,
                                (_, true) => 18.0,
                                (1, _) => 6.0,
                                _ => 12.0,
                            };
                            s.w(w)
                                .opacity(if n == 0 { 0.0 } else { 1.0 })
                                .bg(if focused.get() {
                                    c.primary
                                } else {
                                    c.on_surface_variant
                                })
                        }),
                ),
        )
        .child(styled_menu(
            move || menu_open.get().as_deref() == Some(shown_key.as_str()),
            move || menu_open.set(None),
            groups(),
            MenuStyle::new().placement(Placement::AboveCenter),
        ))
}

fn menu_groups(
    entry: Option<Entry>,
    windows: Vec<Toplevel>,
    launching: Signal<Option<Instant>>,
) -> Vec<Vec<MenuItem>> {
    let Some(entry) = entry else {
        return Vec::new();
    };
    let mut groups = Vec::new();
    if let Some(app) = entry.app.clone() {
        let mut first = vec![menu_item("New window").icon(icons::ADD).on_click({
            let app = app.clone();
            move || {
                launching.set(Some(Instant::now()));
                app.launch();
                set_timeout(Duration::from_secs(4), move || launching.set(None));
            }
        })];
        for action in app.actions.clone() {
            first.push(menu_item(action.name.clone()).on_click(move || {
                action.launch();
            }));
        }
        groups.push(first);
        let id = bare(&app.id).to_owned();
        groups.push(vec![if entry.pinned {
            menu_item("Unpin").icon(icons::PUSH_PIN).on_click(move || {
                settings().update(|s| s.pinned.retain(|p| bare(p) != id));
            })
        } else {
            menu_item("Pin to dock")
                .icon(icons::PUSH_PIN)
                .on_click(move || {
                    settings().update(|s| s.pinned.push(id.clone()));
                })
        }]);
    }
    if !windows.is_empty() {
        let label = if windows.len() == 1 {
            "Close".to_owned()
        } else {
            format!("Close {} windows", windows.len())
        };
        groups.push(vec![menu_item(label).icon(icons::CLOSE).on_click(
            move || {
                for w in &windows {
                    w.close();
                }
            },
        )]);
    }
    groups
}

/// The button that opens the launcher: a square that rounds into a circle and turns its icon when
/// the pointer is near, and stays a circle while the launcher is open.
fn launcher_button(output: Output, growth: impl Fn() -> f32 + Copy + 'static) -> Div {
    let hovered = signal(false);
    let open = memo({
        let output = output.clone();
        move || is_open(&output, Panel::Launcher)
    });
    let round = follow(
        move || {
            if hovered.get() || open.get() {
                1.0
            } else {
                0.0
            }
        },
        fast_spatial(),
    );
    let turn = follow(
        move || if open.get() { 90.0 } else { 0.0 },
        default_spatial(),
    );
    let tint = follow(
        move || if open.get() { 1.0 } else { 0.0 },
        default_effects(),
    );
    div()
        .relative()
        .flex_none()
        .h(CELL)
        .dyn_style(move |s| s.w(CELL * growth()))
        .on_hover(move |h| hovered.set(h))
        .on_click(move |_| toggle(&output, Panel::Launcher))
        .child(
            div()
                .absolute()
                .bottom(LIFT)
                .left(0.0)
                .right(0.0)
                .flex()
                .justify_center()
                .items_end()
                .pointer_events_none()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .transform_origin(0.5, 1.0)
                        .dyn_style(move |s| {
                            let g = growth();
                            let size = ICON * g;
                            let c = colors();
                            let bg = c.primary_container.mix(c.primary, tint.get());
                            s.size(size)
                                .rounded(size * (0.3 + 0.2 * round.get()))
                                .translate_y(-4.0 * (g - 1.0))
                                .bg(bg)
                        })
                        .child(
                            div()
                                .dyn_style(move |s| {
                                    let c = colors();
                                    s.rotate(turn.get() + 45.0 * round.get()).text_color(
                                        c.on_primary_container.mix(c.on_primary, tint.get()),
                                    )
                                })
                                .child(anti_webui_md3e::components::sized_icon(icons::APPS, 24.0)),
                        ),
                ),
        )
}
