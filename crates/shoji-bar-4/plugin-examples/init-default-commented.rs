//! Your changes to the stock shoji-bar-4 interface. Copy this to `plugins/init-default.rs`.
//!
//! As it stands this file changes nothing: the shell looks exactly like the stock one, only that it
//! goes through `patch` below. Every change is in a commented block: delete the two lines that hold
//! `/*` and `*/` around a block to turn it on. Try them one at a time.
//!
//! To see a change, in the project directory:
//!
//!     alter-bar reload
//!
//! which builds the project if anything changed (cargo decides) and restarts the bar. Without
//! alter-bar: `cargo build --release`, then restart the shell.
//!
//! If the build fails, cargo says where, and the bar you have keeps running. If a change builds but
//! breaks the shell, `target/release/shoji-bar-4 --default` starts the stock interface instead, and
//! so does putting the blocks back in their comments.
//!
//! Names you can use:
//!   islands of the bar:  launcher, workspaces, window, clock, media, tray, status
//!   panels:              quick-settings, calendar, launcher, clipboard, media, power
//!
//! Full guide: docs/customization.md in the project.

// What the blocks below use. Unused ones only produce warnings.
#![allow(unused_imports)]

use anti_webui::prelude::*;
use shoji_core::theme::colors;
use shoji_core::ui::island;
use shoji_default::panels::{PanelDef, from_above};
use shoji_default::{Island, Ui};

pub fn patch(ui: &mut Ui) {
    // ---------------------------------------------------------------------------------------
    // 1. A shorter bar, without the tray, and a greeting of your own next to the clock.
    //    (The stock islands are drawn for a height of 38: a few pixels less is fine.)
    // ---------------------------------------------------------------------------------------
    /*
    if let Some(bar) = ui.bar.as_mut() {
        bar.height = 34;
        bar.remove("tray");
        bar.insert_after(
            "clock",
            Island::new("greeting", |_output| {
                island().px(12.0_f32).child(text("hello from init-default.rs"))
            }),
        );
    }
    */

    // ---------------------------------------------------------------------------------------
    // 2. Swap the two sides of the bar: the launcher, workspaces and window go to the right,
    //    what was on the right (media, tray, status) goes to the left.
    // ---------------------------------------------------------------------------------------
    /*
    if let Some(bar) = ui.bar.as_mut() {
        std::mem::swap(&mut bar.left, &mut bar.right);
    }
    */

    // ---------------------------------------------------------------------------------------
    // 3. No dock (the strip that rises from the bottom edge), and no volume/brightness pill.
    // ---------------------------------------------------------------------------------------
    /*
    ui.dock = false;
    ui.services.osd = false;
    */

    // ---------------------------------------------------------------------------------------
    // 4. Replace a stock panel: click the clock island (or run `shoji-bar-4 msg toggle calendar`)
    //    and a note shows up instead of the calendar.
    // ---------------------------------------------------------------------------------------
    /*
    ui.replace_panel(
        PanelDef::new("calendar", |_output| {
            div()
                .p(28.0_f32)
                .rounded(28.0_f32)
                .dyn_style(|s| {
                    s.bg(colors().surface_container_high)
                        .text_color(colors().on_surface)
                })
                .child(text("No calendar today."))
        })
        // Hang it from the top, under the bar, in the middle.
        .place(|root| root.items_center().justify_start().pt(48.0_f32))
        .enter(from_above),
    );
    */

    // ---------------------------------------------------------------------------------------
    // 5. A panel of your own: `shoji-bar-4 msg toggle hello` opens it in the middle of the screen
    //    with the desktop blurred behind it; a click outside it, or Escape, closes it.
    // ---------------------------------------------------------------------------------------
    /*
    ui.replace_panel(
        PanelDef::new("hello", |_output| {
            div()
                .p(32.0_f32)
                .rounded(32.0_f32)
                .dyn_style(|s| {
                    s.bg(colors().primary_container)
                        .text_color(colors().on_primary_container)
                })
                .child(text("Hello! This panel is yours."))
        })
        .veiled(),
    );
    */

    // ---------------------------------------------------------------------------------------
    // 6. No bar at all. (Everything else stays; panels still open with `msg toggle`.)
    // ---------------------------------------------------------------------------------------
    /*
    ui.bar = None;
    */

    // Nothing is active until you remove the comment markers above.
    let _ = ui;
}
