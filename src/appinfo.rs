//! Which installed application a window belongs to, and the icons of both.

use anti_webui::shell::TrayIcon;
use anti_webui::shell::icon_path;
use anti_webui::shell::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

/// A tray item's named icon: an absolute file, an application-supplied directory,
/// or the desktop icon theme. Per-item directories must not share the name-only cache.
pub fn tray_icon_file(icon: &TrayIcon, size: u32) -> Option<PathBuf> {
    let name = icon.name.as_deref().filter(|name| !name.is_empty())?;
    let path = Path::new(name);
    if path.is_absolute() {
        return path.is_file().then(|| path.to_owned());
    }
    if let Some(directory) = icon.theme_path.as_deref().filter(|path| !path.is_empty()) {
        let directory = Path::new(directory);
        let path = directory.join(name);
        if path.is_file() {
            return Some(path);
        }
        for extension in ["png", "svg", "xpm"] {
            let path = directory.join(format!("{name}.{extension}"));
            if path.is_file() {
                return Some(path);
            }
        }
    }
    icon_file(name, size)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn tray_names_resolve_absolute_files_and_private_directory_icons() {
        let directory = tempfile::tempdir().unwrap();
        let png = directory.path().join("tray.png");
        let svg = directory.path().join("tray-symbolic.svg");
        fs::write(&png, []).unwrap();
        fs::write(&svg, []).unwrap();
        let mut icon = TrayIcon {
            name: Some(png.to_str().unwrap().to_owned()),
            ..Default::default()
        };
        assert_eq!(tray_icon_file(&icon, 48), Some(png.clone()));
        icon.theme_path = Some(directory.path().to_str().unwrap().to_owned());
        icon.name = Some("tray".to_owned());
        assert_eq!(tray_icon_file(&icon, 48), Some(png));
        icon.name = Some("tray-symbolic.svg".to_owned());
        assert_eq!(tray_icon_file(&icon, 48), Some(svg));
    }

    #[test]
    fn tray_directory_changes_and_new_files_are_not_hidden_by_the_cache() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        // Use a unique icon name so the test cannot pick an installed theme icon.
        let name = format!(
            "shoji-tray-{}",
            first.path().file_name().unwrap().to_str().unwrap()
        );
        let first_icon = first.path().join(format!("{name}.png"));
        let second_icon = second.path().join(format!("{name}.png"));
        fs::write(&first_icon, []).unwrap();
        let mut icon = TrayIcon {
            name: Some(name),
            theme_path: Some(first.path().to_str().unwrap().to_owned()),
            ..Default::default()
        };
        assert_eq!(tray_icon_file(&icon, 48), Some(first_icon));
        icon.theme_path = Some(second.path().to_str().unwrap().to_owned());
        assert_eq!(tray_icon_file(&icon, 48), None);
        fs::write(&second_icon, []).unwrap();
        assert_eq!(tray_icon_file(&icon, 48), Some(second_icon));
    }
}
