//! The stock panels: quick settings, the calendar, the launcher, the clipboard's history, what is
//! playing and the question before shutting down.
//!
//! The panel manager itself lives in [`shoji_core::panels`]; this registers these six with it, so
//! an interface of your own can use them, replace them or add to them.

use anti_webui::prelude::*;
pub use shoji_core::panels::{
    PanelDef, PanelId, close, current, from_above, install, is_open, lookup, open, register,
    rise_in, toggle, view,
};

/// The stock panels, for the code that opens them without spelling their ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    QuickSettings,
    Calendar,
    Launcher,
    Clipboard,
    Media,
    Power,
}

impl Panel {
    /// The id the panel is registered under, which `msg toggle <id>` also takes.
    pub const fn id(self) -> PanelId {
        PanelId(match self {
            Self::QuickSettings => "quick-settings",
            Self::Calendar => "calendar",
            Self::Launcher => "launcher",
            Self::Clipboard => "clipboard",
            Self::Media => "media",
            Self::Power => "power",
        })
    }
}

impl From<Panel> for PanelId {
    fn from(panel: Panel) -> Self {
        panel.id()
    }
}

/// Registers the stock panels. Call it before [`install`] runs for the outputs.
pub fn register_stock() {
    // How far the panels at the top keep from it: they cover all of the output, the bar included.
    let below_bar = crate::bar::HEIGHT as f32 + 6.0;

    register(
        PanelDef::new(Panel::QuickSettings, crate::quick_settings::panel)
            .place(move |d| d.items_end().justify_start().pt(below_bar).pr(14.0))
            .enter(from_above)
            // Out of a page of quick settings first, then out of the panel.
            .on_escape(crate::quick_settings::back),
    );
    register(
        PanelDef::new(Panel::Calendar, |_| crate::calendar::panel())
            .place(move |d| d.items_center().justify_start().pt(below_bar))
            .enter(from_above),
    );
    register(
        PanelDef::new(Panel::Launcher, crate::launcher::panel)
            .with_field()
            .veiled(),
    );
    register(
        PanelDef::new(Panel::Clipboard, crate::clipboard::panel)
            .with_field()
            .veiled(),
    );
    // Under the bar's island of it, which it places itself under.
    register(
        PanelDef::new(Panel::Media, crate::media_panel::panel)
            .place(move |d| d.items_start().justify_start().pt(below_bar))
            .enter(from_above),
    );
    register(PanelDef::new(Panel::Power, |_| crate::power_dialog::panel()).veiled());
}
