//! Geometry derived from the configured bar edge and padding. One value is shared by one built
//! view, so every island and panel uses the same orientation and spacing snapshot.

use crate::settings::{BarPosition, ShellPadding, settings};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;

pub const HEIGHT: i32 = 38;
pub const WIDTH: i32 = 52;

/// Which side of an output a bar-adjacent panel belongs on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelSide {
    Below,
    Left,
    Right,
}

/// An invoking island's centre in output coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnchorPoint {
    pub x: f32,
    pub y: f32,
}

/// The bar's position, padding, and every geometric fact derived from them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarPlacement {
    position: BarPosition,
    padding: ShellPadding,
}

impl BarPlacement {
    pub const fn new(position: BarPosition) -> Self {
        Self::with_padding(position, ShellPadding::DEFAULT)
    }

    pub const fn with_padding(position: BarPosition, padding: ShellPadding) -> Self {
        Self { position, padding }
    }

    pub fn position(self) -> BarPosition {
        self.position
    }

    pub fn padding(self) -> ShellPadding {
        self.padding
    }

    pub fn is_vertical(self) -> bool {
        self.position.is_vertical()
    }

    pub fn edge(self) -> Anchor {
        match self.position {
            BarPosition::Top => Anchor::TOP,
            BarPosition::Left => Anchor::LEFT,
            BarPosition::Right => Anchor::RIGHT,
        }
    }

    pub fn side(self) -> PanelSide {
        match self.position {
            BarPosition::Top => PanelSide::Below,
            BarPosition::Left => PanelSide::Left,
            BarPosition::Right => PanelSide::Right,
        }
    }

    pub fn thickness(self) -> i32 {
        if self.is_vertical() { WIDTH } else { HEIGHT }
    }

    #[cfg(test)]
    pub fn gaps(self) -> f32 {
        self.padding.gaps as f32
    }

    pub fn top_pad(self) -> f32 {
        self.padding.effective_top() as f32
    }

    pub fn right_pad(self) -> f32 {
        self.padding.effective_right() as f32
    }

    pub fn bottom_pad(self) -> f32 {
        self.padding.effective_bottom() as f32
    }

    pub fn left_pad(self) -> f32 {
        self.padding.effective_left() as f32
    }

    pub fn bar_edge_pad(self) -> f32 {
        match self.position {
            BarPosition::Top => self.top_pad(),
            BarPosition::Left => self.left_pad(),
            BarPosition::Right => self.right_pad(),
        }
    }

    pub fn clearance(self) -> f32 {
        self.thickness() as f32 + self.bar_edge_pad()
    }

    /// Margin for the notification layer (`Exclusive::None`), which the compositor already keeps
    /// clear of the bar's exclusive zone.
    pub fn notification_margin(self, shadow_pad: i32) -> Margin {
        Margin {
            top: self.padding.effective_top() as i32 - shadow_pad,
            right: self.padding.effective_right() as i32 - shadow_pad,
            ..Default::default()
        }
    }

    /// Entry displacement for a panel adjacent to the bar.
    pub fn enter_offset(self) -> (f32, f32) {
        match self.side() {
            PanelSide::Below => (0.0, -80.0),
            PanelSide::Left => (-80.0, 0.0),
            PanelSide::Right => (80.0, 0.0),
        }
    }

    /// Exit displacement for a panel adjacent to the bar.
    pub fn exit_offset(self) -> (f32, f32) {
        match self.side() {
            PanelSide::Below => (0.0, -8.0),
            PanelSide::Left => (-8.0, 0.0),
            PanelSide::Right => (8.0, 0.0),
        }
    }
}

/// The current setting, as one immutable placement snapshot.
pub fn current() -> BarPlacement {
    settings().with(|s| BarPlacement::with_padding(s.bar_position, s.padding.normalized()))
}

/// Follows placement without waking downstream users for unrelated settings changes.
pub fn memoized() -> Memo<BarPlacement> {
    memo(current)
}

/// Follows only the bar edge so padding edits update styles in place without rebuilding open views.
pub fn position_memo() -> Memo<BarPosition> {
    memo(|| settings().with(|s| s.bar_position))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clearance_and_side_padding_follow_edge_overrides() {
        let padding = ShellPadding {
            gaps: 16,
            top: Some(8),
            right: Some(64),
            bottom: Some(0),
            left: None,
        };
        let top = BarPlacement::with_padding(BarPosition::Top, padding);
        assert_eq!(top.gaps(), 16.0);
        assert_eq!(top.top_pad(), 8.0);
        assert_eq!(top.right_pad(), 64.0);
        assert_eq!(top.bottom_pad(), 0.0);
        assert_eq!(top.left_pad(), 16.0);
        assert_eq!(top.bar_edge_pad(), 8.0);
        assert_eq!(top.clearance(), HEIGHT as f32 + 8.0);

        let left = BarPlacement::with_padding(BarPosition::Left, padding);
        assert_eq!(left.bar_edge_pad(), 16.0);
        assert_eq!(left.clearance(), WIDTH as f32 + 16.0);

        let right = BarPlacement::with_padding(BarPosition::Right, padding);
        assert_eq!(right.bar_edge_pad(), 64.0);
        assert_eq!(right.clearance(), WIDTH as f32 + 64.0);
    }

    #[test]
    fn notification_margin_compensates_shadow_without_double_counting_bar_thickness() {
        let padding = ShellPadding {
            gaps: 16,
            top: Some(8),
            right: Some(20),
            bottom: None,
            left: None,
        };
        assert_eq!(
            BarPlacement::with_padding(BarPosition::Top, padding).notification_margin(14),
            Margin {
                top: 8 - 14,
                right: 20 - 14,
                ..Default::default()
            }
        );
        assert_eq!(
            BarPlacement::with_padding(BarPosition::Right, padding).notification_margin(14),
            Margin {
                top: 8 - 14,
                right: 20 - 14,
                ..Default::default()
            }
        );

        let zero = ShellPadding {
            gaps: 0,
            top: Some(0),
            right: Some(0),
            bottom: Some(0),
            left: Some(0),
        };
        assert_eq!(
            BarPlacement::with_padding(BarPosition::Left, zero).notification_margin(14),
            Margin {
                top: -14,
                right: -14,
                ..Default::default()
            }
        );
    }
}
