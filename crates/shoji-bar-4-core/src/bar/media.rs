use crate::bar::BAR_TEXT;
use crate::bar::island::{Island, VERTICAL_ISLAND, bar_island};
use crate::panels::{Panel, is_open, toggle};
use crate::placement::{AnchorPoint, BarPlacement};
use crate::theme::colors;
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui_md3e::motion::slow_spatial;

/// What is playing, while something plays: a little equalizer and, on the top bar, its title.
/// A click opens the player beside it; the middle button pauses it or plays it again.
pub fn media_island(output: Output, at: BarPlacement) -> Show {
    let vertical = at.is_vertical();
    let m = media();
    let current = memo(move || m.current());
    let open = memo({
        let output = output.clone();
        move || is_open(&output, Panel::Media)
    });
    show(
        move || current.with(|p| p.as_ref().is_some_and(|p| !p.title.is_empty())),
        move || {
            let output = output.clone();
            let playing = memo(move || {
                current.with(|p| {
                    p.as_ref()
                        .is_some_and(|p| p.status == PlaybackStatus::Playing)
                })
            });
            let here = node_ref();
            let content = if vertical {
                bar_island(at).size(VERTICAL_ISLAND).justify_center()
            } else {
                bar_island(at).px(12.0_f32).gap(8.0_f32).max_w(260.0)
            };
            let content = content
                .node_ref(here)
                .on_click(move |_| {
                    if let Some(r) = here.logical_rect() {
                        crate::panels::toggle_at(
                            &output,
                            Panel::Media,
                            AnchorPoint {
                                x: r.x + r.width / 2.0,
                                y: r.y + r.height / 2.0,
                            },
                        );
                    } else {
                        toggle(&output, Panel::Media);
                    }
                })
                .on_aux_click(move |e| {
                    if e.button == Some(PointerButton::Middle)
                        && let Some(p) = untrack(|| current.get())
                    {
                        p.play_pause();
                    }
                })
                .dyn_style(move |s| {
                    let c = colors();
                    if open.get() {
                        s.bg(c.primary_container).text_color(c.on_primary_container)
                    } else {
                        s
                    }
                })
                .child(equalizer(playing));
            let content = if vertical {
                content
            } else {
                content.child(
                    txt(BAR_TEXT, move || {
                        current.with(|p| {
                            p.as_ref()
                                .map(|p| p.title.chars().take(40).collect())
                                .unwrap_or_default()
                        })
                    })
                    .min_w(0.0)
                    .overflow_hidden(),
                )
            };
            pressable(content, |c| c.on_surface)
        },
    )
    .enter(|s| s.opacity(0.0_f32).scale(0.6_f32))
    .exit(|s| s.opacity(0.0_f32).scale(0.6_f32))
    .transition(overshoot(400))
}

/// Four bars that dance while `playing` and settle into dots when paused.
fn equalizer(playing: Memo<bool>) -> Div {
    let calming = follow(
        move || if playing.get() { 1.0 } else { 0.0 },
        slow_spatial(),
    );
    // Thirty times a second is plenty for bars that dance, and a quarter of the frames.
    let t = clock_at(30, move || calming.get() > 0.001);
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(2.0_f32)
        .h(13.0)
        .flex_none();
    for i in 0..4 {
        row = row.child(div().w(2.5).rounded(1.25_f32).dyn_style(move |s| {
            let level = calming.get();
            // Only asks for frames while it moves.
            let wave = if level > 0.001 {
                let x = t.get() * (5.0 + i as f32 * 1.3) + i as f32 * 1.7;
                (x.sin() * 0.5 + 0.5) * 0.7 + 0.3
            } else {
                0.0
            };
            s.h(2.5 + 10.5 * wave * level).bg(colors().primary)
        }));
    }
    row
}

/// What is playing, as an island of the bar.
pub fn island() -> Island {
    Island::new("media", |output, at| media_island(output, at).into_view())
}
