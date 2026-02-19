use std::collections::HashMap;

use bevy::prelude::*;
use bevy::ecs::query::QueryFilter;
use enum_map::{Enum, EnumArray};

use crate::systems::{
    audio::{DilatableAudio, TransientAudio, TransientAudioPallet},
    interaction::{Clickable, Selectable, SelectableMenu},
};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabBar {
    pub owner: Entity,
}

impl TabBar {
    pub const fn new(owner: Entity) -> Self {
        Self { owner }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabItem {
    pub index: usize,
}

#[derive(Component, Clone, Debug)]
pub struct TabActivationPolicy {
    pub activate_keys: Vec<KeyCode>,
}

impl Default for TabActivationPolicy {
    fn default() -> Self {
        Self {
            activate_keys: vec![KeyCode::Enter, KeyCode::Tab],
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TabBarState {
    pub active_index: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TabBarStateSync {
    #[default]
    Immediate,
    Explicit,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabChanged {
    pub tab_bar: Entity,
    pub index: usize,
}

pub fn keyboard_activation_target(
    keyboard_input: &ButtonInput<KeyCode>,
    activation_policy: Option<&TabActivationPolicy>,
    selected_index: usize,
    active_index: usize,
) -> Option<usize> {
    let activate_pressed = activation_policy.is_none_or(|policy| policy.activate_keys.is_empty())
        && (keyboard_input.just_pressed(KeyCode::Enter) || keyboard_input.just_pressed(KeyCode::Tab))
        || activation_policy.is_some_and(|policy| {
            policy
                .activate_keys
                .iter()
                .any(|&key| keyboard_input.just_pressed(key))
        });
    if activate_pressed && selected_index != active_index {
        Some(selected_index)
    } else {
        None
    }
}

fn tab_indices_by_bar(tab_item_query: &Query<&Selectable, With<TabItem>>) -> HashMap<Entity, Vec<usize>> {
    let mut indices_by_bar: HashMap<Entity, Vec<usize>> = HashMap::new();
    for selectable in tab_item_query.iter() {
        indices_by_bar
            .entry(selectable.menu_entity)
            .or_default()
            .push(selectable.index);
    }

    for indices in indices_by_bar.values_mut() {
        indices.sort_unstable();
        indices.dedup();
    }

    indices_by_bar
}

pub fn collect_clicked_tab_indices<A, F>(
    tab_item_query: &Query<(Entity, &TabItem, &Selectable, &Clickable<A>), F>,
) -> HashMap<Entity, usize>
where
    A: Copy + Send + Sync + 'static,
    F: QueryFilter,
{
    let mut clicked_by_tab_root: HashMap<Entity, (usize, u64)> = HashMap::new();
    for (entity, tab_item, selectable, clickable) in tab_item_query.iter() {
        if clickable.triggered {
            let root = selectable.menu_entity;
            let rank = entity.to_bits();
            match clicked_by_tab_root.get_mut(&root) {
                Some((selected_index, selected_rank)) => {
                    if rank >= *selected_rank {
                        *selected_index = tab_item.index;
                        *selected_rank = rank;
                    }
                }
                None => {
                    clicked_by_tab_root.insert(root, (tab_item.index, rank));
                }
            }
        }
    }
    clicked_by_tab_root
        .into_iter()
        .map(|(root, (index, _))| (root, index))
        .collect()
}

pub fn apply_tab_activation_with_audio<S>(
    tab_bar_entity: Entity,
    next_active_index: usize,
    tab_state: &mut TabBarState,
    click_pallet: Option<&TransientAudioPallet<S>>,
    commands: &mut Commands,
    audio_query: &mut Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: f32,
    click_sound: S,
    tab_changed: &mut MessageWriter<TabChanged>,
) where
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Send + Sync + Clone,
{
    if tab_state.active_index == next_active_index {
        return;
    }

    tab_state.active_index = next_active_index;
    if let Some(click_pallet) = click_pallet {
        TransientAudioPallet::play_transient_audio(
            tab_bar_entity,
            commands,
            click_pallet,
            click_sound,
            dilation,
            audio_query,
        );
    }
    tab_changed.write(TabChanged {
        tab_bar: tab_bar_entity,
        index: next_active_index,
    });
}

pub fn sanitize_tab_selection_indices(
    mut tab_query: Query<(Entity, &mut SelectableMenu), With<TabBar>>,
    tab_item_query: Query<&Selectable, With<TabItem>>,
) {
    let indices_by_bar = tab_indices_by_bar(&tab_item_query);

    for (tab_bar_entity, mut selectable_menu) in tab_query.iter_mut() {
        let Some(indices) = indices_by_bar.get(&tab_bar_entity) else {
            if selectable_menu.selected_index != 0 {
                selectable_menu.selected_index = 0;
            }
            continue;
        };

        if !indices.contains(&selectable_menu.selected_index) {
            selectable_menu.selected_index = indices[0];
        }
    }
}

pub fn sync_tab_bar_state(
    tab_query: Query<(Entity, &SelectableMenu, Option<&TabBarStateSync>), With<TabBar>>,
    mut tab_state_query: Query<&mut TabBarState, With<TabBar>>,
    mut tab_changed: MessageWriter<TabChanged>,
) {
    for (tab_bar_entity, selectable_menu, sync_mode) in tab_query.iter() {
        if sync_mode.is_some_and(|mode| *mode == TabBarStateSync::Explicit) {
            continue;
        }
        let Ok(mut state) = tab_state_query.get_mut(tab_bar_entity) else {
            continue;
        };

        if state.active_index != selectable_menu.selected_index {
            state.active_index = selectable_menu.selected_index;
            tab_changed.write(TabChanged {
                tab_bar: tab_bar_entity,
                index: state.active_index,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::interaction::Selectable;
    use bevy::ecs::system::SystemState;

    #[test]
    fn collect_clicked_tab_indices_prefers_highest_entity_rank_per_tab_root() {
        let mut world = World::new();
        let tab_root = world.spawn_empty().id();

        let mut clickable_a = Clickable::new(vec![0u8]);
        clickable_a.triggered = true;
        let first = world
            .spawn((
            TabItem { index: 0 },
            Selectable::new(tab_root, 0),
            clickable_a,
            ))
            .id();

        let mut clickable_b = Clickable::new(vec![0u8]);
        clickable_b.triggered = true;
        let winner = world
            .spawn((
                TabItem { index: 1 },
                Selectable::new(tab_root, 1),
                clickable_b,
            ))
            .id();

        let mut state: SystemState<Query<(Entity, &TabItem, &Selectable, &Clickable<u8>)>> =
            SystemState::new(&mut world);
        let query = state.get(&world);

        let clicked = collect_clicked_tab_indices(&query);
        let expected_index = if winner.to_bits() >= first.to_bits() { 1 } else { 0 };
        assert_eq!(clicked.get(&tab_root).copied(), Some(expected_index));

        // Current implementation resolves ties by entity rank (`to_bits`).
    }
}
