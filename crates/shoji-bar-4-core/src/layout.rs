//! Combine `Part`s: a row, a column, and the space that pushes the rest apart.

use std::rc::Rc;

use anti_webui::{
    core::view::{IntoView, div},
    style::Styled,
};

use crate::context::{Cx, Part};

/// The space around the parts of a row or a column: one value for all, or one for each space.
pub struct Gaps(Vec<f32>);

impl From<f32> for Gaps {
    fn from(gap: f32) -> Self {
        Gaps(vec![gap])
    }
}

impl From<Vec<f32>> for Gaps {
    fn from(gaps: Vec<f32>) -> Self {
        Gaps(gaps)
    }
}

/// The `n + 1` spaces of `n` parts (before the first, between each two, after the last), or `None`
/// if `gaps` does not fit them.
fn spaces(gaps: &[f32], n: usize) -> Option<Vec<f32>> {
    let len = gaps.len();
    let gap_checker_logic = len == 0 || len == 1 || len + 1 == n || len == n + 1;
    if !gap_checker_logic {
        return None;
    }
    let mut spaces = vec![0.0; n + 1];
    match len {
        0 => {}
        _ if len == n + 1 => spaces.copy_from_slice(gaps), // all of them
        _ if len + 1 == n => spaces[1..n].copy_from_slice(gaps), // only between
        _ => spaces[1..n].fill(gaps[0]),                   // one value, between
    }

    Some(spaces)
}

/// Parts side by side, from left to right
pub fn row(gaps: impl Into<Gaps>, parts: impl IntoIterator<Item = Part>) -> Part {
    stack(gaps.into(), parts, false)
}

/// Parts one below another, from top to bottom
pub fn column(gaps: impl Into<Gaps>, parts: impl IntoIterator<Item = Part>) -> Part {
    stack(gaps.into(), parts, true)
}

/// Empty space that grows to fill what is left, so what comes after goes to the far end. It only
/// has room to grow if the row around it is wider than its content: a `row` inside an island is as
/// wide as its parts, so there the spacer does nothing.
pub fn spacer() -> Part {
    Rc::new(|_cx: &Cx| div().flex_1().into_view())
}

fn stack(gaps: Gaps, parts: impl IntoIterator<Item = Part>, vertical: bool) -> Part {
    let parts: Vec<Part> = parts.into_iter().collect();
    let uniform = match gaps.0[..] {
        [g] => Some(g),
        _ => None,
    };
    let spaces = match uniform {
        Some(_) => Vec::new(),
        None => spaces(&gaps.0, parts.len()).unwrap_or_else(|| {
            report();
            vec![0.0; parts.len() + 1]
        }),
    };
    Rc::new(move |cx| {
        let node = div().flex().items_center();
        let node = if vertical {
            node.flex_col()
        } else {
            node.flex_row()
        };
        let node = match uniform {
            Some(g) => node.gap(g).children(parts.iter().map(|part| part(cx))),
            None => {
                let space = |g: f32| if vertical { div().h(g) } else { div().w(g) }.flex_none();
                let mut node = node;
                for (part, g) in parts.iter().zip(&spaces) {
                    if *g > 0.0 {
                        node = node.child(space(*g));
                    }
                    node = node.child(part(cx));
                }
                if spaces[parts.len()] > 0.0 {
                    node = node.child(space(spaces[parts.len()]));
                }
                node
            }
        };
        node.into_view()
    })
}

// TODO: implement notification system to send errors as gap vector malformed
fn report() {
    eprintln!("shoji-bar-4: the gaps do not fit the parts, drawing without gaps.")
}

#[cfg(test)]
mod tests {
    use super::spaces;

    #[test]
    fn the_gaps_that_fit_the_parts() {
        assert_eq!(spaces(&[], 2), Some(vec![0.0, 0.0, 0.0])); // no gaps
        assert_eq!(spaces(&[8.0], 3), Some(vec![0.0, 8.0, 8.0, 0.0])); // between (2 middle gaps)
        assert_eq!(spaces(&[1.0], 2), Some(vec![0.0, 1.0, 0.0])); // between
        assert_eq!(spaces(&[1.0, 2.0, 3.0], 2), Some(vec![1.0, 2.0, 3.0])); // all
    }

    #[test]
    fn the_gaps_that_do_not_fit_are_refused() {
        assert_eq!(spaces(&[1.0, 2.0], 2), None);
        assert_eq!(spaces(&[1.0, 2.0, 3.0], 3), None);
        assert_eq!(spaces(&[1.0, 2.0], 4), None);
    }
}
