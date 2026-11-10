//! Which installed application a window belongs to, and the icons of both.

use anti_webui::shell::icon_path;
use anti_webui::shell::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

/// The application a window's `app_id` stands for: its desktop entry by id, or by the last part
/// of a reverse-DNS id, or by name.
pub fn app_for(app_id: &str) -> Option<Application> {
    if app_id.is_empty() {
        return None;
    }
    let all = apps().all();
    let lower = app_id.to_lowercase();
    let last = |id: &str| id.rsplit('.').next().unwrap_or(id).to_lowercase();
    all.iter()
        .find(|a| bare(&a.id) == app_id)
        .or_else(|| all.iter().find(|a| bare(&a.id).to_lowercase() == lower))
        .or_else(|| all.iter().find(|a| last(bare(&a.id)) == last(&lower)))
        .or_else(|| all.iter().find(|a| a.name.to_lowercase() == lower))
        .cloned()
}

/// A desktop entry id without `.desktop`.
pub fn bare(id: &str) -> &str {
    id.strip_suffix(".desktop").unwrap_or(id)
}

/// An installed application by its desktop entry id, with or without `.desktop`.
pub fn app_by_id(id: &str) -> Option<Application> {
    let all = apps();
    all.get(id)
        .or_else(|| all.get(&format!("{}.desktop", bare(id))))
}

thread_local! {
    static ICONS: RefCell<HashMap<(String, u32), Option<PathBuf>>> = RefCell::default();
}

/// The file of an icon by its name, looked up in the icon theme once.
pub fn icon_file(name: &str, size: u32) -> Option<PathBuf> {
    ICONS.with(|icons| {
        icons
            .borrow_mut()
            .entry((name.to_owned(), size))
            .or_insert_with(|| icon_path(name, size))
            .clone()
    })
}

/// The icon of an application.
pub fn app_icon(app: &Application, size: u32) -> Option<PathBuf> {
    icon_file(app.icon.as_deref()?, size)
}

/// The icon of a window with this `app_id`: its application's, else one named like the `app_id`.
pub fn app_id_icon(app_id: &str, size: u32) -> Option<PathBuf> {
    app_for(app_id)
        .and_then(|a| app_icon(&a, size))
        .or_else(|| icon_file(app_id, size))
        .or_else(|| icon_file(&app_id.to_lowercase(), size))
}
