use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::systems::interaction::{OptionCycler, Selectable};

#[derive(Component, Clone, Copy)]
pub struct ShortcutKey(pub KeyCode);

pub fn collect_shortcut_commands<C: Component + Clone>(
    keyboard_input: &ButtonInput<KeyCode>,
    active_parents: &HashSet<Entity>,
    option_query: &Query<(&Selectable, &ShortcutKey, &C)>,
) -> Vec<(Entity, C)> {
    let mut selected_by_parent: HashMap<Entity, (usize, C)> = HashMap::new();
    for (selectable, shortcut, command) in option_query.iter() {
        if !active_parents.contains(&selectable.menu_entity) {
            continue;
        }
        if !keyboard_input.just_pressed(shortcut.0) {
            continue;
        }

        let parent = selectable.menu_entity;
        match selected_by_parent.get_mut(&parent) {
            Some((best_index, best_command)) => {
                if selectable.index < *best_index {
                    *best_index = selectable.index;
                    *best_command = (*command).clone();
                }
            }
            None => {
                selected_by_parent.insert(parent, (selectable.index, (*command).clone()));
            }
        }
    }

    let mut shortcuts: Vec<(Entity, usize, C)> = selected_by_parent
        .into_iter()
        .map(|(parent, (index, command))| (parent, index, command))
        .collect();
    shortcuts.sort_by_key(|(parent, index, _)| (parent.index(), *index));

    shortcuts
        .into_iter()
        .map(|(parent, _, command)| (parent, command))
        .collect()
}

pub fn sync_option_cycler_bounds(mut option_query: Query<&mut OptionCycler, With<Selectable>>) {
    for mut cycler in option_query.iter_mut() {
        cycler.at_min = false;
        cycler.at_max = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::interaction::Selectable;
    use bevy::ecs::system::SystemState;

    #[derive(Component, Clone, Debug, PartialEq, Eq)]
    struct DummyCommand(&'static str);

    #[test]
    fn collect_shortcuts_is_deterministic_per_parent_and_index() {
        let mut world = World::new();

        let menu_a = world.spawn_empty().id();
        let menu_b = world.spawn_empty().id();

        world.spawn((
            Selectable::new(menu_a, 2),
            ShortcutKey(KeyCode::KeyR),
            DummyCommand("a_high"),
        ));
        world.spawn((
            Selectable::new(menu_a, 0),
            ShortcutKey(KeyCode::KeyR),
            DummyCommand("a_low"),
        ));
        world.spawn((
            Selectable::new(menu_b, 1),
            ShortcutKey(KeyCode::KeyR),
            DummyCommand("b_only"),
        ));

        let mut state: SystemState<Query<(&Selectable, &ShortcutKey, &DummyCommand)>> =
            SystemState::new(&mut world);
        let query = state.get(&world);

        let mut keyboard = ButtonInput::<KeyCode>::default();
        keyboard.press(KeyCode::KeyR);

        let active_parents: HashSet<Entity> = [menu_b, menu_a].into_iter().collect();
        let commands = collect_shortcut_commands(&keyboard, &active_parents, &query);

        assert_eq!(commands.len(), 2);
        let by_parent: HashMap<Entity, &str> =
            commands.into_iter().map(|(menu, cmd)| (menu, cmd.0)).collect();
        assert_eq!(by_parent.get(&menu_a).copied(), Some("a_low"));
        assert_eq!(by_parent.get(&menu_b).copied(), Some("b_only"));
    }
}
