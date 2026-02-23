use super::*;

#[inline]
fn video_row_supports_dropdown(active_tab: VideoTabKind, row: usize) -> bool {
    row < VIDEO_TOP_OPTION_COUNT && video_top_option_uses_dropdown(active_tab, row)
}

pub(super) fn open_dropdown_for_menu(
    menu_entity: Entity,
    row: usize,
    selected_index: usize,
    dropdown_anchor_state: &mut DropdownAnchorState,
    dropdown_state: &mut DropdownLayerState,
    dropdown_query: &mut VideoDropdownVisibilityQuery,
    dropdown_menu_query: &mut VideoDropdownMenuQuery,
    scroll_root_query: &mut Query<
        (
            &crate::systems::ui::scroll::ScrollableTableAdapter,
            &mut crate::systems::ui::scroll::ScrollState,
            &mut crate::systems::ui::scroll::ScrollFocusFollowLock,
        ),
        With<VideoTopOptionsScrollRoot>,
    >,
) {
    scroll_adapter::ensure_video_top_row_visible(menu_entity, row, scroll_root_query);
    dropdown_anchor_state.set_for_parent(menu_entity, menu_entity, row);
    dropdown::open_for_parent::<VideoResolutionDropdown>(
        menu_entity,
        menu_entity,
        selected_index,
        dropdown_state,
        dropdown_query,
        dropdown_menu_query,
    );
}

pub(super) fn close_all_dropdowns(
    dropdown_state: &mut DropdownLayerState,
    dropdown_query: &mut VideoDropdownVisibilityQuery,
) {
    dropdown::close_all::<VideoResolutionDropdown>(dropdown_state, dropdown_query);
}

pub(super) fn close_dropdowns_for_menu(
    menu_entity: Entity,
    dropdown_state: &mut DropdownLayerState,
    dropdown_query: &mut VideoDropdownVisibilityQuery,
) {
    dropdown::close_for_parent::<VideoResolutionDropdown>(
        menu_entity,
        menu_entity,
        dropdown_state,
        dropdown_query,
    );
}

pub(super) fn sync_video_tab_content_state(
    mut tab_changed: MessageReader<tabs::TabChanged>,
    mut dropdown_state: ResMut<DropdownLayerState>,
    dropdown_anchor_state: Res<DropdownAnchorState>,
    tab_query: Query<
        (Entity, &tabs::TabBar, &tabs::TabBarState),
        With<tabbed_menu::TabbedMenuConfig>,
    >,
    menu_query: Query<&SelectableMenu, With<MenuRoot>>,
    mut dropdown_query: VideoDropdownVisibilityQuery,
) {
    let mut owner_by_tab_root: HashMap<Entity, Entity> = HashMap::new();
    for (tab_root, tab_bar, _) in tab_query.iter() {
        owner_by_tab_root.insert(tab_root, tab_bar.owner);
    }
    let mut changed_owners = HashSet::new();
    for changed in tab_changed.read() {
        let Some(owner) = owner_by_tab_root.get(&changed.tab_bar).copied() else {
            continue;
        };
        changed_owners.insert(owner);
    }

    for (_, tab_bar, tab_state) in tab_query.iter() {
        if changed_owners.contains(&tab_bar.owner) {
            close_dropdowns_for_menu(tab_bar.owner, &mut dropdown_state, &mut dropdown_query);
            continue;
        }
        let Ok(menu) = menu_query.get(tab_bar.owner) else {
            continue;
        };
        let row = if dropdown_state.is_parent_open_for_owner(tab_bar.owner, tab_bar.owner) {
            dropdown_anchor_state.row_for_parent(tab_bar.owner, tab_bar.owner, menu.selected_index)
        } else {
            menu.selected_index
        };
        let active_tab = video_tab_kind(tab_state.active_index);
        let supports_dropdown = video_row_supports_dropdown(active_tab, row);
        if !supports_dropdown {
            close_dropdowns_for_menu(tab_bar.owner, &mut dropdown_state, &mut dropdown_query);
        }
    }
}

pub(super) fn handle_resolution_dropdown_item_commands(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor: Res<crate::startup::cursor::CustomCursor>,
    interaction_state: Res<UiInteractionState>,
    mut settings: ResMut<VideoSettingsState>,
    dropdown_anchor_state: Res<DropdownAnchorState>,
    menu_query: Query<(Entity, &MenuStack, &SelectableMenu), With<MenuRoot>>,
    tab_query: Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
    mut dropdown_query: VideoDropdownVisibilityQuery,
    mut dropdown_state: ResMut<DropdownLayerState>,
    mut item_query: Query<(
        Entity,
        &ChildOf,
        &VideoResolutionDropdownItem,
        &mut Clickable<SystemMenuActions>,
        &Transform,
        &GlobalTransform,
        Option<&InheritedVisibility>,
        Option<&TransientAudioPallet<SystemMenuSounds>>,
    )>,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
) {
    // Query contract:
    // - dropdown-item click consumption happens through `item_query`, while menu
    //   stack reads remain read-only in `menu_query`.
    // - dropdown visibility mutates through `dropdown_query`.
    if !settings.initialized {
        return;
    }

    let active_layers = &interaction_state.active_layers_by_owner;
    let active_tabs = active_video_tabs_by_menu(&tab_query);

    let mut dropdown_owner_parent_by_entity: HashMap<Entity, (Entity, Entity)> = HashMap::new();
    for (dropdown_entity, parent, ui_layer, visibility) in dropdown_query.iter_mut() {
        if *visibility == Visibility::Visible {
            dropdown_owner_parent_by_entity.insert(dropdown_entity, (ui_layer.owner, parent.parent()));
        }
    }

    let mut chosen_by_owner: HashMap<Entity, (usize, u64, Entity, Entity)> = HashMap::new();
    let primary_mouse_click = mouse_input.just_pressed(MouseButton::Left);
    let click_position = if primary_mouse_click {
        cursor.position
    } else {
        None
    };
    for (
        entity,
        parent,
        item,
        mut clickable,
        transform,
        global_transform,
        inherited_visibility,
        _,
    ) in item_query.iter_mut()
    {
        let cursor_pressed_inside_item = click_position.is_some_and(|cursor_position| {
            if inherited_visibility.is_some_and(|visibility| !visibility.get()) {
                return false;
            }
            clickable.region.is_some_and(|region| {
                crate::systems::interaction::is_cursor_within_region(
                    cursor_position,
                    transform,
                    global_transform,
                    region,
                    Vec2::ZERO,
                )
            })
        });
        let pressed_click = primary_mouse_click && cursor_pressed_inside_item;
        if !(clickable.triggered || pressed_click) {
            continue;
        }
        clickable.triggered = false;

        let dropdown_entity = parent.parent();
        let Some((owner, menu_entity)) = dropdown_owner_parent_by_entity
            .get(&dropdown_entity)
            .copied()
        else {
            continue;
        };
        if layer::active_layer_kind_for_owner(active_layers, owner) != UiLayerKind::Dropdown {
            continue;
        }
        if !layer::is_active_layer_entity_for_owner(active_layers, owner, dropdown_entity) {
            continue;
        }

        let candidate = (item.index, entity.to_bits(), entity, menu_entity);
        match chosen_by_owner.get_mut(&owner) {
            Some((best_index, best_rank, best_entity, best_menu_entity)) => {
                if candidate.1 > *best_rank {
                    *best_index = candidate.0;
                    *best_rank = candidate.1;
                    *best_entity = candidate.2;
                    *best_menu_entity = candidate.3;
                }
            }
            None => {
                chosen_by_owner.insert(owner, candidate);
            }
        }
    }

    if chosen_by_owner.is_empty() {
        return;
    }

    let mut close_targets: Vec<(Entity, Entity)> = Vec::new();
    for owner in layer::ordered_active_owners_by_kind(active_layers, UiLayerKind::Dropdown) {
        let Some((selected_index, _, item_entity, menu_entity)) =
            chosen_by_owner.get(&owner).copied()
        else {
            continue;
        };
        if let Ok((_, _, _, _, _, _, _, click_pallet)) = item_query.get_mut(item_entity) {
            if let Some(click_pallet) = click_pallet {
                TransientAudioPallet::play_transient_audio(
                    item_entity,
                    &mut commands,
                    click_pallet,
                    SystemMenuSounds::Click,
                    dilation.0,
                    &mut audio_query,
                );
            }
        }

        let Ok((resolved_menu_entity, menu_stack, selectable_menu)) = menu_query.get(menu_entity)
        else {
            continue;
        };
        if resolved_menu_entity != menu_entity || menu_stack.current_page() != Some(MenuPage::Video)
        {
            continue;
        }
        let row = dropdown_anchor_state.row_for_parent(
            menu_entity,
            menu_entity,
            selectable_menu.selected_index,
        );
        let Some(active_tab) = active_tabs.get(&menu_entity).copied().map(video_tab_kind) else {
            continue;
        };
        let choice_count = video_top_option_choice_count(active_tab, row);
        if choice_count == 0 {
            continue;
        }
        let clamped = selected_index.min(choice_count - 1);
        let _ =
            apply_video_top_option_selected_index(&mut settings.pending, active_tab, row, clamped);
        close_targets.push((owner, menu_entity));
    }

    for (owner, menu_entity) in close_targets {
        dropdown_state.mark_suppress_toggle_for_owner(owner);
        close_dropdowns_for_menu(menu_entity, &mut dropdown_state, &mut dropdown_query);
    }
}

pub(super) fn close_resolution_dropdown_on_outside_click(
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor: Res<crate::startup::cursor::CustomCursor>,
    interaction_state: Res<UiInteractionState>,
    settings: Res<VideoSettingsState>,
    mut dropdown_query: VideoDropdownVisibilityQuery,
    mut dropdown_state: ResMut<DropdownLayerState>,
    dropdown_hit_query: Query<
        (
            Entity,
            &Transform,
            &GlobalTransform,
            &Sprite,
            Option<&InheritedVisibility>,
        ),
        With<VideoResolutionDropdown>,
    >,
    item_query: Query<
        (
            &ChildOf,
            &Clickable<SystemMenuActions>,
            &Transform,
            &GlobalTransform,
            Option<&InheritedVisibility>,
        ),
        With<VideoResolutionDropdownItem>,
    >,
) {
    // Query contract:
    // - dropdown hit/item queries are read-only and keyed by explicit dropdown
    //   markers, so outside-click evaluation cannot alias menu mutators.
    // - dropdown visibility mutates only through `dropdown_query`.
    if !settings.initialized || !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }
    let active_layers = &interaction_state.active_layers_by_owner;
    let any_active_dropdown = active_layers
        .values()
        .any(|active| active.kind == UiLayerKind::Dropdown);
    if !any_active_dropdown {
        return;
    }
    let active_dropdowns: HashSet<Entity> = active_layers
        .values()
        .filter_map(|active| (active.kind == UiLayerKind::Dropdown).then_some(active.entity))
        .collect();

    let cursor_position = cursor.position;
    let click_inside_item = cursor_position.is_some_and(|cursor_position| {
        item_query.iter().any(
            |(parent, clickable, transform, global_transform, inherited_visibility)| {
                if !active_dropdowns.contains(&parent.parent()) {
                    return false;
                }
                if inherited_visibility.is_some_and(|visibility| !visibility.get()) {
                    return false;
                }
                clickable.region.is_some_and(|region| {
                    crate::systems::interaction::is_cursor_within_region(
                        cursor_position,
                        transform,
                        global_transform,
                        region,
                        Vec2::ZERO,
                    )
                })
            },
        )
    });
    if click_inside_item {
        return;
    }

    let click_inside_dropdown_surface = cursor_position.is_some_and(|cursor_position| {
        dropdown_hit_query.iter().any(
            |(dropdown_entity, transform, global_transform, sprite, inherited_visibility)| {
                if !active_dropdowns.contains(&dropdown_entity) {
                    return false;
                }
                if inherited_visibility.is_some_and(|visibility| !visibility.get()) {
                    return false;
                }
                let Some(size) = sprite.custom_size else {
                    return false;
                };
                crate::systems::interaction::is_cursor_within_region(
                    cursor_position,
                    transform,
                    global_transform,
                    size,
                    Vec2::ZERO,
                )
            },
        )
    });
    if click_inside_dropdown_surface {
        return;
    }

    let any_visible = dropdown_query
        .iter_mut()
        .any(|(_, _, _, visibility)| *visibility == Visibility::Visible);
    if !any_visible {
        return;
    }

    dropdown_state.mark_suppress_toggle_for_open_owners();
    close_all_dropdowns(&mut dropdown_state, &mut dropdown_query);
}

pub(super) fn handle_resolution_dropdown_keyboard_navigation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    interaction_state: Res<UiInteractionState>,
    mut dropdown_query: VideoDropdownVisibilityQuery,
    settings: Res<VideoSettingsState>,
    mut dropdown_state: ResMut<DropdownLayerState>,
    mut dropdown_anchor_state: ResMut<DropdownAnchorState>,
    tabbed_focus: Res<tabbed_menu::TabbedMenuFocusState>,
    tab_query: Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
    mut selectable_menu_queries: ParamSet<(
        Query<(Entity, &MenuStack, &MenuRoot, &mut SelectableMenu)>,
        Query<&mut SelectableMenu, (With<VideoResolutionDropdown>, Without<MenuRoot>)>,
    )>,
    mut scroll_root_query: Query<
        (
            &crate::systems::ui::scroll::ScrollableTableAdapter,
            &mut crate::systems::ui::scroll::ScrollState,
            &mut crate::systems::ui::scroll::ScrollFocusFollowLock,
        ),
        With<VideoTopOptionsScrollRoot>,
    >,
    mut option_query: Query<
        (
            &Selectable,
            &mut Hoverable,
            &mut Clickable<SystemMenuActions>,
        ),
        With<MenuOptionCommand>,
    >,
) {
    // Query contract:
    // - `selectable_menu_queries` uses ParamSet to keep mutable menu and dropdown
    //   selectable writes disjoint.
    // - option-row visual state writes are isolated to `MenuOptionCommand` items.
    // - dropdown visibility mutates only through `dropdown_query`.
    if !settings.initialized {
        return;
    }

    let left_pressed = keyboard_input.just_pressed(KeyCode::ArrowLeft);
    let right_pressed = keyboard_input.just_pressed(KeyCode::ArrowRight);
    let backspace_pressed = keyboard_input.just_pressed(KeyCode::Backspace);
    let escape_pressed = keyboard_input.just_pressed(KeyCode::Escape);
    let dropdown_shortcut = dropdown_digit_shortcut_index(&keyboard_input);
    let dropdown_open = dropdown_state.is_any_open();

    if !(left_pressed
        || right_pressed
        || backspace_pressed
        || escape_pressed
        || dropdown_shortcut.is_some()
        || dropdown_open)
    {
        return;
    }

    let active_tabs = active_video_tabs_by_menu(&tab_query);
    let active_layers = &interaction_state.active_layers_by_owner;
    let mut selected_dropdown_menu: Option<(Entity, usize, VideoTabKind)> = None;
    let ordered_menu_owners: Vec<Entity> = layer::ordered_active_layers_by_owner(active_layers)
        .into_iter()
        .map(|(owner, _)| owner)
        .collect();

    {
        let mut menu_query = selectable_menu_queries.p0();
        for menu_entity in ordered_menu_owners {
            let Ok((_, menu_stack, menu_root, mut selectable_menu)) =
                menu_query.get_mut(menu_entity)
            else {
                continue;
            };
            if !ui_input_policy_allows_mode(
                Some(&menu_root.gate),
                interaction_state.input_mode_for_owner(menu_entity),
            ) {
                continue;
            }
            if menu_stack.current_page() != Some(MenuPage::Video) {
                continue;
            }
            if tabbed_focus.is_tabs_focused(menu_entity) {
                continue;
            }
            let Some(active_tab) = active_tabs.get(&menu_entity).copied().map(video_tab_kind)
            else {
                continue;
            };
            let selected_row = selectable_menu.selected_index;
            let anchored_row =
                dropdown_anchor_state.row_for_parent(menu_entity, menu_entity, selected_row);
            let supports_dropdown = video_row_supports_dropdown(active_tab, selected_row);
            let active_kind = layer::active_layer_kind_for_owner(active_layers, menu_entity);
            if active_kind == UiLayerKind::Modal {
                continue;
            }

            if active_kind == UiLayerKind::Dropdown {
                let selected_row = anchored_row;
                selectable_menu.selected_index = selected_row;
                let supports_dropdown = video_row_supports_dropdown(active_tab, selected_row);
                if !supports_dropdown {
                    close_dropdowns_for_menu(menu_entity, &mut dropdown_state, &mut dropdown_query);
                    return;
                }
                for (selectable, mut hoverable, mut clickable) in option_query.iter_mut() {
                    if selectable.menu_entity != menu_entity {
                        continue;
                    }
                    clickable.triggered = false;
                    if selectable.index != selected_row {
                        hoverable.hovered = false;
                    }
                }

                if let Some(shortcut_index) = dropdown_shortcut {
                    let choice_count = video_top_option_choice_count(active_tab, selected_row);
                    if choice_count > 0 && shortcut_index < choice_count {
                        let mut dropdown_menu_query = selectable_menu_queries.p1();
                        open_dropdown_for_menu(
                            menu_entity,
                            selected_row,
                            shortcut_index,
                            &mut dropdown_anchor_state,
                            &mut dropdown_state,
                            &mut dropdown_query,
                            &mut dropdown_menu_query,
                            &mut scroll_root_query,
                        );
                    }
                } else if (left_pressed && !right_pressed) || backspace_pressed || escape_pressed {
                    close_dropdowns_for_menu(menu_entity, &mut dropdown_state, &mut dropdown_query);
                }
                return;
            }

            if supports_dropdown && selected_dropdown_menu.is_none() {
                selected_dropdown_menu = Some((menu_entity, selected_row, active_tab));
            }
        }
    }

    if let Some(shortcut_index) = dropdown_shortcut {
        if let Some((menu_entity, row, active_tab)) = selected_dropdown_menu {
            let choice_count = video_top_option_choice_count(active_tab, row);
            if choice_count > 0 && shortcut_index < choice_count {
                let mut dropdown_menu_query = selectable_menu_queries.p1();
                open_dropdown_for_menu(
                    menu_entity,
                    row,
                    shortcut_index,
                    &mut dropdown_anchor_state,
                    &mut dropdown_state,
                    &mut dropdown_query,
                    &mut dropdown_menu_query,
                    &mut scroll_root_query,
                );
            }
        }
        return;
    }

    if dropdown_state.is_any_open() {
        if (left_pressed && !right_pressed) || backspace_pressed || escape_pressed {
            close_all_dropdowns(&mut dropdown_state, &mut dropdown_query);
        }
        return;
    }

    if right_pressed && !left_pressed && !backspace_pressed && !escape_pressed {
        if let Some((menu_entity, row, active_tab)) = selected_dropdown_menu {
            let selected_index =
                video_top_option_selected_index(settings.pending, active_tab, row).unwrap_or(0);
            let mut dropdown_menu_query = selectable_menu_queries.p1();
            open_dropdown_for_menu(
                menu_entity,
                row,
                selected_index,
                &mut dropdown_anchor_state,
                &mut dropdown_state,
                &mut dropdown_query,
                &mut dropdown_menu_query,
                &mut scroll_root_query,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::{
        ecs::system::{IntoSystem, SystemState},
        prelude::*,
    };

    use super::*;
    use crate::{
        startup::cursor::CustomCursor,
        systems::{
            interaction::{Clickable, SelectableMenu, SystemMenuActions, UiInputPolicy},
            ui::layer::{UiLayer, UiLayerKind},
        },
    };

    fn test_selectable_menu() -> SelectableMenu {
        SelectableMenu::new(
            0,
            vec![KeyCode::ArrowUp],
            vec![KeyCode::ArrowDown],
            vec![KeyCode::Enter],
            true,
        )
    }

    #[test]
    fn open_dropdown_for_menu_scrolls_row_into_view_and_sets_anchor() {
        let mut world = World::new();

        let mut dropdown_anchor_state = DropdownAnchorState::default();
        let mut dropdown_state = DropdownLayerState::default();

        let menu_entity = world
            .spawn((
                MenuRoot {
                    host: MenuHost::Pause,
                    gate: UiInputPolicy::CapturedOnly,
                },
                test_selectable_menu(),
            ))
            .id();

        let dropdown_entity = world
            .spawn((
                VideoResolutionDropdown,
                UiLayer::new(menu_entity, UiLayerKind::Dropdown),
                Visibility::Hidden,
                test_selectable_menu(),
            ))
            .id();
        world.entity_mut(menu_entity).add_child(dropdown_entity);

        let scroll_root = world
            .spawn((
                VideoTopOptionsScrollRoot,
                crate::systems::ui::scroll::ScrollableTableAdapter::new(
                    menu_entity,
                    10,
                    40.0,
                    60.0,
                ),
                crate::systems::ui::scroll::ScrollState {
                    offset_px: 0.0,
                    content_extent: 520.0,
                    viewport_extent: 240.0,
                    max_offset: 280.0,
                },
                crate::systems::ui::scroll::ScrollFocusFollowLock {
                    manual_override: true,
                },
            ))
            .id();

        let mut query_state: SystemState<(
            Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<VideoResolutionDropdown>>,
            Query<&mut SelectableMenu, (With<VideoResolutionDropdown>, Without<MenuRoot>)>,
            Query<
                (
                    &crate::systems::ui::scroll::ScrollableTableAdapter,
                    &mut crate::systems::ui::scroll::ScrollState,
                    &mut crate::systems::ui::scroll::ScrollFocusFollowLock,
                ),
                With<VideoTopOptionsScrollRoot>,
            >,
        )> = SystemState::new(&mut world);

        {
            let (mut dropdown_query, mut dropdown_menu_query, mut scroll_root_query) =
                query_state.get_mut(&mut world);
            open_dropdown_for_menu(
                menu_entity,
                7,
                3,
                &mut dropdown_anchor_state,
                &mut dropdown_state,
                &mut dropdown_query,
                &mut dropdown_menu_query,
                &mut scroll_root_query,
            );
        }
        query_state.apply(&mut world);

        assert_eq!(
            dropdown_anchor_state.row_for_parent(menu_entity, menu_entity, 0),
            7
        );
        assert_eq!(
            dropdown_state.open_parent_for_owner(menu_entity),
            Some(menu_entity)
        );
        assert_eq!(
            world.entity(dropdown_entity).get::<Visibility>(),
            Some(&Visibility::Visible)
        );
        assert_eq!(
            world
                .entity(dropdown_entity)
                .get::<SelectableMenu>()
                .map(|menu| menu.selected_index),
            Some(3)
        );

        let state = world
            .entity(scroll_root)
            .get::<crate::systems::ui::scroll::ScrollState>()
            .copied()
            .expect("scroll state");
        let lock = world
            .entity(scroll_root)
            .get::<crate::systems::ui::scroll::ScrollFocusFollowLock>()
            .copied()
            .expect("focus-follow lock");
        assert!((state.offset_px - 140.0).abs() < 0.001);
        assert!(!lock.manual_override);
    }

    #[test]
    fn outside_click_does_not_close_when_cursor_is_inside_dropdown_item() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ButtonInput<MouseButton>>();
        app.init_resource::<CustomCursor>();
        app.init_resource::<DropdownLayerState>();
        app.add_systems(Update, close_resolution_dropdown_on_outside_click);

        let mut settings = VideoSettingsState::default();
        settings.initialized = true;
        app.insert_resource(settings);

        let menu_entity = app.world_mut().spawn_empty().id();
        let dropdown_entity = app
            .world_mut()
            .spawn((
                VideoResolutionDropdown,
                UiLayer::new(menu_entity, UiLayerKind::Dropdown),
                Visibility::Visible,
                Sprite::from_color(Color::NONE, Vec2::new(200.0, 140.0)),
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();
        app.world_mut()
            .entity_mut(menu_entity)
            .add_child(dropdown_entity);

        let item_entity = app
            .world_mut()
            .spawn((
                VideoResolutionDropdownItem { index: 2 },
                Clickable::with_region(vec![SystemMenuActions::Activate], Vec2::new(100.0, 40.0)),
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();
        app.world_mut()
            .entity_mut(dropdown_entity)
            .add_child(item_entity);

        {
            let world = app.world_mut();
            let mut query_state: SystemState<(
                ResMut<DropdownLayerState>,
                Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<VideoResolutionDropdown>>,
                Query<&mut SelectableMenu, With<VideoResolutionDropdown>>,
            )> = SystemState::new(world);
            let (mut dropdown_state, mut dropdown_query, mut dropdown_menu_query) =
                query_state.get_mut(world);
            dropdown::open_for_parent::<VideoResolutionDropdown>(
                menu_entity,
                menu_entity,
                0,
                dropdown_state.as_mut(),
                &mut dropdown_query,
                &mut dropdown_menu_query,
            );
            query_state.apply(world);
        }

        app.world_mut().resource_mut::<CustomCursor>().position = Some(Vec2::ZERO);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);

        app.update();

        assert_eq!(
            app.world().get::<Visibility>(dropdown_entity),
            Some(&Visibility::Visible)
        );
        assert_eq!(
            app.world()
                .resource::<DropdownLayerState>()
                .open_parent_for_owner(menu_entity),
            Some(menu_entity)
        );
    }

    #[test]
    fn dropdown_flow_systems_initialize_without_query_alias_panics() {
        let mut world = World::new();

        let mut sync_tabs_system = IntoSystem::into_system(sync_video_tab_content_state);
        sync_tabs_system.initialize(&mut world);

        let mut item_command_system =
            IntoSystem::into_system(handle_resolution_dropdown_item_commands);
        item_command_system.initialize(&mut world);

        let mut outside_click_system =
            IntoSystem::into_system(close_resolution_dropdown_on_outside_click);
        outside_click_system.initialize(&mut world);

        let mut keyboard_nav_system =
            IntoSystem::into_system(handle_resolution_dropdown_keyboard_navigation);
        keyboard_nav_system.initialize(&mut world);
    }

    #[test]
    fn keyboard_dropdown_open_prefers_lowest_owner_index_when_multiple_menus_match() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<DropdownLayerState>();
        app.init_resource::<DropdownAnchorState>();
        app.init_resource::<tabbed_menu::TabbedMenuFocusState>();
        app.add_systems(Update, handle_resolution_dropdown_keyboard_navigation);

        let mut settings = VideoSettingsState::default();
        settings.initialized = true;
        app.insert_resource(settings);

        let menu_high = app
            .world_mut()
            .spawn((
                MenuRoot {
                    host: MenuHost::Debug,
                    gate: UiInputPolicy::WorldOnly,
                },
                MenuStack::new(MenuPage::Video),
                SelectableMenu::new(
                    VIDEO_RESOLUTION_OPTION_INDEX,
                    vec![KeyCode::ArrowUp],
                    vec![KeyCode::ArrowDown],
                    vec![KeyCode::Enter],
                    true,
                ),
                UiLayer::new(Entity::PLACEHOLDER, UiLayerKind::Base),
                Visibility::Visible,
            ))
            .id();
        app.world_mut()
            .entity_mut(menu_high)
            .insert(UiLayer::new(menu_high, UiLayerKind::Base));

        let menu_low = app
            .world_mut()
            .spawn((
                MenuRoot {
                    host: MenuHost::Debug,
                    gate: UiInputPolicy::WorldOnly,
                },
                MenuStack::new(MenuPage::Video),
                SelectableMenu::new(
                    VIDEO_RESOLUTION_OPTION_INDEX,
                    vec![KeyCode::ArrowUp],
                    vec![KeyCode::ArrowDown],
                    vec![KeyCode::Enter],
                    true,
                ),
                UiLayer::new(Entity::PLACEHOLDER, UiLayerKind::Base),
                Visibility::Visible,
            ))
            .id();
        app.world_mut()
            .entity_mut(menu_low)
            .insert(UiLayer::new(menu_low, UiLayerKind::Base));

        let (expected_first, expected_second) = if menu_low.index() < menu_high.index() {
            (menu_low, menu_high)
        } else {
            (menu_high, menu_low)
        };

        let dropdown_high = app
            .world_mut()
            .spawn((
                VideoResolutionDropdown,
                UiLayer::new(menu_high, UiLayerKind::Dropdown),
                Visibility::Hidden,
                SelectableMenu::new(0, vec![], vec![], vec![], true),
            ))
            .id();
        app.world_mut()
            .entity_mut(menu_high)
            .add_child(dropdown_high);

        let dropdown_low = app
            .world_mut()
            .spawn((
                VideoResolutionDropdown,
                UiLayer::new(menu_low, UiLayerKind::Dropdown),
                Visibility::Hidden,
                SelectableMenu::new(0, vec![], vec![], vec![], true),
            ))
            .id();
        app.world_mut().entity_mut(menu_low).add_child(dropdown_low);

        app.world_mut().spawn((
            tabs::TabBar::new(menu_high),
            tabs::TabBarState { active_index: 0 },
            tabbed_menu::TabbedMenuConfig::new(
                VIDEO_TOP_OPTION_COUNT,
                VIDEO_FOOTER_OPTION_START_INDEX,
                VIDEO_FOOTER_OPTION_COUNT,
            ),
        ));
        app.world_mut().spawn((
            tabs::TabBar::new(menu_low),
            tabs::TabBarState { active_index: 0 },
            tabbed_menu::TabbedMenuConfig::new(
                VIDEO_TOP_OPTION_COUNT,
                VIDEO_FOOTER_OPTION_START_INDEX,
                VIDEO_FOOTER_OPTION_COUNT,
            ),
        ));

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowRight);
        app.update();

        let state = app.world().resource::<DropdownLayerState>();
        assert_eq!(
            state.open_parent_for_owner(expected_first),
            Some(expected_first)
        );
        assert_eq!(state.open_parent_for_owner(expected_second), None);
    }
}
