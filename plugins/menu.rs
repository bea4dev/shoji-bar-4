//! The palettes in the quick settings panel: the "Dark theme" tile says which palette is in use and
//! whether the bar has its pane (a click on it switches the pane), and its chevron opens a page of
//! the palettes with a tick on the one in use. What the palettes are, and the state, is in
//! `colour.rs`.

use super::colour;
use anti_webui::prelude::*;
use anti_webui_md3e::icons;
use shoji_core::ui::sym;
use shoji_default::qs_pages::{header, item, scrolling, subheading};
use shoji_default::quick_settings::section;
use shoji_default::Ui;

pub fn install(ui: &mut Ui) {
    shoji_default::quick_settings::set_palettes_page(page);
    shoji_default::quick_settings::set_dark_tile_text(|| {
        let on = colour::bar_on().get();
        Some((colour::label().get(), if on { "Bar ON" } else { "Bar OFF" }.to_owned(), on))
    });
    shoji_default::quick_settings::set_dark_tile_click(colour::toggle_bar);
    ui.on_start(colour::restore);
}

/// The page of palettes, which slides in from the "Dark theme" tile like Wi-Fi's: a line for each
/// palette, with a tick on the one in use.
fn page() -> Div {
    let mut list = div().flex().flex_col().gap(2.0_f32);
    for entry in colour::list() {
        let name = entry.name.clone();
        let chosen = memo({
            let name = name.clone();
            move || colour::current().get() == name
        });
        let symbol = match name.as_str() {
            "dark" => icons::DARK_MODE,
            "light" => icons::LIGHT_MODE,
            _ => icons::PALETTE,
        };
        let status = "Palette";
        list = list.child(
            item(
                move || symbol,
                {
                    let label = entry.label.clone();
                    move || label.clone()
                },
                move || if chosen.get() { "In use".to_owned() } else { status.to_owned() },
                move || chosen.get(),
                div().child(show(move || chosen.get(), || {
                    sym(icons::CHECK, 20.0, |c| c.primary).mr(8.0)
                })),
            )
            .on_click(move |_| colour::choose(&entry)),
        );
    }
    div()
        .flex()
        .flex_col()
        .gap(10.0_f32)
        .child(header("Theme", div()))
        .child(section().child(subheading("Palettes")).child(scrolling(list)))
}
