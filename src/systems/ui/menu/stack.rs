use bevy::prelude::*;

use super::defs::MenuRoot;

#[derive(Resource, Debug, Default)]
pub struct MenuNavigationState {
    pub exit_prompt_target_menu: Option<Entity>,
    pub exit_prompt_closes_menu_system: bool,
    pub pending_exit_menu: Option<Entity>,
    pub pending_exit_closes_menu_system: bool,
}

pub fn clear_stale_menu_targets(
    navigation_state: &mut MenuNavigationState,
    menu_root_query: &Query<Entity, With<MenuRoot>>,
) {
    let clear_if_stale = |slot: &mut Option<Entity>| {
        if slot
            .as_ref()
            .is_some_and(|menu_entity| menu_root_query.get(*menu_entity).is_err())
        {
            *slot = None;
        }
    };

    clear_if_stale(&mut navigation_state.exit_prompt_target_menu);
    clear_if_stale(&mut navigation_state.pending_exit_menu);
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::SystemState;

    use super::*;
    use crate::systems::ui::menu::{MenuHost, MenuPage};

    #[test]
    fn clear_stale_menu_targets_removes_missing_and_keeps_live() {
        let mut world = World::new();

        let live_menu = world
            .spawn((
                MenuRoot {
                    host: MenuHost::Pause,
                    gate: crate::systems::interaction::UiInputPolicy::CapturedOnly,
                },
                super::super::MenuStack::new(MenuPage::PauseRoot),
            ))
            .id();
        let stale_menu = Entity::from_bits(live_menu.to_bits().saturating_add(10_000));

        let mut navigation_state = MenuNavigationState {
            exit_prompt_target_menu: Some(stale_menu),
            exit_prompt_closes_menu_system: true,
            pending_exit_menu: Some(live_menu),
            pending_exit_closes_menu_system: false,
        };

        let mut query_state: SystemState<Query<Entity, With<MenuRoot>>> =
            SystemState::new(&mut world);
        let menu_root_query = query_state.get(&world);

        clear_stale_menu_targets(&mut navigation_state, &menu_root_query);

        assert_eq!(navigation_state.exit_prompt_target_menu, None);
        assert_eq!(navigation_state.pending_exit_menu, Some(live_menu));
        assert!(navigation_state.exit_prompt_closes_menu_system);
        assert!(!navigation_state.pending_exit_closes_menu_system);
    }
}
