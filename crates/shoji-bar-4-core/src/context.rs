use std::rc::Rc;

use crate::placement::BarPlacement;
use anti_webui::{
    core::{
        reactive::{Memo, Signal, memo, signal},
        view::View,
    },
    shell::Output,
};

pub type Part = Rc<dyn Fn(&Cx) -> View>;

// TODO: to be completed on step 6: motion.rs
pub struct Motion;

#[derive(Clone, Copy)]
pub struct IslandState {
    pub hovered: Signal<bool>,
    pub active: Memo<bool>, // highlighted, e.g. while its panel is open
    pub index: usize,       // position in the bar, for staggered entrances
}

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
            island: IslandState {
                hovered: signal(false),
                active: memo(|| false),
                index: 0,
            },
        }
    }

    pub fn with_island(&self, island: IslandState) -> Cx {
        Cx {
            island,
            ..self.clone()
        }
    }
}
