//! The default shoji-bar-4 user interface: the bar, the dock, the panels, the lock screen, the
//! notifications and the OSD, built on the foundation in [`shoji_core`].

// The foundation lives in `shoji_core`; these keep `crate::theme::...` style paths working.
pub use shoji_core::{appinfo, pam, settings, theme, ui, wm};

mod bar;
mod battery_card;
mod calendar;
mod clipboard;
mod dock;
mod ipc;
mod launcher;
mod lock;
mod media_panel;
mod notify;
mod osd;
mod power_dialog;
mod panels;
mod preview;
mod qs_pages;
mod qs_settings;
mod quick_settings;
mod user;
mod wallpaper;

use anti_webui::shell::prelude::*;

/// Runs the shell the way the stock `shoji-bar-4` binary does: handles the `msg` and `preview`
/// subcommands, otherwise starts the shell with the default interface.
pub fn run() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("msg") {
        if let Err(e) = ipc::send(&args[1..]) {
            eprintln!("shoji-bar-4: {e}");
            std::process::exit(1);
        }
        return;
    }
    if args.first().map(String::as_str) == Some("preview") {
        if let Err(e) = preview::run(&args[1..]) {
            eprintln!("shoji-bar-4: {e}");
            std::process::exit(1);
        }
        return;
    }
    let result = Shell::new()
        .namespace("shoji-bar-4")
        .app_id("shoji-bar-4")
        .run(|| {
            ipc::serve();
            theme::install();
            wm::share_theme();
            notify::open_popups();
            osd::install();
            lock::install();
            panels::register_stock();
            for_each_output(|output| {
                bar::open(output.clone());
                dock::open(output.clone());
                wallpaper::open(output.clone());
                panels::install(output);
            });
        });
    if let Err(e) = result {
        eprintln!("shoji-bar-4: {e}");
        std::process::exit(1);
    }
}
