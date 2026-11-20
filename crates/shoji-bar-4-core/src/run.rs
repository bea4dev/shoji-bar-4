//! Starting the shell: the subcommands that do not start one, and the shell itself.

use crate::bar::Bar;
use crate::{ipc, preview};
use anti_webui::shell::prelude::*;

/// Everything that describes a shell, given once to run. Motion and the panels join it in the next steps.
#[derive(Clone)]
pub struct Layout {
    pub bar: Bar,
}

/// Handles the subcommands that do not start a shell: `msg` talks to the one that is running and
/// `preview` draws a picture of a part of the interface. Whether there was one.
fn subcommand(layout: &Layout) -> bool {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("msg") => {
            if let Err(e) = ipc::send(&args[1..]) {
                eprintln!("shoji-bar-4: {e}");
                std::process::exit(1);
            }
            true
        }
        Some("preview") => {
            if let Err(e) = preview::run(&args[1..], layout) {
                eprintln!("shoji-bar-4: {e}");
                std::process::exit(1);
            }
            true
        }
        _ => false,
    }
}

/// Runs the shell: handles the `msg` and `preview` subcommands, otherwise connects to the compositor
/// and runs `root` once, as the root of the interface, on the UI thread. `layout` describes the
/// shell: `preview` draws it and `root` receives the same value, so what is previewed is what the
/// shell shows.
pub fn run(layout: Layout, root: impl FnOnce(Layout) + 'static) {
    if subcommand(&layout) {
        return;
    }
    let result = Shell::new()
        .namespace("shoji-bar-4")
        .app_id("shoji-bar-4")
        .run(move || root(layout));
    if let Err(e) = result {
        eprintln!("shoji-bar-4: {e}");
        std::process::exit(1);
    }
}
