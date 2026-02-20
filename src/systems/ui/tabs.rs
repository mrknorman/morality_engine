//! Reusable tab selection/activation primitives.
//!
//! Tabs expose a thin data model (`TabBar`, `TabItem`, `TabBarState`) and
//! deterministic helpers for click and keyboard activation. Menu-specific
//! focus policy should remain in composition modules.
use std::collections::HashMap;

use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;
use enum_map::{Enum, EnumArray};

use crate::systems::{
    audio::{DilatableAudio, TransientAudio, TransientAudioPallet},
    interaction::{Clickable, Selectable, SelectableMenu},
};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[require(SelectableMenu, TabBarState, TabActivationPolicy, TabBarStateSync)]
pub struct TabBar {
    /// Owner entity for layer/gating arbitration.
    pub owner: Entity,
}

impl TabBar {
    pub const fn new(owner: Entity) -> Self {
        Self { owner }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabItem {
    /// Logical tab index within a tab bar.
    pub index: usize,
}

#[derive(Component, Clone, Debug)]
pub struct TabActivationPolicy {
    /// Keys that confirm activation of the currently selected tab.
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
    /// Currently active tab index.
    pub active_index: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TabBarStateSync {
    /// Mirror `SelectableMenu.selected_index` into `TabBarState` each frame.
    #[default]
    Immediate,
    /// Keep `TabBarState` explicit; composition systems drive updates manually.
    Explicit,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabChanged {
    /// Tab bar entity that changed.
    pub tab_bar: Entity,
    /// Newly active tab index.
    pub index: usize,
}

/// Resolves a keyboard activation target for the selected tab index.
///
/// Returns `Some(index)` only when an activation key was pressed and the
/// selected tab differs from the currently active tab.
pub fn keyboard_activation_target(
    keyboard_input: &ButtonInput<KeyCode>,
    activation_policy: Option<&TabActivationPolicy>,
    selected_index: usize,
    active_index: usize,
) -> Option<usize> {
    let activate_pressed = activation_policy.is_none_or(|policy| policy.activate_keys.is_empty())
        && (keyboard_input.just_pressed(KeyCode::Enter)
            || keyboard_input.just_pressed(KeyCode::Tab))
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

fn tab_indices_by_bar(
    tab_item_query: &Query<&Selectable, With<TabItem>>,
) -> HashMap<Entity, Vec<usize>> {
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

/// Collects clicked tab indices keyed by tab-root entity.
///
/// When multiple clicked tab entities exist for one root in a frame, the
/// highest entity-rank winner is chosen deterministically.
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

/// Applies tab activation transition and emits optional click audio.
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

/// Clamps tab-bar selected indices to the set of known tab items.
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

/// Synchronizes tab active state from `SelectableMenu` for `Immediate` bars.
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
        let expected_index = if winner.to_bits() >= first.to_bits() {
            1
        } else {
            0
        };
        assert_eq!(clicked.get(&tab_root).copied(), Some(expected_index));

        // Current implementation resolves ties by entity rank (`to_bits`).
    }

    #[test]
    fn tab_bar_insertion_adds_required_components() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let tab_bar = world.spawn(TabBar::new(owner)).id();

        assert!(world.entity(tab_bar).get::<SelectableMenu>().is_some());
        assert!(world.entity(tab_bar).get::<TabBarState>().is_some());
        assert!(world.entity(tab_bar).get::<TabActivationPolicy>().is_some());
        assert!(world.entity(tab_bar).get::<TabBarStateSync>().is_some());
    }

    #[test]
    fn sync_tab_bar_state_updates_active_index_and_emits_message() {
        let mut app = App::new();
        app.add_message::<TabChanged>();
        let owner = app.world_mut().spawn_empty().id();
        let tab_bar = app
            .world_mut()
            .spawn((
                TabBar::new(owner),
                SelectableMenu::new(2, vec![], vec![], vec![], true),
                TabBarState { active_index: 0 },
            ))
            .id();

        app.add_systems(Update, sync_tab_bar_state);
        app.update();

        let state = app
            .world()
            .get::<TabBarState>(tab_bar)
            .copied()
            .expect("tab state");
        assert_eq!(state.active_index, 2);

        let mut reader = app
            .world_mut()
            .resource_mut::<Messages<TabChanged>>()
            .get_cursor();
        let messages: Vec<TabChanged> = reader
            .read(app.world().resource::<Messages<TabChanged>>())
            .copied()
            .collect();
        assert_eq!(messages, vec![TabChanged { tab_bar, index: 2 }]);
    }

    #[test]
    fn sync_tab_bar_state_respects_explicit_mode() {
        let mut app = App::new();
        app.add_message::<TabChanged>();
        let owner = app.world_mut().spawn_empty().id();
        let tab_bar = app
            .world_mut()
            .spawn((
                TabBar::new(owner),
                SelectableMenu::new(1, vec![], vec![], vec![], true),
                TabBarState { active_index: 0 },
                TabBarStateSync::Explicit,
            ))
            .id();

        app.add_systems(Update, sync_tab_bar_state);
        app.update();

        let state = app
            .world()
            .get::<TabBarState>(tab_bar)
            .copied()
            .expect("tab state");
        assert_eq!(state.active_index, 0);

        let mut reader = app
            .world_mut()
            .resource_mut::<Messages<TabChanged>>()
            .get_cursor();
        let messages: Vec<TabChanged> = reader
            .read(app.world().resource::<Messages<TabChanged>>())
            .copied()
            .collect();
        assert!(messages.is_empty());
    }
}
