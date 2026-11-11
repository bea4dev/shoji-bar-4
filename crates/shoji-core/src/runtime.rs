//! Starting the shell: the process-wide setup, then the shell's event loop.

use anti_webui::shell::prelude::*;

/// Process-wide setup that has to happen before the shell starts, while the process still has a
/// single thread.
fn prepare() {
    // The patched AntiWebUI reads its zoom from the environment, once.
    let zoom = crate::settings::stored_ui_scale();
    if (zoom - 1.0).abs() > 1e-3 && std::env::var_os("ANTIWEBUI_UI_ZOOM").is_none() {
        // SAFETY: nothing else is running yet, as the doc above says, so no thread can be reading
        // the environment while it changes.
        unsafe { std::env::set_var("ANTIWEBUI_UI_ZOOM", zoom.to_string()) };
    }
}

/// Starts the shell and runs `root` as the root of its interface, which decides what is on the
/// screen (it typically registers panels and opens the bar, dock and so on for every output).
///
/// Returns when the shell stops; exits the process with an error code if it cannot start.
pub fn start(root: impl FnOnce()) {
    prepare();
    let result = Shell::new()
        .namespace("shoji-bar-4")
        .app_id("shoji-bar-4")
        .run(root);
    if let Err(e) = result {
        eprintln!("shoji-bar-4: {e}");
        std::process::exit(1);
    }
}
