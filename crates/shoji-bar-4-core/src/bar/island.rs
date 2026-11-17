use std::rc::Rc;

use anti_webui::{
    core::view::{Div, View, div},
    shell::Output,
    style::{Length, Styled},
};

use crate::{
    placement::BarPlacement,
    theme::{colors, elevation},
};

#[derive(Clone)]
pub struct Island {
    pub id: String,
    pub build: Rc<dyn Fn(Output, BarPlacement) -> View>,
    // TODO: add other fields: shape, padding, hover, click, highlight, entrance
}

// Aux
pub const VERTICAL_ISLAND: f32 = 40.0;
/// A floating container of the bar: a pill on `surface_container`.
/// How tall an island of the bar is.
pub const ISLAND: f32 = 32.0;

/// A horizontal pill, or a vertical island with its height set by its contents.
pub fn bar_island(at: BarPlacement) -> Div {
    let node = island();
    if at.is_vertical() {
        node.flex_col()
            .w(VERTICAL_ISLAND)
            .h(Length::Auto)
            .rounded(VERTICAL_ISLAND / 2.0)
    } else {
        node
    }
}

pub fn island() -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .flex_none()
        .h(ISLAND)
        .rounded(ISLAND / 2.0)
        .shadows(elevation(1))
        .dyn_style(|s| s.bg(colors().surface_container))
}
