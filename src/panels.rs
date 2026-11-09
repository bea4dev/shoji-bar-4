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

    /// How long its closing takes, after which the layer goes.
    fn exit_time(self) -> Duration {
        Duration::from_millis(match self {
            Self::Launcher => 380,
            _ => 300,
        })
    }
}

thread_local! {
    /// Which panel is open on which output, by connector name.
    static OPEN: Cell<Option<Signal<Option<(String, Panel)>>>> = const { Cell::new(None) };
}

fn state() -> Signal<Option<(String, Panel)>> {
    OPEN.with(|o| {
        o.get().unwrap_or_else(|| {
            let (s, _) = create_root(|| signal(None));
            o.set(Some(s));
            s
        })
    })
}

pub fn is_open(output: &Output, panel: Panel) -> bool {
    state().with(|s| s.as_ref().is_some_and(|(o, p)| *o == output.name && *p == panel))
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
                pending.set(Some(set_timeout(panel.exit_time(), move || {
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
            let keyboard = match panel {
                Panel::Launcher => Keyboard::Exclusive,
                _ => Keyboard::OnDemand,
            };
            let content_output = output.clone();
            let layer = open_layer(
                LayerOptions::new()
                    .layer(Layer::Top)
                    .anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT)
                    .keyboard(keyboard)
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

/// All of the output: a catcher of clicks outside the panel, and the panel where it goes.
fn surface(output: Output, panel: Panel, wanted: Memo<bool>) -> Div {
    let content = move || match panel {
        Panel::QuickSettings => crate::quick_settings::panel(output.clone()),
        Panel::Calendar => crate::calendar::panel(),
        Panel::Launcher => crate::launcher::panel(output.clone()),
    };
    let (place, enter): (fn(Div) -> Div, fn(Style) -> Style) = match panel {
        Panel::QuickSettings => (
            |d| d.items_end().justify_start().pt(6.0).pr(10.0),
            |s| s.opacity(0.0).scale(0.82).translate_y(-18.0),
        ),
        Panel::Calendar => (
            |d| d.items_center().justify_start().pt(6.0),
            |s| s.opacity(0.0).scale(0.82).translate_y(-18.0),
        ),
        Panel::Launcher => (
            |d| d.items_center().justify_end().pb(12.0),
            |s| s.opacity(0.0).scale(0.7).translate_y(80.0),
        ),
    };
    let root = div()
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .on_key_down(|e| {
            if e.key == Key::Named(NamedKey::Escape) {
                close();
            }
        })
        // Behind the panel: what is clicked outside it.
        .child(div().absolute().inset(0.0).on_click(move |_| {
            if untrack(|| wanted.get()) {
                close();
            }
        }));
    let mut root = place(root);
    if panel == Panel::Launcher {
        // A veil over the desktop, so that the launcher stands out.
        root = root.child(
            show(move || wanted.get(), || {
                div()
                    .absolute()
                    .inset(0.0)
                    .pointer_events_none()
                    .dyn_style(|s| s.bg(crate::theme::colors().scrim.with_alpha(0.32)))
            })
            .appear()
            .enter(|s| s.opacity(0.0))
            .exit(|s| s.opacity(0.0))
            .transition(Transition::ms(300).ease_out()),
        );
    }
    root.child(
        show(move || wanted.get(), move || {
            content()
        })
        .appear()
        .enter(enter)
        .exit(|s| s.opacity(0.0).scale(0.9).translate_y(-8.0))
        .transition(crate::ui::overshoot(420)),
    )
}
