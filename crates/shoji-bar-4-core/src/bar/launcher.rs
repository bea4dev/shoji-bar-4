use crate::panels::toggle;
use crate::ui::launcher_glyph;
use anti_webui::prelude::{IntoView, Styled};
use anti_webui::{
    core::{
        reactive::{memo, signal},
        view::Div,
    },
    shell::Output,
};

use crate::{
    bar::island::{Island, ISLAND, VERTICAL_ISLAND, bar_island},
    panels::{Panel, is_open},
    placement::BarPlacement,
};

/// The button that opens the launcher: an icon on a shape that turns and blooms on hover.
pub fn launcher_island(output: Output, at: BarPlacement) -> Div {
    let hovered = signal(false);
    let open = memo({
        let output = output.clone();
        move || is_open(&output, Panel::Launcher)
    });
    let side = if at.is_vertical() {
        VERTICAL_ISLAND
    } else {
        ISLAND
    };
    bar_island(at)
        .size(side)
        .justify_center()
        .on_hover(move |h| hovered.set(h))
        .on_click(move |_| toggle(&output, Panel::Launcher))
        .child(launcher_glyph(open, hovered, || 15.0).size(26.0))
}

/// The launcher button, as an island of the bar.
pub fn island() -> Island {
    Island::new("launcher", |output, at| {
        launcher_island(output, at).into_view()
    })
}
