//! The foundation shared by every shoji-bar-4 interface: persisted settings, the Material theme,
//! small UI building blocks, desktop-entry lookup, PAM authentication and the ShojiWM link.
//!
//! An interface (the stock one in `shoji-default`, or one of your own) is built on top of this.

pub mod appinfo;
pub mod pam;
pub mod settings;
pub mod theme;
pub mod ui;
pub mod wm;
