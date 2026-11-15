use std::rc::Rc;

use anti_webui::{core::view::View, shell::Output};

use crate::placement::BarPlacement;

#[derive(Clone)]
pub struct Island {
    pub id: String,
    pub build: Rc<dyn Fn(Output, BarPlacement) -> View>,
}
