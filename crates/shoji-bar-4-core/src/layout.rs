//! Combine `Part`s: a row, a column, and the space that pushes the rest apart.

use std::rc::Rc;

use anti_webui::{
    core::view::{IntoView, div},
    style::Styled,
};

use crate::context::{Cx, Part};

/// Parts side by side, from left to right
pub fn row(parts: impl IntoIterator<Item = Part>) -> Part {
    stack(parts, false)
}

/// Parts one below another, from top to bottom
pub fn column(parts: impl IntoIterator<Item = Part>) -> Part {
    stack(parts, true)
}

/// Empty space that grows to fill what is left, so what comes after goes to the far end.
pub fn spacer() -> Part {
    Rc::new(|_cx: &Cx| div().flex_1().into_view())
}

fn stack(parts: impl IntoIterator<Item = Part>, vertical: bool) -> Part {
    let parts: Vec<Part> = parts.into_iter().collect();
    Rc::new(move |cx| {
        let node = div().flex().items_center();
        let node = if vertical {
            node.flex_col()
        } else {
            node.flex_row()
        };
        node.children(parts.iter().map(|part| part(cx))).into_view()
    })
}
