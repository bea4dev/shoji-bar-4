use crate::appinfo::app_id_icon;
use crate::bar::BAR_TEXT;
use crate::bar::island::{ISLAND, Island, VERTICAL_ISLAND, bar_island};
use crate::placement::BarPlacement;
use crate::theme::colors;
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui_md3e::icons;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

/// The window with the keyboard on this output.
pub fn window_island(output: Output, at: BarPlacement) -> Show {
    focused_island(
        memo(move || {
            toplevels()
                .active()
                .filter(|t| t.outputs.is_empty() || t.outputs.contains(&output.name))
                .map(|t| Focused {
                    id: t.id(),
                    title: t.title.chars().take(60).collect(),
                    app_id: t.app_id,
                })
        }),
        at,
    )
}

/// What the bar shows of the window with the keyboard.
#[derive(Clone, PartialEq, Debug)]
pub struct Focused {
    pub id: u64,
    pub title: String,
    pub app_id: String,
}

/// The icon and title of `active` on the top bar, or only its application icon on a vertical bar.
/// When another window takes the keyboard, its row comes up from
/// below as the last one goes up; when the title changes, the new one comes up the same way. The
/// island goes to the width of what comes on a spring, which keeps its speed when that changes
/// again on the way, so it follows however fast they change. What goes stays until it has gone
/// up, filling what the island has yet to give up, and a title wider than the island fades out at
/// its edge rather than being cut off.
pub fn focused_island(active: Memo<Option<Focused>>, at: BarPlacement) -> Show {
    if at.is_vertical() {
        let app_id = memo(move || active.with(|t| t.as_ref().map(|t| t.app_id.clone())));
        return show(
            move || active.with(Option::is_some),
            move || {
                bar_island(at)
                    .size(VERTICAL_ISLAND)
                    .justify_center()
                    .child(window_icon(app_id))
            },
        )
        .enter(|s| s.opacity(0.0_f32).scale(0.8_f32))
        .exit(|s| s.opacity(0.0_f32).scale(0.8_f32))
        .transition(overshoot(350));
    }
    show(
        move || active.with(Option::is_some),
        move || {
            let active_id = memo(move || active.with(|t| t.as_ref().map(|t| t.id)));
            // Each turn a window has with the keyboard gets a row of its own, even when the
            // window had one a moment ago: rows only come up and go up.
            let turn = signal(0u64);
            let rows = signal(vec![0u64]);
            let seen = Cell::new(untrack(|| active_id.get()));
            effect(move || {
                let id = active_id.get();
                if id.is_none() || id == seen.get() {
                    return;
                }
                seen.set(id);
                let gone = untrack(|| turn.get());
                rows.update(|rows| rows.push(gone + 1));
                turn.set(gone + 1);
                after_going(move || {
                    if rows.is_alive() {
                        rows.update(|rows| rows.retain(|r| *r != gone));
                    }
                });
            });
            let width = animated(0.0f32);
            let sized = Rc::new(Cell::new(false));
            bar_island(at)
                .relative()
                .px(5.0_f32)
                .min_w(0.0)
                .flex_shrink(1.0_f32)
                .overflow_hidden()
                .child(
                    div()
                        .relative()
                        .h_full()
                        .min_w(0.0)
                        .overflow_hidden()
                        .dyn_style(move |s| s.w(width.get()))
                        .child(for_each(
                            move || rows.get(),
                            |row| *row,
                            move |row| {
                                let id = untrack(|| active_id.get());
                                // The window as it was in its turn, kept while it goes.
                                let shown = signal(untrack(|| active.get()));
                                effect(move || {
                                    if turn.get() == row
                                        && let Some(t) = active.get().filter(|t| Some(t.id) == id)
                                    {
                                        shown.set(Some(t));
                                    }
                                });
                                let place = place_of(row == 0, move || turn.get() == row);
                                let sized = sized.clone();
                                window_title(shown)
                                    .absolute()
                                    .top(0.0)
                                    .bottom(0.0)
                                    .left(0.0)
                                    .dyn_style(move |s| {
                                        let p = place.get();
                                        s.opacity((1.0 - p.abs()).max(0.0)).translate_y(-16.0 * p)
                                    })
                                    .on_resized(move |w, _| {
                                        if untrack(|| turn.get()) != row {
                                            return;
                                        }
                                        if sized.replace(true) {
                                            width.spring_to(w, flow());
                                        } else {
                                            width.set(w);
                                        }
                                    })
                            },
                        )),
                )
                .child(edge_fade())
        },
    )
    .enter(|s| s.opacity(0.0_f32).scale(0.8_f32))
    .exit(|s| s.opacity(0.0_f32).scale(0.8_f32))
    .transition(overshoot(350))
}

/// How the window island and its titles move: quick, without going past.
fn flow() -> Spring {
    Spring::from_ratio(500.0, 1.0)
}
/// How long what goes takes to be gone, with room to spare.
const GOING: Duration = Duration::from_millis(450);
/// How wide the fade at the end of the window island is: as wide as the room after a title, so
/// that it is over nothing until a title is cut off.
const EDGE: f32 = 14.0;

/// Runs `f` once what started going now has gone.
fn after_going(f: impl FnOnce() + 'static) {
    set_timeout(GOING, f);
}

/// Where a row or a title is, from -1 below, where it comes from, through 0 in its place, to 1
/// above, where it goes. It is in its place from the start if `first`, and goes up once `here`
/// is false.
fn place_of(first: bool, here: impl Fn() -> bool + 'static) -> Animated<f32> {
    let place = animated(if first { 0.0f32 } else { -1.0 });
    effect(move || {
        let to = if here() { 0.0 } else { 1.0 };
        if untrack(|| place.get()) != to {
            place.spring_to(to, flow());
        }
    });
    place
}

/// The end of the window island fading into it, over a title that is wider than it is for now.
/// It is drawn in the island's own round end, which it would otherwise stick out of.
fn edge_fade() -> Svg {
    svg()
        .absolute()
        .top(0.0)
        .right(0.0)
        .w(EDGE)
        .h(ISLAND)
        .pointer_events_none()
        .dyn_style(|s| {
            let c = colors().surface_container;
            s.fill(
                Gradient::linear(0.0, 0.0, EDGE, 0.0)
                    .units(GradientUnits::UserSpace)
                    .stop(0.0, c.with_alpha(0.0))
                    .stop(1.0, c),
            )
        })
        .child(svg::rounded_rect(
            (EDGE - ISLAND) as f64,
            0.0,
            ISLAND as f64,
            ISLAND as f64,
            (ISLAND / 2.0) as f64,
        ))
}

/// A window's current application icon.
fn window_icon(app_id: Memo<Option<String>>) -> View {
    let icon = memo(move || app_id.with(|a| a.as_deref().and_then(|a| app_id_icon(a, 64))));
    dyn_view(move || match icon.get() {
        Some(path) => img(path).size(22.0).flex_none().into_view(),
        None => div()
            .size(22.0)
            .rounded(11.0_f32)
            .flex()
            .items_center()
            .justify_center()
            .flex_none()
            .dyn_style(|s| s.bg(colors().secondary_container))
            .child(sym(icons::DESKTOP_WINDOWS, 13.0, |c| {
                c.on_secondary_container
            }))
            .into_view(),
    })
}

/// A window's icon and title, on one line.
fn window_title(shown: Signal<Option<Focused>>) -> Div {
    let app_id = memo(move || shown.with(|t| t.as_ref().map(|t| t.app_id.clone())));
    let title =
        memo(move || shown.with(|t| t.as_ref().map(|t| t.title.clone()).unwrap_or_default()));
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(8.0_f32)
        .pr(9.0_f32)
        .child(window_icon(app_id))
        .child(changing_title(title))
}

/// A title that, when it changes, comes up from below as the last one goes up. It is as wide as
/// the title it has now; one that goes lies over what is beside it until it has gone. A change of
/// a few characters in one place (a spinner, a counter, a clock) is made where it is, without
/// coming up.
fn changing_title(title: Memo<String>) -> Div {
    let turn = signal(0u64);
    let lines = signal(vec![(0u64, untrack(|| title.get()))]);
    let seen = RefCell::new(untrack(|| title.get()));
    effect(move || {
        let text = title.get();
        let small = {
            let seen = seen.borrow();
            if *seen == text {
                return;
            }
            small_change(&seen, &text)
        };
        seen.replace(text.clone());
        if small {
            lines.update(|lines| {
                if let Some(last) = lines.last_mut() {
                    last.1 = text;
                }
            });
            return;
        }
        let gone = untrack(|| turn.get());
        lines.update(|lines| lines.push((gone + 1, text)));
        turn.set(gone + 1);
        after_going(move || {
            if lines.is_alive() {
                lines.update(|lines| lines.retain(|(t, _)| *t != gone));
            }
        });
    });
    div()
        .relative()
        .flex_none()
        // Unseen, it gives this the width of the title it has now.
        .child(
            txt(BAR_TEXT, move || title.get())
                .flex_none()
                .max_w(320.0)
                .overflow_hidden()
                .opacity(0.0_f32),
        )
        .child(for_each(
            move || lines.with(|lines| lines.iter().map(|(t, _)| *t).collect::<Vec<_>>()),
            |t| *t,
            move |t| {
                let place = place_of(t == 0, move || turn.get() == t);
                txt(BAR_TEXT, move || {
                    lines.with(|lines| {
                        lines
                            .iter()
                            .find(|(l, _)| *l == t)
                            .map(|(_, text)| text.clone())
                            .unwrap_or_default()
                    })
                })
                .flex_none()
                .max_w(320.0)
                .overflow_hidden()
                .absolute()
                .top(0.0)
                .left(0.0)
                .dyn_style(move |s| {
                    let p = place.get();
                    s.opacity((1.0 - p.abs()).max(0.0)).translate_y(-16.0 * p)
                })
            },
        ))
}

/// Whether `new` is `old` with a few characters changed in one place, and at least twice as many
/// kept: what a spinner, a counter or a clock in a title does, not a new title.
fn small_change(old: &str, new: &str) -> bool {
    let (old, new): (Vec<char>, Vec<char>) = (old.chars().collect(), new.chars().collect());
    let head = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
    let tail = old[head..]
        .iter()
        .rev()
        .zip(new[head..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let kept = head + tail;
    let changed = old.len().max(new.len()) - kept;
    changed <= 4 && kept >= 3.max(2 * changed)
}

/// The window with the keyboard, as an island of the bar.
pub fn island() -> Island {
    Island::new("window", |output, at| window_island(output, at).into_view())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spinner_or_a_counter_is_a_small_change() {
        assert!(small_change("✳ Claude Code", "✶ Claude Code"));
        assert!(small_change("Copying 3/10", "Copying 4/10"));
        assert!(small_change("Downloading 9%", "Downloading 10%"));
        assert!(small_change("12:59 - clock", "13:00 - clock"));
    }

    #[test]
    fn another_title_is_not_a_small_change() {
        assert!(!small_change("New Tab", "New Tab - Google Chrome"));
        assert!(!small_change("✳ Claude Code", "✳ Fixing the window title"));
        assert!(!small_change("", "abc"));
        assert!(!small_change("ab", "ac"));
    }
}
