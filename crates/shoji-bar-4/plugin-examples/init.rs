//! Copy this to `plugins/init.rs` (at the root of the project) and run `cargo build --release`.
//!
//! With this file the stock interface is not used at all: `init` runs as the root of the shell, and
//! only what it puts on the screen is there. Everything below is yours to change.
//!
//! This one is a bar that only says what it is, and a panel, both coloured by the stock theme. Split it
//! into more files if you like: `mod helpers;` here loads `helpers.rs` from this directory.
//!
//! If it does not build, `cargo build` says where; if it builds but does not run well, start the
//! shell with `shoji-bar-4 --default` (or `SHOJI_SAFE=1`) to get the stock interface back.

use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use shoji_core::panels::{self, PanelDef};
use shoji_core::theme::{self, colors};

pub fn init() {
    // Commands from outside (`shoji-bar-4 msg ...`) and the colors. Both are optional, but
    // without the theme `colors()` has nothing to read.
    shoji_default::ipc::serve();
    theme::install();

    // A panel, openable with `shoji-bar-4 msg toggle hello`.
    panels::register(PanelDef::new("hello", |_output| {
        div()
            .p(24.0_f32)
            .rounded(24.0_f32)
            .dyn_style(|s| s.bg(colors().surface_container).text_color(colors().on_surface))
            .child(text("hello from init.rs"))
    }));

    for_each_output(|output| {
        open_layer(
            LayerOptions::bar(Anchor::TOP, 30).namespace("shoji-bar-4-bar"),
            || {
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .px(14.0_f32)
                    .dyn_style(|s| {
                        s.bg(colors().surface.with_alpha(0.6))
                            .text_color(colors().on_surface)
                    })
                    .child(text("my own bar"))
            },
        );
        panels::install(output);
    });
}
