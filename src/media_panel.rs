//! What is playing, as a panel under the bar's island of it: the cover behind everything, the
//! title, a wavy seek bar that can be clicked or dragged, the times, and the buttons.

use crate::panels::close;
use crate::theme::{TextStyle, colors, elevation};
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui_md3e::components::{sized_icon, wavy_progress_eased};
use anti_webui_md3e::icons;
use anti_webui_md3e::motion::fast_spatial;
use std::cell::Cell;
use std::time::Duration;

/// The panel's, with the card of what is playing inside it.
const WIDTH: f32 = 392.0;
/// How far the panel keeps from the sides of the output.
const EDGE: f32 = 14.0;
/// How tall the place that takes the pointer around the seek bar is.
const SEEK_H: f32 = 28.0;

thread_local! {
    /// Where the middle of the bar's island was when it was clicked, from the left of the output.
    static ANCHOR: Cell<Option<f32>> = const { Cell::new(None) };
}

/// Says where the island is, for the panel to open under it.
pub fn anchor(center: f32) {
    ANCHOR.with(|a| a.set(Some(center)));
}

/// `1:05`, `1:02:03`.
fn clock(d: Duration) -> String {
    let s = d.as_secs();
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// The cover's file, when it is one.
fn art_of(p: &Player) -> Option<String> {
    p.art_url
        .as_ref()
        .and_then(|u| u.strip_prefix("file://").map(str::to_owned))
}

pub fn panel(output: Output) -> Div {
    let m = media();
    let current = memo(move || m.current());
    // When it stops being there, so does this.
    effect(move || {
        if current.with(Option::is_none) {
            close();
        }
    });
    let art = memo(move || current.with(|p| p.as_ref().and_then(art_of)));
    let playing = memo(move || {
        current.with(|p| {
            p.as_ref()
                .is_some_and(|p| p.status == PlaybackStatus::Playing)
        })
    });
    let has_art = move || art.with(Option::is_some);
    let fg = move |c: &anti_webui_md3e::ColorScheme| {
        if has_art() {
            rgb(0xffffff)
        } else {
            c.on_secondary_container
        }
    };
    let width = output.logical_size.width as f32;
    let center = ANCHOR.with(Cell::get).unwrap_or(width - EDGE - WIDTH / 2.0);
    let left = (center - WIDTH / 2.0).clamp(EDGE, (width - WIDTH - EDGE).max(EDGE));

    // The panel, like the others: the theme's color around a card, which has the cover behind it.
    div()
        .ml(left)
        .w(WIDTH)
        .p(16.0_f32)
        .rounded(30.0_f32)
        .shadows(elevation(3))
        .dyn_style(|s| s.bg(colors().surface_container))
        .child(
            div()
                .relative()
                .flex()
                .flex_col()
                .gap(14.0_f32)
                .p(16.0_f32)
                .rounded(22.0_f32)
                .overflow_hidden()
                // Its own stacking context, so that the cover and its veil stay behind the rest.
                .z_index(0)
                .dyn_style(move |s| {
                    let c = colors();
                    s.bg(c.secondary_container).text_color(fg(&c))
                })
                .child(backdrop(art))
                .child(about(current, art, fg))
                .child(seek_bar(current, playing, fg))
                .child(buttons(current, playing, fg)),
        )
}

/// The cover over all of the panel, darkened so that what is on it can be read; a new one fades in
/// over the old one.
fn backdrop(art: Memo<Option<String>>) -> Div {
    div()
        .absolute()
        .inset(0.0)
        .z_index(-1)
        .pointer_events_none()
        .child(
            for_each(
                move || art.get().into_iter().collect::<Vec<_>>(),
                Clone::clone,
                |path| {
                    div()
                        .absolute()
                        .inset(0.0)
                        .bg_image(path)
                        .bg_size(BackgroundSize::Cover)
                        .child(
                            div()
                                .absolute()
                                .inset(0.0)
                                .bg(rgba(0x000000).with_alpha(0.6)),
                        )
                },
            )
            .appear()
            .enter(|s| s.opacity(0.0_f32).scale(1.1_f32))
            .exit(|s| s.opacity(0.0_f32))
            .transition(emphasized(600)),
        )
}

/// The cover, the title, who it is by, and what plays it. A new title slides in over the old one.
fn about(
    current: Memo<Option<Player>>,
    art: Memo<Option<String>>,
    fg: impl crate::ui::Role,
) -> Div {
    let title = memo(move || current.with(|p| p.as_ref().map(|p| p.title.clone())));
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(14.0_f32)
        .child(
            div()
                .size(72.0)
                .flex_none()
                .relative()
                .rounded(18.0_f32)
                .overflow_hidden()
                .dyn_style(|s| s.bg(colors().primary_container))
                .child(
                    div()
                        .absolute()
                        .inset(0.0)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(sym(icons::MUSIC_NOTE, 32.0, |c| c.on_primary_container)),
                )
                .child(
                    for_each(
                        move || art.get().into_iter().collect::<Vec<_>>(),
                        Clone::clone,
                        |path| img(path).absolute().inset(0.0).object_fit(ObjectFit::Cover),
                    )
                    .appear()
                    .enter(|s| s.opacity(0.0_f32).scale(0.8_f32))
                    .exit(|s| s.opacity(0.0_f32))
                    .transition(overshoot(500)),
                ),
        )
        .child(
            div()
                .relative()
                .flex_1()
                .min_w(0.0)
                .h(64.0)
                .overflow_hidden()
                .child(
                    for_each(
                        move || vec![title.get()],
                        Clone::clone,
                        move |_| {
                            let p = untrack(|| current.get());
                            let title = p.as_ref().map(|p| p.title.clone()).unwrap_or_default();
                            let by = p.as_ref().map(|p| p.artists.join(", ")).unwrap_or_default();
                            let player = p.map(|p| p.identity).unwrap_or_default();
                            let mut column = div()
                                .absolute()
                                .inset(0.0)
                                .flex()
                                .flex_col()
                                .justify_center()
                                .child(
                                    static_txt(TextStyle::TITLE_MEDIUM, title).overflow_hidden(),
                                );
                            if !by.is_empty() {
                                column = column.child(
                                    static_txt(TextStyle::BODY_MEDIUM, by)
                                        .overflow_hidden()
                                        .opacity(0.85_f32),
                                );
                            }
                            column.child(
                                static_txt(TextStyle::LABEL_SMALL, player)
                                    .overflow_hidden()
                                    .opacity(0.65_f32),
                            )
                        },
                    )
                    .enter(|s| s.opacity(0.0_f32).translate_x(40.0_f32))
                    .exit(|s| s.opacity(0.0_f32).translate_x(-40.0_f32))
                    .transition(emphasized(450)),
                )
                .dyn_style(move |s| s.text_color(fg(&colors()))),
        )
}

/// The wavy line of how far it has played, with a handle that comes under the pointer. A click goes
/// there; dragging shows where it would go and goes there when let go.
fn seek_bar(current: Memo<Option<Player>>, playing: Memo<bool>, fg: impl crate::ui::Role) -> Div {
    let can_seek = memo(move || {
        current.with(|p| {
            p.as_ref()
                .is_some_and(|p| p.can_seek && p.length.is_some_and(|l| !l.is_zero()))
        })
    });
    // Where it is dragged to, and for a moment after it is let go, until the player has gone there.
    let held: Signal<Option<f32>> = signal(None);
    let pending: Cell<Option<Timeout>> = Cell::new(None);
    // Smooth while it plays, a little under the frames the wave asks for anyway.
    let tick = clock_at(30, move || playing.get());
    let played = move || {
        tick.get();
        local_time(Every::Second);
        current.with(|p| match p.as_ref() {
            Some(p) => match p.length {
                Some(len) if !len.is_zero() => {
                    (p.position().as_secs_f32() / len.as_secs_f32()).clamp(0.0, 1.0)
                }
                _ => 0.0,
            },
            None => 0.0,
        })
    };
    let progress = move || held.get().unwrap_or_else(played);
    let length = move || {
        current
            .with(|p| p.as_ref().and_then(|p| p.length))
            .unwrap_or_default()
    };
    let go = std::rc::Rc::new(move |ratio: f32| {
        held.set(Some(ratio));
        if let Some(p) = untrack(|| current.get())
            && let Some(len) = p.length
        {
            drop(p.set_position(len.mul_f32(ratio)));
        }
        if let Some(t) = pending.take() {
            t.cancel();
        }
        pending.set(Some(set_timeout(Duration::from_millis(700), move || {
            held.set(None)
        })));
    });
    let go_too = go.clone();
    let ratio_of = |local_x: f32, width: i32| (local_x / width.max(1) as f32).clamp(0.0, 1.0);
    let hovered = signal(false);
    let dragging = signal(false);
    let handle = follow(
        move || {
            if dragging.get() {
                1.0f32
            } else if hovered.get() && can_seek.get() {
                0.75
            } else {
                0.0
            }
        },
        fast_spatial(),
    );
    let width = signal(0.0f32);

    let bar = div()
        .relative()
        .h(SEEK_H)
        .flex()
        .items_center()
        .on_resized(move |w, _| width.set(w))
        .child(wavy_progress_eased(progress, Transition::ms(120).ease_out()).w_full())
        // The handle, a short upright bar at the end of the line.
        .child(
            div()
                .absolute()
                .top(0.0)
                .w(6.0)
                .rounded(3.0_f32)
                .pointer_events_none()
                .dyn_style(move |s| {
                    let t = handle.get().max(0.0);
                    let h = 8.0 + 16.0 * t;
                    s.left(progress() * width.get() - 3.0)
                        .top((SEEK_H - h) / 2.0)
                        .h(h)
                        .bg(colors().primary)
                        .opacity(t.min(1.0))
                }),
        )
        .on_hover(move |h| hovered.set(h))
        .on_click(move |e| {
            if untrack(|| can_seek.get()) {
                go(ratio_of(e.local.x, e.rect.width));
            }
        })
        .on_grab(move |e| {
            if !untrack(|| can_seek.get()) {
                return;
            }
            let ratio = ratio_of(e.local.x, e.rect.width);
            match e.phase {
                GrabPhase::Start | GrabPhase::Move => {
                    dragging.set(true);
                    held.set(Some(ratio));
                }
                GrabPhase::End => {
                    dragging.set(false);
                    go_too(ratio);
                }
                GrabPhase::Cancel => {
                    dragging.set(false);
                    held.set(None);
                }
            }
        })
        .dyn_style(move |s| {
            if can_seek.get() {
                s.cursor(CursorIcon::Pointer)
            } else {
                s
            }
        });

    let times = div()
        .flex()
        .flex_row()
        .justify_between()
        .px(2.0_f32)
        .child(txt(TextStyle::LABEL_SMALL, move || {
            clock(length().mul_f32(progress()))
        }))
        .child(txt(TextStyle::LABEL_SMALL, move || clock(length())))
        .opacity(0.8_f32)
        .dyn_style(move |s| s.text_color(fg(&colors())));

    div().flex().flex_col().gap(2.0_f32).child(bar).child(times)
}

/// Back, play or pause, and forward. The middle one is a big rounded square that goes round while
/// it plays.
fn buttons(current: Memo<Option<Player>>, playing: Memo<bool>, fg: impl crate::ui::Role) -> Div {
    let morph = follow(
        move || if playing.get() { 1.0 } else { 0.0 },
        fast_spatial(),
    );
    let pressed = signal(false);
    let squash = follow(
        move || if pressed.get() { 0.9 } else { 1.0 },
        fast_spatial(),
    );
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(20.0_f32)
        .child(icon_btn(icons::SKIP_PREVIOUS, 44.0, fg).on_click(move |_| {
            if let Some(p) = current.get() {
                drop(p.previous());
            }
        }))
        .child(pressable(
            div()
                .w(72.0)
                .h(56.0)
                .flex()
                .items_center()
                .justify_center()
                .flex_none()
                .on_active(move |a| pressed.set(a))
                .on_click(move |_| {
                    if let Some(p) = current.get() {
                        drop(p.play_pause());
                    }
                })
                .dyn_style(move |s| {
                    let c = colors();
                    let t = morph.get();
                    s.rounded(16.0 + 12.0 * t)
                        .bg(c.primary)
                        .text_color(c.on_primary)
                        .scale(squash.get())
                })
                .child(dyn_view(move || {
                    sized_icon(
                        if playing.get() {
                            icons::PAUSE
                        } else {
                            icons::PLAY_ARROW
                        },
                        28.0,
                    )
                })),
            |c| c.on_primary,
        ))
        .child(icon_btn(icons::SKIP_NEXT, 44.0, fg).on_click(move |_| {
            if let Some(p) = current.get() {
                drop(p.next());
            }
        }))
}
