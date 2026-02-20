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
    dropdown_query: &mut Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<VideoResolutionDropdown>>,
    dropdown_menu_query: &mut Query<
        &mut SelectableMenu,
        (With<VideoResolutionDropdown>, Without<MenuRoot>),
    >,
    scroll_root_query: &mut Query<
        (
            &scroll_adapter::ScrollableTableAdapter,
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
    dropdown_query: &mut Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<VideoResolutionDropdown>>,
) {
    dropdown::close_all::<VideoResolutionDropdown>(dropdown_state, dropdown_query);
}

pub(super) fn close_dropdowns_for_menu(
    menu_entity: Entity,
    dropdown_state: &mut DropdownLayerState,
    dropdown_query: &mut Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<VideoResolutionDropdown>>,
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
    mut dropdown_query: Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<VideoResolutionDropdown>>,
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
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    mut settings: ResMut<VideoSettingsState>,
    dropdown_anchor_state: Res<DropdownAnchorState>,
    menu_query: Query<(Entity, &MenuStack, &SelectableMenu), With<MenuRoot>>,
    tab_query: Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
    mut layer_queries: ParamSet<(
        Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
        Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<VideoResolutionDropdown>>,
    )>,
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
    // - `layer_queries` separates active-layer reads (p0) from dropdown-visibility
    //   mutations (p1), preventing visibility access overlap.
    // - dropdown-item click consumption happens through `item_query`, while menu
    //   stack reads remain read-only in `menu_query`.
    if !settings.initialized {
        return;
    }

    // Resolve active layers before mutating dropdown visibility.
    let active_layers = {
        let ui_layer_query = layer_queries.p0();
        layer::active_layers_by_owner_scoped(pause_state.as_ref(), &capture_query, &ui_layer_query)
    };
    let active_tabs = active_video_tabs_by_menu(&tab_query);

    let mut dropdown_owner_parent_by_entity: HashMap<Entity, (Entity, Entity)> = HashMap::new();
    {
        let mut dropdown_query = layer_queries.p1();
        for (dropdown_entity, parent, ui_layer, visibility) in dropdown_query.iter_mut() {
            if *visibility == Visibility::Visible {
                dropdown_owner_parent_by_entity
                    .insert(dropdown_entity, (ui_layer.owner, parent.parent()));
            }
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
        if layer::active_layer_kind_for_owner(&active_layers, owner) != UiLayerKind::Dropdown {
            continue;
        }
        if !layer::is_active_layer_entity_for_owner(&active_layers, owner, dropdown_entity) {
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

    let mut sorted_owners: Vec<Entity> = chosen_by_owner.keys().copied().collect();
    sorted_owners.sort_by_key(|entity| entity.index());

    let mut close_targets: Vec<(Entity, Entity)> = Vec::new();
    for owner in sorted_owners {
        let Some((selected_index, _, item_entity, menu_entity)) = chosen_by_owner.get(&owner).copied()
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

        let Ok((resolved_menu_entity, menu_stack, selectable_menu)) = menu_query.get(menu_entity) else {
            continue;
        };
        if resolved_menu_entity != menu_entity || menu_stack.current_page() != Some(MenuPage::Video) {
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
        let _ = apply_video_top_option_selected_index(
            &mut settings.pending,
            active_tab,
            row,
            clamped,
        );
        close_targets.push((owner, menu_entity));
    }

    close_targets.sort_by_key(|(owner, menu)| (owner.index(), menu.index()));
    close_targets.dedup();

    let mut dropdown_query = layer_queries.p1();
    for (owner, menu_entity) in close_targets {
        dropdown_state.mark_suppress_toggle_for_owner(owner);
        close_dropdowns_for_menu(menu_entity, &mut dropdown_state, &mut dropdown_query);
    }
}

pub(super) fn close_resolution_dropdown_on_outside_click(
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor: Res<crate::startup::cursor::CustomCursor>,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    settings: Res<VideoSettingsState>,
    mut layer_queries: ParamSet<(
        Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
        Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<VideoResolutionDropdown>>,
    )>,
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
    // - `layer_queries` separates read-only active-layer resolution (`p0`) from
    //   dropdown visibility mutation (`p1`) to keep visibility access disjoint.
    // - dropdown hit/item queries are read-only and keyed by explicit dropdown
    //   markers, so outside-click evaluation cannot alias menu mutators.
    if !settings.initialized || !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }
    // Resolve active layers before mutating dropdown visibility.
    let active_layers = {
        let ui_layer_query = layer_queries.p0();
        layer::active_layers_by_owner_scoped(pause_state.as_ref(), &capture_query, &ui_layer_query)
    };
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
            |(
                parent,
                clickable,
                transform,
                global_transform,
                inherited_visibility,
            )| {
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
        dropdown_hit_query
            .iter()
            .any(|(dropdown_entity, transform, global_transform, sprite, inherited_visibility)| {
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
            })
    });
    if click_inside_dropdown_surface {
        return;
    }

    let mut dropdown_query = layer_queries.p1();
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
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    mut layer_queries: ParamSet<(
        Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
        Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<VideoResolutionDropdown>>,
    )>,
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
            &scroll_adapter::ScrollableTableAdapter,
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
    // - `layer_queries` and `selectable_menu_queries` use ParamSet to keep
    //   read/write accesses disjoint across UI layer metadata and menu selection
    //   state mutations.
    // - option-row visual state writes are isolated to `MenuOptionCommand` items.
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

    let pause_state = pause_state.as_ref();
    let active_tabs = active_video_tabs_by_menu(&tab_query);
    // Resolve active layers before mutating dropdown visibility.
    let active_layers = {
        let ui_layer_query = layer_queries.p0();
        layer::active_layers_by_owner_scoped(pause_state, &capture_query, &ui_layer_query)
    };
    let mut dropdown_query = layer_queries.p1();
    let mut selected_dropdown_menu: Option<(Entity, usize, VideoTabKind)> = None;
    {
        let mut menu_query = selectable_menu_queries.p0();
        for (menu_entity, menu_stack, menu_root, mut selectable_menu) in menu_query.iter_mut() {
            if !interaction_gate_allows_for_owner(
                Some(&menu_root.gate),
                pause_state,
                &capture_query,
                menu_entity,
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
            let anchored_row = dropdown_anchor_state.row_for_parent(
                menu_entity,
                menu_entity,
                selected_row,
            );
            let supports_dropdown = video_row_supports_dropdown(active_tab, selected_row);
            let active_kind = layer::active_layer_kind_for_owner(&active_layers, menu_entity);
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
                for (selectable, mut hoverable, mut clickable) in option_query.iter_mut()
                {
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
            let selected_index = video_top_option_selected_index(settings.pending, active_tab, row)
                .unwrap_or(0);
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
