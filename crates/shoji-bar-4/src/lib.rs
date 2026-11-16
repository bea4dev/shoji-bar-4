use shoji_bar_4_core::bar::Bar;

pub fn empty_bar() -> Bar {
    Bar::new().left(vec![]).center(vec![]).right(vec![])
}
