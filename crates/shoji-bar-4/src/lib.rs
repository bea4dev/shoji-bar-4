//! shoji-bar-4 as it ships: the bars that arrange the islands of `shoji-bar-4-core`. Another shell
//! can start from them, and change what it wants.

use shoji_bar_4_core::bar::{Bar, clock, launcher, media, status, tray, window, workspaces};

/// A bar with no islands, to build one's own from scratch.
pub fn empty_bar() -> Bar {
    Bar::new()
}

/// The bar of the shell as it ships: the launcher, the workspaces and the window at the start, the
/// clock in the middle, and what is playing, the tray and the state of the system at the end.
pub fn default_bar() -> Bar {
    Bar::new()
        .left([launcher::island(), workspaces::island(), window::island()])
        .center([clock::island()])
        .right([media::island(), tray::island(), status::island()])
}
