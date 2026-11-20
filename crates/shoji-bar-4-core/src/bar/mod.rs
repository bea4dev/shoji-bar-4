//! The bar along an edge of every output: the launcher and the workspaces at the start, the window
//! with the keyboard and the clock in the middle, and what is playing, the tray and the state of the
//! system at the end. Each group floats in an island of its own, on a pane of frosted glass that is
//! all of the bar. Vertical bars use compact icons and stack the clock.

use std::cell::Cell;

use crate::bar::island::Island;
use crate::context::Cx;
use crate::placement::BarPlacement;
use crate::theme::{TextStyle, colors};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;

pub mod clock;
pub mod island;
pub mod launcher;
pub mod media;
pub mod status;
pub mod tray;
pub mod window;
pub mod workspaces;

pub use status::{battery_icon, wifi_icon};
pub use window::{Focused, focused_island};
pub use workspaces::{Ws, workspaces_island};

/// The size of the words in the bar: smaller than Material's titles, as a bar is.
pub(crate) const BAR_TEXT: TextStyle = TextStyle {
    size: 13.0,
    line_height: 18.0,
    letter_spacing: 0.1,
    weight: 500,
};

#[derive(Clone, Default)]
pub struct Bar {
    pub left: Vec<Island>,
    pub center: Vec<Island>,
    pub right: Vec<Island>,
}

impl Bar {
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the bar on `output`, and keeps it up to date with the placement.
    pub fn open(&self, output: Output) {
        crate::shell_layer::wayland::open_bar(self.clone(), output);
    }

    pub fn left(mut self, islands: impl IntoIterator<Item = Island>) -> Self {
        self.left = islands.into_iter().collect();
        self
    }

    pub fn center(mut self, islands: impl IntoIterator<Item = Island>) -> Self {
        self.center = islands.into_iter().collect();
        self
    }

    pub fn right(mut self, islands: impl IntoIterator<Item = Island>) -> Self {
        self.right = islands.into_iter().collect();
        self
    }
}

/// The bar as it is drawn on `output`: the islands of `bar`, on a pane of frosted glass.
pub fn view(bar: &Bar, output: Output, at: BarPlacement) -> Div {
    let cx = Cx::new(output, at);
    let index = Cell::new(0);
    let draw = |island: &Island| {
        let i = index.get();
        index.set(i + 1);
        island.view(&cx, i)
    };
    let vertical = at.is_vertical();
    let mut root = div()
        .select_none()
        .size_full()
        .relative()
        // Its own stacking context, for the glass to go behind the islands but not behind this.
        .z_index(0)
        .flex()
        .items_center()
        .gap(10.0_f32);
    root = if vertical {
        root.flex_col().px(6.0_f32).py(14.0_f32)
    } else {
        root.flex_row().px(14.0_f32)
    };
    let leading = div().flex().items_center().gap(10.0_f32).flex_1();
    let leading = if vertical {
        leading.flex_col().min_h(0.0)
    } else {
        leading.flex_row().min_w(0.0)
    };
    let trailing = div()
        .flex()
        .items_center()
        .justify_end()
        .gap(10.0_f32)
        .flex_1();
    let trailing = if vertical {
        trailing.flex_col().min_h(0.0)
    } else {
        trailing.flex_row().min_w(0.0)
    };
    root.dyn_style(|s| s.text_color(colors().on_surface))
        // Faint, but enough for the compositor to blur what is behind it: it blurs behind what is
        // at least a quarter opaque.
        .child(
            div()
                .absolute()
                .z_index(-1)
                .inset(0.0)
                .pointer_events_none()
                .dyn_style(|s| s.bg(colors().surface.with_alpha(0.34))),
        )
        .child(
            bar.left
                .iter()
                .fold(leading, |group, island| group.child(draw(island))),
        )
        .children(bar.center.iter().map(|island| draw(island)))
        .child(
            bar.right
                .iter()
                .fold(trailing, |group, island| group.child(draw(island))),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn island(id: &str) -> Island {
        Island::new(id, |_| unreachable!())
    }

    #[test]
    fn the_order_of_the_calls_does_not_matter() {
        let bar = Bar::new()
            .right([island("c")])
            .left([island("a")])
            .center([island("b")]);

        assert_eq!(bar.left[0].id, "a");
        assert_eq!(bar.center[0].id, "b");
        assert_eq!(bar.right[0].id, "c");
    }
}
