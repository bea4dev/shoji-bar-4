use std::{cell::Cell, rc::Rc};

use anti_webui::{
    core::{
        animation::{Spring, animated},
        reactive::{Memo, effect, memo, untrack},
        view::{Div, IntoView, div, for_each},
    },
    shell::{Output, workspaces},
    style::{Prop, Styled},
};
use anti_webui_md3e::typography::TextStyle;

use crate::{
    bar::island::{Island, VERTICAL_ISLAND, bar_island},
    placement::BarPlacement,
    ui::emphasized,
};
use crate::{theme::colors, ui::overshoot};

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
pub fn workspaces_of(output: Output) -> Memo<Vec<Ws>> {
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
/// The pill's thickness on the cross axis.
const PILL: f32 = 22.0;

/// The workspaces of the output as dots, with a pill under the active one that stretches as it
/// goes from one to another: its front edge leads on a quick spring, its back follows on a slow
/// one.
pub fn workspaces_island(list: Memo<Vec<Ws>>, activate: Rc<dyn Fn(&str)>, at: BarPlacement) -> Div {
    let vertical = at.is_vertical();
    // Between one workspace going and the next coming there may be none: the pill stays.
    let last = Cell::new(0);
    let active = memo(move || {
        let at = list
            .with(|l| l.iter().position(|w| w.active))
            .unwrap_or(last.get());
        last.set(at);
        at
    });
    let scrolled = activate.clone();
    let start = animated(0.0f32);
    let end = animated(ACTIVE);
    let first = Cell::new(true);
    effect(move || {
        let at = active.get() as f32 * SLOT;
        let (to_start, to_end) = (at, at + ACTIVE);
        untrack(|| {
            if first.replace(false) {
                start.set(to_start);
                end.set(to_end);
            } else if to_start >= start.get_untracked() {
                end.spring_to(to_end, Spring::from_ratio(900.0, 0.7));
                start.spring_to(to_start, Spring::from_ratio(260.0, 0.85));
            } else {
                start.spring_to(to_start, Spring::from_ratio(900.0, 0.7));
                end.spring_to(to_end, Spring::from_ratio(260.0, 0.85));
            }
        })
    });
    // Under the slots, also while one of them comes or goes: fading or scaling, a slot is
    // painted as one image with what is in it, and would otherwise go under the pill.
    let pill = if vertical {
        div()
            .absolute()
            .z_index(-1)
            .left(0.0)
            .w(PILL)
            .rounded(PILL / 2.0)
            .dyn_style(move |s| {
                let (start, end) = (start.get(), end.get());
                s.top(start).h((end - start).max(PILL)).bg(colors().primary)
            })
    } else {
        div()
            .absolute()
            .z_index(-1)
            .top(0.0)
            .h(PILL)
            .rounded(PILL / 2.0)
            .dyn_style(move |s| {
                let (start, end) = (start.get(), end.get());
                s.left(start)
                    .w((end - start).max(PILL))
                    .bg(colors().primary)
            })
    };
    let slots = for_each(
        move || list.get(),
        |w| w.id.clone(),
        move |w| {
            let id = w.id.clone();
            // Once gone from the list it leaves as it was, without turning back into a dot.
            let seen = std::cell::RefCell::new(w.clone());
            let me = memo(move || {
                if let Some(now) = list.with(|l| l.iter().find(|x| x.id == id).cloned()) {
                    *seen.borrow_mut() = now;
                }
                seen.borrow().clone()
            });
            let on = memo(move || me.with(|w| w.active));
            let empty = memo(move || me.with(|w| w.empty));
            let urgent = memo(move || me.with(|w| w.urgent));
            let name = w.name.clone();
            let activate = activate.clone();
            let slot = if vertical {
                div()
                    .relative()
                    .w(PILL)
                    .flex_none()
                    .transition_of([Prop::Height], emphasized(350))
                    .dyn_style(move |s| s.h(if on.get() { ACTIVE } else { SLOT }))
            } else {
                div()
                    .relative()
                    .h(PILL)
                    .flex_none()
                    .transition_of([Prop::Width], emphasized(350))
                    .dyn_style(move |s| s.w(if on.get() { ACTIVE } else { SLOT }))
            };
            let number = TextStyle::LABEL_MEDIUM
                .apply(div())
                .absolute()
                .flex()
                .items_center()
                .justify_center()
                .pointer_events_none()
                .transition(overshoot(400))
                .dyn_style(move |s| {
                    s.text_color(colors().on_primary)
                        .opacity(if on.get() { 1.0 } else { 0.0_f32 })
                        .scale(if on.get() { 1.0 } else { 0.4_f32 })
                })
                .child(name);
            let number = if vertical {
                number.top(0.0).left(0.0).right(0.0).h(ACTIVE)
            } else {
                number.top(0.0).bottom(0.0).left(0.0).w(ACTIVE)
            };
            slot.on_click(move |_| activate(&w.id))
                // The dot, which gives way to the number.
                .child(
                    div()
                        .absolute()
                        .inset(0.0)
                        .flex()
                        .items_center()
                        .justify_center()
                        .pointer_events_none()
                        .child(div().rounded(3.0_f32).transition(overshoot(350)).dyn_style(
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
                                    .scale(if on.get() { 0.0 } else { 1.0_f32 })
                            },
                        )),
                )
                // As long as the active slot on the main axis, whatever it is on the way, so the
                // number stays on the pill while the slot opens and closes.
                .child(number)
        },
    )
    // They open and close their place, so the others and the island move over with them.
    .enter(move |s| {
        if vertical {
            s.h(0.0).opacity(0.0_f32).scale(0.3_f32)
        } else {
            s.w(0.0).opacity(0.0_f32).scale(0.3_f32)
        }
    })
    .exit(move |s| {
        if vertical {
            s.h(0.0).opacity(0.0_f32).scale(0.3_f32)
        } else {
            s.w(0.0).opacity(0.0_f32).scale(0.3_f32)
        }
    })
    .transition(emphasized(350));
    let container = if vertical {
        // The outer radius is VERTICAL_ISLAND / 2 and the pill's is PILL / 2.
        // Their difference keeps the capsule centers together at either end.
        bar_island(at).py((VERTICAL_ISLAND - PILL) / 2.0)
    } else {
        bar_island(at).px(5.0_f32)
    };
    container
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
        .child(if vertical {
            div()
                .relative()
                .z_index(0)
                .flex()
                .flex_col()
                .items_center()
                .w(PILL)
                .child(pill)
                .child(slots)
        } else {
            div()
                .relative()
                .z_index(0)
                .flex()
                .flex_row()
                .items_center()
                .h(PILL)
                .child(pill)
                .child(slots)
        })
}

/// The workspaces of the output, as an island of the bar.
pub fn island() -> Island {
    Island::new("workspaces", |output, at| {
        let activate: Rc<dyn Fn(&str)> = Rc::new({
            let output = output.clone();
            move |id: &str| {
                if let Some(w) = workspaces().on(&output).into_iter().find(|w| w.id() == id) {
                    w.activate();
                }
            }
        });
        workspaces_island(workspaces_of(output), activate, at).into_view()
    })
}
