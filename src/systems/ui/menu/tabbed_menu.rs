use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use super::{
    defs::MenuRoot,
    tabbed_focus::{resolve_tabbed_focus, TabbedFocusInputs},
};
pub use super::tabbed_focus::TabbedMenuFocus;
use crate::{
    data::states::PauseState,
    startup::cursor::CustomCursor,
    systems::{
        audio::{DilatableAudio, TransientAudio, TransientAudioPallet},
        interaction::{
            interaction_gate_allows_for_owner, Clickable, Hoverable, InteractionCapture,
            InteractionCaptureOwner, InteractionGate, InteractionVisualState, Selectable,
            SelectableClickActivation, SelectableMenu, SystemMenuActions, SystemMenuSounds,
        },
        time::Dilation,
        ui::{
            layer::{self, UiLayer, UiLayerKind},
            tabs::{self, TabBar, TabBarState, TabChanged, TabItem},
        },
    },
};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabbedMenuConfig {
    pub top_option_count: usize,
    pub footer_start_index: usize,
    pub footer_count: usize,
}

impl TabbedMenuConfig {
    pub const fn new(top_option_count: usize, footer_start_index: usize, footer_count: usize) -> Self {
        Self {
            top_option_count,
            footer_start_index,
            footer_count,
        }
    }

    pub fn footer_contains(self, index: usize) -> bool {
        index >= self.footer_start_index && index < self.footer_start_index + self.footer_count
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabbedMenuOption {
    pub owner: Entity,
}

impl TabbedMenuOption {
    pub const fn new(owner: Entity) -> Self {
        Self { owner }
    }
}

#[derive(Resource, Default)]
pub struct TabbedMenuFocusState {
    pub by_menu: HashMap<Entity, TabbedMenuFocus>,
    previous_selection_by_menu: HashMap<Entity, usize>,
    option_lock_by_menu: HashMap<Entity, usize>,
}

impl TabbedMenuFocusState {
    pub fn is_tabs_focused(&self, menu_entity: Entity) -> bool {
        self.by_menu
            .get(&menu_entity)
            .is_some_and(|focus| *focus == TabbedMenuFocus::Tabs)
    }

    fn previous_selection(&self, menu_entity: Entity, fallback: usize) -> usize {
        self.previous_selection_by_menu
            .get(&menu_entity)
            .copied()
            .unwrap_or(fallback)
    }

    fn set_previous_selection(&mut self, menu_entity: Entity, selected_index: usize) {
        self.previous_selection_by_menu
            .insert(menu_entity, selected_index);
    }

    pub fn option_lock(&self, menu_entity: Entity) -> Option<usize> {
        self.option_lock_by_menu.get(&menu_entity).copied()
    }

    pub fn set_option_lock(&mut self, menu_entity: Entity, selected_index: Option<usize>) {
        if let Some(selected_index) = selected_index {
            self.option_lock_by_menu.insert(menu_entity, selected_index);
        } else {
            self.option_lock_by_menu.remove(&menu_entity);
        }
    }
}

pub fn cleanup_tabbed_menu_state(
    mut focus_state: ResMut<TabbedMenuFocusState>,
    tab_query: Query<&TabBar, With<TabbedMenuConfig>>,
    mut menu_query: Query<&mut SelectableMenu, With<MenuRoot>>,
) {
    let mut live_menus = HashSet::new();
    for tab_bar in tab_query.iter() {
        live_menus.insert(tab_bar.owner);
    }

    let mut stale_menus = HashSet::new();
    for menu_entity in focus_state.by_menu.keys() {
        if !live_menus.contains(menu_entity) {
            stale_menus.insert(*menu_entity);
        }
    }
    for menu_entity in focus_state.previous_selection_by_menu.keys() {
        if !live_menus.contains(menu_entity) {
            stale_menus.insert(*menu_entity);
        }
    }
    for menu_entity in focus_state.option_lock_by_menu.keys() {
        if !live_menus.contains(menu_entity) {
            stale_menus.insert(*menu_entity);
        }
    }

    for menu_entity in stale_menus {
        let Ok(mut menu) = menu_query.get_mut(menu_entity) else {
            continue;
        };
        menu.wrap = true;
        menu.click_activation = SelectableClickActivation::SelectedOnAnyClick;
        menu.up_keys = vec![KeyCode::ArrowUp];
        menu.down_keys = vec![KeyCode::ArrowDown];
        menu.activate_keys = vec![KeyCode::Enter];
    }

    focus_state
        .by_menu
        .retain(|menu_entity, _| live_menus.contains(menu_entity));
    focus_state
        .previous_selection_by_menu
        .retain(|menu_entity, _| live_menus.contains(menu_entity));
    focus_state
        .option_lock_by_menu
        .retain(|menu_entity, _| live_menus.contains(menu_entity));
}

pub fn sync_tabbed_menu_focus(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor: Res<CustomCursor>,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    ui_layer_query: Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
    tab_item_query: Query<
        (
            Entity,
            &TabItem,
            &Selectable,
            &Hoverable,
            &Clickable<SystemMenuActions>,
        ),
        Without<TabbedMenuOption>,
    >,
    option_query: Query<
        (
            Entity,
            &Selectable,
            &TabbedMenuOption,
            &Hoverable,
            &Clickable<SystemMenuActions>,
        ),
        Without<TabItem>,
    >,
    mut previous_cursor_position: Local<Option<Vec2>>,
    mut focus_state: ResMut<TabbedMenuFocusState>,
    mut menu_query: Query<
        &mut SelectableMenu,
        (With<MenuRoot>, Without<TabbedMenuConfig>),
    >,
    mut tab_queries: ParamSet<(
        Query<
            (
                Entity,
                &TabBar,
                &TabBarState,
                &SelectableMenu,
                &TabbedMenuConfig,
                Option<&InteractionGate>,
            ),
            With<TabbedMenuConfig>,
        >,
        Query<&mut SelectableMenu, With<TabbedMenuConfig>>,
        Query<&mut TabBarState, With<TabbedMenuConfig>>,
    )>,
    mut tab_changed: MessageWriter<TabChanged>,
) {
    // Query contract:
    // - `tab_item_query` and `option_query` are disjoint via `Without` filters.
    // - Base-menu mutable `SelectableMenu` access excludes tab roots, while tab-root
    //   mutable `SelectableMenu` access lives in the ParamSet tab query branch.
    // This keeps focus arbitration B0001-safe as tabbed layers evolve.
    let pause_state = pause_state.as_ref();
    let active_layers =
        layer::active_layers_by_owner_scoped(pause_state, &capture_query, &ui_layer_query);
    let tab_pressed = keyboard_input.just_pressed(KeyCode::Tab);
    let up_pressed = keyboard_input.just_pressed(KeyCode::ArrowUp);
    let down_pressed = keyboard_input.just_pressed(KeyCode::ArrowDown);
    let left_pressed = keyboard_input.just_pressed(KeyCode::ArrowLeft);
    let right_pressed = keyboard_input.just_pressed(KeyCode::ArrowRight);
    let keyboard_focus_navigation =
        tab_pressed || up_pressed || down_pressed || left_pressed || right_pressed;
    let mouse_moved = match (*previous_cursor_position, cursor.position) {
        (Some(previous), Some(current)) => previous.distance_squared(current) > f32::EPSILON,
        (None, Some(_)) | (Some(_), None) => true,
        (None, None) => false,
    };
    *previous_cursor_position = cursor.position;

    let mut tabbed_by_menu: HashMap<Entity, (Entity, usize, usize, TabbedMenuConfig)> = HashMap::new();
    let mut owner_by_tab_root: HashMap<Entity, Entity> = HashMap::new();
    {
        let tab_root_query = tab_queries.p0();
        for (tab_root_entity, tab_bar, tab_state, tab_menu, config, gate) in tab_root_query.iter() {
            if !interaction_gate_allows_for_owner(gate, pause_state, &capture_query, tab_bar.owner) {
                continue;
            }
            owner_by_tab_root.insert(tab_root_entity, tab_bar.owner);
            tabbed_by_menu.insert(
                tab_bar.owner,
                (
                    tab_root_entity,
                    tab_state.active_index,
                    tab_menu.selected_index,
                    *config,
                ),
            );
        }
    }

    let mut hovered_tab_by_menu: HashMap<Entity, (usize, u8, u64)> = HashMap::new();
    let mut clicked_tab_by_menu: HashMap<Entity, (usize, u64)> = HashMap::new();
    let mut hovered_option_by_menu: HashMap<Entity, (usize, u8, u64)> = HashMap::new();
    let mut clicked_option_by_menu: HashMap<Entity, (usize, u64)> = HashMap::new();
    for (tab_entity, tab_item, selectable, hoverable, clickable) in tab_item_query.iter() {
        let Some(owner) = owner_by_tab_root.get(&selectable.menu_entity).copied() else {
            continue;
        };

        if clickable.triggered {
            let rank = tab_entity.to_bits();
            match clicked_tab_by_menu.get_mut(&owner) {
                Some((index, existing_rank)) => {
                    if rank > *existing_rank {
                        *index = tab_item.index;
                        *existing_rank = rank;
                    }
                }
                None => {
                    clicked_tab_by_menu.insert(owner, (tab_item.index, rank));
                }
            }
        }

        let pressed = clickable.triggered || (hoverable.hovered && mouse_input.pressed(MouseButton::Left));
        let priority = if pressed {
            2
        } else if mouse_moved && hoverable.hovered {
            1
        } else {
            0
        };
        if priority == 0 {
            continue;
        }
        let rank = tab_entity.to_bits();
        match hovered_tab_by_menu.get_mut(&owner) {
            Some((index, existing_priority, existing_rank)) => {
                if priority > *existing_priority
                    || (priority == *existing_priority && rank >= *existing_rank)
                {
                    *index = tab_item.index;
                    *existing_priority = priority;
                    *existing_rank = rank;
                }
            }
            None => {
                hovered_tab_by_menu.insert(owner, (tab_item.index, priority, rank));
            }
        }
    }
    for (option_entity, selectable, tabbed_option, hoverable, clickable) in option_query.iter() {
        if clickable.triggered {
            let rank = option_entity.to_bits();
            match clicked_option_by_menu.get_mut(&tabbed_option.owner) {
                Some((index, existing_rank)) => {
                    if rank > *existing_rank {
                        *index = selectable.index;
                        *existing_rank = rank;
                    }
                }
                None => {
                    clicked_option_by_menu.insert(tabbed_option.owner, (selectable.index, rank));
                }
            }
        }
        let pressed = clickable.triggered || (hoverable.hovered && mouse_input.pressed(MouseButton::Left));
        let priority = if pressed {
            2
        } else if mouse_moved && hoverable.hovered {
            1
        } else {
            0
        };
        if priority == 0 {
            continue;
        }
        let rank = option_entity.to_bits();
        match hovered_option_by_menu.get_mut(&tabbed_option.owner) {
            Some((index, existing_priority, existing_rank)) => {
                if priority > *existing_priority
                    || (priority == *existing_priority && rank >= *existing_rank)
                {
                    *index = selectable.index;
                    *existing_priority = priority;
                    *existing_rank = rank;
                }
            }
            None => {
                hovered_option_by_menu.insert(tabbed_option.owner, (selectable.index, priority, rank));
            }
        }
    }

    let mut tab_updates: Vec<(Entity, TabbedMenuFocus, Option<usize>)> = Vec::new();
    let mut pending_tab_activations: Vec<(Entity, usize)> = Vec::new();
    for menu_entity in layer::ordered_active_owners_by_kind(&active_layers, UiLayerKind::Base) {
        let Some(&(tab_root_entity, active_tab_index, selected_tab_index, config)) =
            tabbed_by_menu.get(&menu_entity)
        else {
            continue;
        };
        let Ok(mut selectable_menu) = menu_query.get_mut(menu_entity) else {
            continue;
        };
        selectable_menu.wrap = false;

        let previous_focus = focus_state
            .by_menu
            .get(&menu_entity)
            .copied()
            .unwrap_or(TabbedMenuFocus::Options);
        let previous_selected_index =
            focus_state.previous_selection(menu_entity, selectable_menu.selected_index);
        let clicked_tab_index = clicked_tab_by_menu
            .get(&menu_entity)
            .map(|(clicked_index, _)| *clicked_index);
        let hovered_tab_index = hovered_tab_by_menu
            .get(&menu_entity)
            .map(|(hovered_index, _, _)| *hovered_index);
        let clicked_option_index = clicked_option_by_menu
            .get(&menu_entity)
            .map(|(clicked_index, _)| *clicked_index);
        let hovered_option_index = hovered_option_by_menu
            .get(&menu_entity)
            .map(|(hovered_index, _, _)| *hovered_index);
        let transition = resolve_tabbed_focus(TabbedFocusInputs {
            previous_focus,
            selected_option_index: selectable_menu.selected_index,
            previous_selected_index,
            active_tab_index,
            selected_tab_index,
            option_lock: focus_state.option_lock_by_menu.get(&menu_entity).copied(),
            top_option_count: config.top_option_count,
            footer_start_index: config.footer_start_index,
            footer_count: config.footer_count,
            tab_pressed,
            up_pressed,
            down_pressed,
            left_pressed,
            right_pressed,
            keyboard_focus_navigation,
            clicked_tab_index,
            clicked_option_index,
            hovered_tab_index,
            hovered_option_index,
        });
        let focus = transition.focus;
        let pointer_activity_for_menu = transition.pointer_activity_for_menu;
        let mut option_lock = transition.option_lock;
        selectable_menu.selected_index = transition.selected_option_index;
        if let Some(next_active_tab) = transition.pending_tab_activation {
            pending_tab_activations.push((tab_root_entity, next_active_tab));
        }
        let tab_selection_target = transition.tab_selection_target;

        match focus {
            TabbedMenuFocus::Options => {
                selectable_menu.click_activation = SelectableClickActivation::SelectedOnAnyClick;
                if !keyboard_focus_navigation && !pointer_activity_for_menu {
                    if let Some(locked_index) = option_lock {
                        selectable_menu.selected_index = locked_index;
                    }
                }
                selectable_menu.activate_keys = vec![KeyCode::Enter];
                if config.footer_contains(selectable_menu.selected_index) {
                    // Footer row uses tabbed-focus reducer transitions for vertical movement.
                    // Keeping selectable up/down empty prevents one-step footer cycling before jump.
                    selectable_menu.up_keys.clear();
                    selectable_menu.down_keys.clear();
                } else {
                    selectable_menu.up_keys = vec![KeyCode::ArrowUp];
                    selectable_menu.down_keys = vec![KeyCode::ArrowDown];
                }
            }
            TabbedMenuFocus::Tabs => {
                // Keep base option selection pinned while tabs own focus.
                selectable_menu.selected_index = 0;
                option_lock = None;
                selectable_menu.click_activation = SelectableClickActivation::HoveredOnly;
                selectable_menu.up_keys.clear();
                selectable_menu.down_keys.clear();
                selectable_menu.activate_keys.clear();
            }
        }

        focus_state.by_menu.insert(menu_entity, focus);
        focus_state.set_previous_selection(menu_entity, selectable_menu.selected_index);
        focus_state.set_option_lock(menu_entity, option_lock);
        tab_updates.push((tab_root_entity, focus, tab_selection_target));
    }

    let mut tab_menu_query = tab_queries.p1();
    for (tab_root_entity, focus, tab_selection_target) in tab_updates {
        let Ok(mut tab_menu) = tab_menu_query.get_mut(tab_root_entity) else {
            continue;
        };
        match focus {
            TabbedMenuFocus::Tabs => {
                tab_menu.up_keys = vec![KeyCode::ArrowLeft];
                tab_menu.down_keys = vec![KeyCode::ArrowRight, KeyCode::Tab];
                tab_menu.activate_keys = vec![KeyCode::Enter];
            }
            TabbedMenuFocus::Options => {
                tab_menu.up_keys.clear();
                tab_menu.down_keys.clear();
                tab_menu.activate_keys.clear();
            }
        }
        tab_menu.wrap = true;
        tab_menu.click_activation = SelectableClickActivation::HoveredOnly;
        if let Some(target_index) = tab_selection_target {
            if tab_menu.selected_index != target_index {
                tab_menu.selected_index = target_index;
            }
        }
    }

    let mut tab_state_query = tab_queries.p2();
    for (tab_root_entity, next_active_index) in pending_tab_activations {
        let Ok(mut tab_state) = tab_state_query.get_mut(tab_root_entity) else {
            continue;
        };
        if tab_state.active_index == next_active_index {
            continue;
        }
        tab_state.active_index = next_active_index;
        tab_changed.write(TabChanged {
            tab_bar: tab_root_entity,
            index: next_active_index,
        });
    }
}

pub fn suppress_tabbed_options_while_tabs_focused(
    focus_state: Res<TabbedMenuFocusState>,
    mut option_query: Query<(&TabbedMenuOption, &mut InteractionVisualState)>,
) {
    for (option, mut visual_state) in option_query.iter_mut() {
        if focus_state.is_tabs_focused(option.owner) {
            visual_state.selected = false;
            visual_state.hovered = false;
            visual_state.pressed = false;
        }
    }
}

pub fn commit_tab_activation(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    ui_layer_query: Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
    focus_state: Res<TabbedMenuFocusState>,
    tab_item_query: Query<
        (Entity, &TabItem, &Selectable, &Clickable<SystemMenuActions>),
        Without<TabbedMenuOption>,
    >,
    mut tab_query: Query<
        (
            Entity,
            &TabBar,
            &mut TabBarState,
            &SelectableMenu,
            Option<&tabs::TabActivationPolicy>,
            Option<&TransientAudioPallet<SystemMenuSounds>>,
            Option<&InteractionGate>,
        ),
        With<TabbedMenuConfig>,
    >,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
    mut tab_changed: MessageWriter<TabChanged>,
) {
    // Tab activation only reads tab-item click state from entities that are not
    // tabbed menu options (`Without<TabbedMenuOption>`), keeping click ownership
    // contracts explicit and avoiding mixed-role entities.
    let pause_state = pause_state.as_ref();
    let active_layers =
        layer::active_layers_by_owner_scoped(pause_state, &capture_query, &ui_layer_query);

    let clicked_by_tab_root = tabs::collect_clicked_tab_indices(&tab_item_query);

    for (
        tab_root_entity,
        tab_bar,
        mut tab_state,
        tab_menu,
        activation_policy,
        click_pallet,
        gate,
    ) in tab_query.iter_mut()
    {
        if !interaction_gate_allows_for_owner(gate, pause_state, &capture_query, tab_bar.owner) {
            continue;
        }
        if layer::active_layer_kind_for_owner(&active_layers, tab_bar.owner) != UiLayerKind::Base {
            continue;
        }

        let clicked_target = clicked_by_tab_root.get(&tab_root_entity).copied();
        let keyboard_target = if focus_state.is_tabs_focused(tab_bar.owner) {
            tabs::keyboard_activation_target(
                &keyboard_input,
                activation_policy,
                tab_menu.selected_index,
                tab_state.active_index,
            )
        } else {
            None
        };
        let Some(next_active) = clicked_target.or(keyboard_target) else {
            continue;
        };
        tabs::apply_tab_activation_with_audio(
            tab_root_entity,
            next_active,
            &mut tab_state,
            click_pallet,
            &mut commands,
            &mut audio_query,
            dilation.0,
            SystemMenuSounds::Click,
            &mut tab_changed,
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::IntoSystem;

    use super::*;

    #[test]
    fn tabbed_menu_systems_initialize_without_query_alias_panics() {
        let mut world = World::new();

        let mut cleanup_system = IntoSystem::into_system(cleanup_tabbed_menu_state);
        cleanup_system.initialize(&mut world);

        let mut sync_focus_system = IntoSystem::into_system(sync_tabbed_menu_focus);
        sync_focus_system.initialize(&mut world);

        let mut suppress_system = IntoSystem::into_system(suppress_tabbed_options_while_tabs_focused);
        suppress_system.initialize(&mut world);

        let mut commit_system = IntoSystem::into_system(commit_tab_activation);
        commit_system.initialize(&mut world);
    }

    #[test]
    fn suppress_tabbed_options_only_clears_focused_owner_options() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<TabbedMenuFocusState>();
        app.add_systems(Update, suppress_tabbed_options_while_tabs_focused);

        let owner_a = app.world_mut().spawn_empty().id();
        let owner_b = app.world_mut().spawn_empty().id();
        app.world_mut()
            .resource_mut::<TabbedMenuFocusState>()
            .by_menu
            .insert(owner_a, TabbedMenuFocus::Tabs);

        let focused_option = app
            .world_mut()
            .spawn((
                TabbedMenuOption::new(owner_a),
                InteractionVisualState {
                    selected: true,
                    hovered: true,
                    pressed: true,
                    keyboard_locked: true,
                },
            ))
            .id();
        let unfocused_option = app
            .world_mut()
            .spawn((
                TabbedMenuOption::new(owner_b),
                InteractionVisualState {
                    selected: true,
                    hovered: true,
                    pressed: true,
                    keyboard_locked: true,
                },
            ))
            .id();

        app.update();

        let focused_state = app
            .world()
            .get::<InteractionVisualState>(focused_option)
            .copied()
            .expect("focused option state");
        let unfocused_state = app
            .world()
            .get::<InteractionVisualState>(unfocused_option)
            .copied()
            .expect("unfocused option state");
        assert!(!focused_state.selected);
        assert!(!focused_state.hovered);
        assert!(!focused_state.pressed);
        assert!(focused_state.keyboard_locked);
        assert!(unfocused_state.selected);
        assert!(unfocused_state.hovered);
        assert!(unfocused_state.pressed);
        assert!(unfocused_state.keyboard_locked);
    }

    #[test]
    fn cleanup_tabbed_state_restores_menu_navigation_for_stale_menu() {
        let mut world = World::new();
        world.init_resource::<TabbedMenuFocusState>();

        let stale_menu = world
            .spawn(
                SelectableMenu::new(3, vec![], vec![], vec![], false)
                    .with_click_activation(SelectableClickActivation::HoveredOnly),
            )
            .id();
        world.entity_mut(stale_menu).insert(MenuRoot {
            host: crate::systems::ui::menu::MenuHost::Pause,
            gate: InteractionGate::PauseMenuOnly,
        });
        world.resource_mut::<TabbedMenuFocusState>().by_menu.insert(
            stale_menu,
            TabbedMenuFocus::Tabs,
        );
        world
            .resource_mut::<TabbedMenuFocusState>()
            .set_previous_selection(stale_menu, 2);
        world
            .resource_mut::<TabbedMenuFocusState>()
            .set_option_lock(stale_menu, Some(1));

        let mut cleanup_system = IntoSystem::into_system(cleanup_tabbed_menu_state);
        cleanup_system.initialize(&mut world);
        let _ = cleanup_system.run((), &mut world);
        cleanup_system.apply_deferred(&mut world);

        let menu = world.get::<SelectableMenu>(stale_menu).expect("stale menu");
        assert!(menu.wrap);
        assert_eq!(
            menu.click_activation,
            SelectableClickActivation::SelectedOnAnyClick
        );
        assert_eq!(menu.up_keys, vec![KeyCode::ArrowUp]);
        assert_eq!(menu.down_keys, vec![KeyCode::ArrowDown]);
        assert_eq!(menu.activate_keys, vec![KeyCode::Enter]);

        let focus = world.resource::<TabbedMenuFocusState>();
        assert!(focus.by_menu.is_empty());
        assert!(focus.previous_selection_by_menu.is_empty());
        assert!(focus.option_lock_by_menu.is_empty());
    }
}
