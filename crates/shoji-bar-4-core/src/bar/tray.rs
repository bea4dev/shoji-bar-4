use crate::appinfo::tray_icon_image;
use crate::bar::island::{Island, VERTICAL_ISLAND, bar_island};
use crate::placement::{BarPlacement, PanelSide};
use crate::theme::{TextStyle, hex_of, target_scheme};
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui::shell::{MenuToggle, TrayMenuItem};
use anti_webui_md3e::components::{MenuItem, MenuStyle, menu_item, styled_menu};

const TRAY_ICON: f32 = 24.0;

/// The icons of the tray.
fn tray_island(at: BarPlacement) -> Show {
    show(
        || !tray().items().is_empty(),
        move || {
            let content = if at.is_vertical() {
                // Center the end icon's circle in the island's semicircle.
                bar_island(at).py((VERTICAL_ISLAND - TRAY_ICON) / 2.0)
            } else {
                bar_island(at).px(5.0_f32)
            };
            content.gap(2.0_f32).child(
                for_each(
                    || tray().items(),
                    |item| item.key(),
                    move |item| tray_icon(item, at),
                )
                .enter(|s| s.opacity(0.0_f32).scale(0.4_f32))
                .exit(|s| s.opacity(0.0_f32).scale(0.4_f32))
                .transition(overshoot(300)),
            )
        },
    )
    .enter(|s| s.opacity(0.0_f32).scale(0.8_f32))
    .exit(|s| s.opacity(0.0_f32).scale(0.8_f32))
    .transition(overshoot(300))
}

fn tray_icon(item: TrayItem, at: BarPlacement) -> Div {
    let open = signal(false);
    let key = item.key();
    let now = move || tray().items().into_iter().find(|i| i.key() == key);
    let shown = now.clone();
    let icon = memo(move || now().map(|item| item.icon));
    // A symbolic icon is a picture made in a color: in the one the theme is going to, made again
    // when that changes rather than at each step of the way there.
    let symbolic_color = memo(|| hex_of(target_scheme().on_surface));
    let clicked = item.clone();
    pressable(
        div()
            .size(TRAY_ICON)
            .rounded(TRAY_ICON / 2.0)
            .flex()
            .items_center()
            .justify_center()
            .child(dyn_view(move || {
                let icon = icon.get();
                let picture = icon
                    .as_ref()
                    .and_then(|icon| tray_icon_image(icon, 48, symbolic_color.get()));
                if let Some(picture) = picture {
                    return img(picture)
                        .size(16.0)
                        .object_fit(ObjectFit::Contain)
                        .into_view();
                }
                match icon.and_then(|icon| icon.best(48)) {
                    Some(picture) => bitmap_view(move || Some(picture.clone()))
                        .size(16.0)
                        .into_view(),
                    None => txt(TextStyle::LABEL_MEDIUM, {
                        let shown = shown.clone();
                        move || {
                            shown()
                                .and_then(|i| i.title.chars().next())
                                .map(String::from)
                                .unwrap_or_default()
                        }
                    })
                    .into_view(),
                }
            }))
            .on_pointer_down(move |e| match e.button {
                Some(PointerButton::Secondary) if clicked.has_menu => open.set(true),
                Some(PointerButton::Primary) if clicked.item_is_menu => open.set(true),
                Some(PointerButton::Primary) => {
                    clicked.activate(0, 0);
                }
                _ => {}
            })
            .child(tray_menu(&item, open, at)),
        |c| c.on_surface,
    )
}

/// The item's menu, in the theme's menus: the separators part it into groups. It follows the item's
/// menu as that changes.
fn tray_menu(item: &TrayItem, open: Signal<bool>, at: BarPlacement) -> View {
    let item = item.clone();
    effect({
        let item = item.clone();
        move || {
            if open.get() {
                // Programs fill some menus only when told they are about to be shown.
                let _ = item.menu_about_to_show(0);
            }
        }
    });
    let placement = match at.side() {
        PanelSide::Below => Placement::BelowCenter,
        PanelSide::Left => Placement::RightStart,
        PanelSide::Right => Placement::LeftStart,
    };
    dyn_view(move || {
        styled_menu(
            move || open.get(),
            move || open.set(false),
            tray_groups(&item, &item.menu()),
            MenuStyle::new().placement(placement),
        )
    })
}

fn tray_groups(item: &TrayItem, list: &[TrayMenuItem]) -> Vec<Vec<MenuItem>> {
    let mut groups = vec![Vec::new()];
    for m in list.iter().filter(|m| m.visible) {
        if m.separator {
            groups.push(Vec::new());
            continue;
        }
        let mut row = menu_item(m.label.clone());
        if !m.children.is_empty() {
            row = row.submenu(tray_groups(item, &m.children));
        } else {
            if let Some(MenuToggle::Check(on) | MenuToggle::Radio(on)) = m.toggle {
                row = row.selected_when(move || on);
            }
            let (item, id) = (item.clone(), m.id);
            row = row.on_click(move || {
                let _ = item.menu_clicked(id);
            });
        }
        if !m.enabled {
            row = row.disabled_when(|| true);
        }
        if let Some(keys) = &m.shortcut {
            row = row.trailing(keys.clone());
        }
        groups.last_mut().expect("never empty").push(row);
    }
    groups.retain(|g| !g.is_empty());
    groups
}

/// The icons of the tray, as an island of the bar.
pub fn island() -> Island {
    Island::new("tray", |cx| tray_island(cx.placement).into_view())
}
