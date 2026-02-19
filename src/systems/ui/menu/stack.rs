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
