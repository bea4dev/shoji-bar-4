//! Copy this to `~/.config/shoji-bar-4/plugins/init-default.rs` and run `cargo build --release`.
//!
//! It keeps the stock interface and changes it before it is put on the screen. Delete the file
//! (and build again) to go back to the stock interface.

use anti_webui::prelude::*;
use shoji_default::{Island, Ui};

pub fn patch(ui: &mut Ui) {
    // A little shorter than the stock 38. The islands are drawn for 38, so go gently.
    if let Some(bar) = ui.bar.as_mut() {
        bar.height = 34;

        // Take an island off, and put one of your own after the clock.
        bar.remove("tray");
        bar.insert_after(
            "clock",
            Island::new("greeting", |_output| {
                div().child(text("hello from init-default.rs"))
            }),
        );
    }

    // No dock.
    ui.dock = false;
}
