//! The panels that open over the desktop: quick settings, the calendar and the launcher. At most
//! one is open on an output. Each is a layer over all of the output that is there from when the
//! panel starts to open until it has finished closing; a click outside the panel closes it.

use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::Cell;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    QuickSettings,
    Calendar,
    Launcher,
}

impl Panel {
    const ALL: [Panel; 3] = [Self::QuickSettings, Self::Calendar, Self::Launcher];

    fn name(self) -> &'static str {
        match self {
            Self::QuickSettings => "quick-settings",
            Self::Calendar => "calendar",
            Self::Launcher => "launcher",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == name)
    }
}

/// How long a panel's closing takes, after which the layer goes.
const EXIT_TIME: Duration = Duration::from_millis(380);

/// Which panel is open on which output, by connector name.
type Open = Signal<Option<(String, Panel)>>;

thread_local! {
    static OPEN: Cell<Option<Open>> = const { Cell::new(None) };
}

fn state() -> Open {
    OPEN.with(|o| {
        o.get().unwrap_or_else(|| {
            let (s, _) = create_root(|| signal(None));
            o.set(Some(s));
            s
        })
    })
}

pub fn is_open(output: &Output, panel: Panel) -> bool {
    state().with(|s| {
        s.as_ref()
            .is_some_and(|(o, p)| *o == output.name && *p == panel)
    })
}

pub fn toggle(output: &Output, panel: Panel) {
    let now = is_open(output, panel);
    state().set((!now).then(|| (output.name.clone(), panel)));
}

pub fn open(output_name: &str, panel: Panel) {
    state().set(Some((output_name.to_owned(), panel)));
}

pub fn close() {
    state().set(None);
}

/// Whether anything is open, and where.
pub fn current() -> Option<(String, Panel)> {
    state().get()
}

/// Keeps the layers of the panels of `output` as they should be. Call once per output.
pub fn install(output: Output) {
    for panel in Panel::ALL {
        let wanted = memo({
            let output = output.clone();
            move || is_open(&output, panel)
        });
        // There while it opens, is open, and closes.
        let mounted = signal(false);
        let pending: Cell<Option<Timeout>> = Cell::new(None);
        effect(move || {
            if let Some(t) = pending.take() {
                t.cancel();
            }
            if wanted.get() {
                mounted.set(true);
            } else if mounted.get_untracked() {
                pending.set(Some(set_timeout(EXIT_TIME, move || {
                    if !untrack(|| wanted.get()) {
                        mounted.set(false);
                    }
                })));
            }
        });
        let output = output.clone();
        effect(move || {
            if !mounted.get() {
                return;
            }
            let content_output = output.clone();
            let layer = open_layer(
                LayerOptions::new()
                    .layer(Layer::Top)
                    .anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT)
                    // Over the bar and the dock too, so that a click anywhere closes it.
                    .exclusive(Exclusive::Ignore)
                    // And Escape, without clicking it first.
                    .keyboard(Keyboard::Exclusive)
                    .output(&output)
                    .namespace(format!("shoji-bar-4-{}", panel.name())),
                move || surface(content_output, panel, wanted),
            );
            // While it closes, clicks go to what is under it.
            effect(move || {
                layer.set_input_region(if wanted.get() {
                    InputRegion::Full
                } else {
                    InputRegion::Empty
                });
            });
        });
    }
}

/// A panel as it is when open, for pictures of it.
pub fn view(output: Output, panel: Panel) -> Div {
    let wanted = memo({
        let output = output.clone();
        move || is_open(&output, panel)
    });
    surface(output, panel, wanted)
}

/// How far the panels at the top keep from it: they cover all of the output, the bar included.
const BELOW_BAR: f32 = crate::bar::HEIGHT as f32 + 6.0;

/// Where a panel goes on the output, and how it comes in.
type Place = fn(Div) -> Div;
type Enter = fn(Style) -> Style;

/// Down from above, growing as it fades in.
fn from_above(s: Style) -> Style {
    s.opacity(0.0).scale(0.7).translate_y(-80.0)
}

/// All of the output: a catcher of clicks outside the panel, and the panel where it goes.
fn surface(output: Output, panel: Panel, wanted: Memo<bool>) -> Div {
    let content = move || match panel {
        Panel::QuickSettings => crate::quick_settings::panel(output.clone()),
        Panel::Calendar => crate::calendar::panel(),
        Panel::Launcher => crate::launcher::panel(output.clone()),
    };
    let (place, enter): (Place, Enter) = match panel {
        Panel::QuickSettings => (
            |d| d.items_end().justify_start().pt(BELOW_BAR).pr(14.0),
            from_above,
        ),
        Panel::Calendar => (
            |d| d.items_center().justify_start().pt(BELOW_BAR),
            from_above,
        ),
        // In the middle of the output, coming up a little as it grows.
        Panel::Launcher => (
            |d| d.items_center().justify_center(),
            |s| s.opacity(0.0).scale(0.8).translate_y(40.0),
        ),
    };
    // Keys go to what has the focus, and with nothing focused not to here but to what holds it: the
    // panel takes the focus as it opens, for Escape. The launcher gives it to its search instead.
    let focus = node_ref();
    if panel != Panel::Launcher {
        request_frame(move |_| focus.focus());
    }
    let root = div()
        .node_ref(focus)
        .focusable()
        .size_full()
        .relative()
        // Its own stacking context, so that the catcher below is behind the panel but not behind
        // this, which would take its clicks.
        .z_index(0)
        .flex()
        .flex_col()
        .select_none()
        .on_key_down(move |e| {
            // Out of a page of quick settings first, then out of the panel.
            if e.key == Key::Named(NamedKey::Escape)
                && !(panel == Panel::QuickSettings && crate::quick_settings::back())
            {
                close();
            }
        })
        // Behind the panel (`absolute` alone would put it above): what is clicked outside it.
        .child(div().absolute().inset(0.0).z_index(-1).on_click(move |_| {
            if untrack(|| wanted.get()) {
                close();
            }
        }));
    let mut root = place(root);
    if panel == Panel::Launcher {
        // A veil over the desktop, so that the launcher stands out.
        root = root.child(
            show(
                move || wanted.get(),
                || {
                    div()
                        .absolute()
                        .z_index(-1)
                        .inset(0.0)
                        .pointer_events_none()
                        .dyn_style(|s| s.bg(crate::theme::colors().scrim.with_alpha(0.32)))
                },
            )
            .appear()
            .enter(|s| s.opacity(0.0))
            .exit(|s| s.opacity(0.0))
            .transition(Transition::ms(300).ease_out()),
        );
    }
    root.child(
        show(move || wanted.get(), content)
            .appear()
            .enter(enter)
            .exit(|s| s.opacity(0.0).scale(0.9).translate_y(-8.0))
            .transition(crate::ui::overshoot(420)),
    )
}
