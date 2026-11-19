use std::rc::Rc;

use crate::placement::BarPlacement;
use anti_webui::{core::view::View, shell::Output};

pub type Part = Rc<dyn Fn(&Cx) -> View>;

// TODO: to be completed on step 6: motion.rs
pub struct Motion;
// TODO: to be completed on step 4
#[derive(Clone)]
pub struct IslandState;

#[non_exhaustive]
#[derive(Clone)]
pub struct Cx {
    pub output: Output,
    pub placement: BarPlacement,
    pub motion: Rc<Motion>,
    pub island: IslandState,
}

impl Cx {
    pub fn new(output: Output, placement: BarPlacement) -> Self {
        Self {
            output,
            placement,
            motion: Rc::new(Motion),
            island: IslandState,
        }
    }
}
