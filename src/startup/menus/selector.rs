use std::collections::HashSet;

use bevy::prelude::*;

use crate::systems::interaction::{interaction_gate_allows, OptionCycler, Selectable};

use super::defs::{MenuCommand, MenuOptionCommand, MenuRoot, MenuStack};

#[derive(Component, Clone, Copy)]
pub struct MenuOptionShortcut(pub KeyCode);

pub fn collect_shortcut_commands(
    keyboard_input: &ButtonInput<KeyCode>,
    interaction_captured: bool,
    menu_query: &Query<(Entity, &MenuStack, &MenuRoot)>,
    option_query: &Query<(&Selectable, &MenuOptionShortcut, &MenuOptionCommand)>,
) -> Vec<(Entity, MenuCommand)> {
    let mut active_menus = HashSet::new();
    for (menu_entity, _, menu_root) in menu_query.iter() {
        if interaction_gate_allows(Some(&menu_root.gate), interaction_captured) {
            active_menus.insert(menu_entity);
        }
    }

    let mut seen_menus = HashSet::new();
    let mut shortcuts = Vec::new();
    for (selectable, shortcut, option_command) in option_query.iter() {
        if !active_menus.contains(&selectable.menu_entity) {
            continue;
        }
        if !keyboard_input.just_pressed(shortcut.0) {
            continue;
        }
        if !seen_menus.insert(selectable.menu_entity) {
            continue;
        }

        shortcuts.push((selectable.menu_entity, option_command.0.clone()));
    }
    shortcuts
}

pub fn sync_option_cycler_bounds(mut option_query: Query<(&MenuOptionCommand, &mut OptionCycler)>) {
    for (_, mut cycler) in option_query.iter_mut() {
        cycler.at_min = false;
        cycler.at_max = false;
    }
}
