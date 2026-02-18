use bevy::prelude::*;

use crate::systems::interaction::SelectableMenu;

use super::defs::MenuRoot;

#[derive(Resource, Debug, Default)]
pub struct MenuDropdownState {
    pub open_menu: Option<Entity>,
    pub suppress_toggle_once: bool,
}

pub fn any_open<D: Component>(dropdown_query: &Query<&Visibility, With<D>>) -> bool {
    dropdown_query
        .iter()
        .any(|visibility| *visibility == Visibility::Visible)
}

pub fn open_for_menu<D: Component>(
    menu_entity: Entity,
    selected_index: usize,
    dropdown_state: &mut MenuDropdownState,
    dropdown_query: &mut Query<(Entity, &ChildOf, &mut Visibility), With<D>>,
    dropdown_menu_query: &mut Query<&mut SelectableMenu, (With<D>, Without<MenuRoot>)>,
) {
    let mut found = false;
    for (dropdown_entity, parent, mut visibility) in dropdown_query.iter_mut() {
        if parent.parent() == menu_entity {
            *visibility = Visibility::Visible;
            found = true;
            if let Ok(mut dropdown_menu) = dropdown_menu_query.get_mut(dropdown_entity) {
                dropdown_menu.selected_index = selected_index;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    if found {
        dropdown_state.open_menu = Some(menu_entity);
    } else if dropdown_state.open_menu == Some(menu_entity) {
        dropdown_state.open_menu = None;
    }
}

pub fn close_all<D: Component>(
    dropdown_state: &mut MenuDropdownState,
    dropdown_query: &mut Query<(Entity, &ChildOf, &mut Visibility), With<D>>,
) {
    for (_, _, mut visibility) in dropdown_query.iter_mut() {
        *visibility = Visibility::Hidden;
    }
    dropdown_state.open_menu = None;
}

pub fn close_for_menu<D: Component>(
    menu_entity: Entity,
    dropdown_state: &mut MenuDropdownState,
    dropdown_query: &mut Query<(Entity, &ChildOf, &mut Visibility), With<D>>,
) {
    for (_, parent, mut visibility) in dropdown_query.iter_mut() {
        if parent.parent() == menu_entity {
            *visibility = Visibility::Hidden;
        }
    }
    if dropdown_state.open_menu == Some(menu_entity) {
        dropdown_state.open_menu = None;
    }
}

pub fn enforce_single_visible_layer<D: Component>(
    dropdown_state: &mut MenuDropdownState,
    menu_root_query: &Query<Entity, With<MenuRoot>>,
    modal_open: bool,
    dropdown_query: &mut Query<(Entity, &ChildOf, &mut Visibility), With<D>>,
) {
    if dropdown_state
        .open_menu
        .as_ref()
        .is_some_and(|menu_entity| menu_root_query.get(*menu_entity).is_err())
    {
        dropdown_state.open_menu = None;
    }

    let mut visible_dropdowns: Vec<(Entity, Entity)> = Vec::new();
    for (dropdown_entity, parent, visibility) in dropdown_query.iter() {
        if *visibility == Visibility::Visible {
            visible_dropdowns.push((dropdown_entity, parent.parent()));
        }
    }

    if modal_open {
        if !visible_dropdowns.is_empty() {
            for (_, _, mut visibility) in dropdown_query.iter_mut() {
                *visibility = Visibility::Hidden;
            }
        }
        dropdown_state.open_menu = None;
        return;
    }

    if visible_dropdowns.is_empty() {
        dropdown_state.open_menu = None;
        return;
    }

    let keep_dropdown = if visible_dropdowns.len() == 1 {
        visible_dropdowns[0].0
    } else if let Some(open_menu) = dropdown_state.open_menu {
        visible_dropdowns
            .iter()
            .find_map(|(dropdown_entity, parent_menu)| {
                if *parent_menu == open_menu {
                    Some(*dropdown_entity)
                } else {
                    None
                }
            })
            .unwrap_or(visible_dropdowns[0].0)
    } else {
        visible_dropdowns[0].0
    };

    let mut keep_parent_menu = None;
    for (dropdown_entity, parent, mut visibility) in dropdown_query.iter_mut() {
        if dropdown_entity == keep_dropdown {
            keep_parent_menu = Some(parent.parent());
            *visibility = Visibility::Visible;
        } else if *visibility == Visibility::Visible {
            *visibility = Visibility::Hidden;
        }
    }

    dropdown_state.open_menu = keep_parent_menu;
}
