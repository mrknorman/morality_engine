use super::*;
use super::modal_flow::spawn_exit_unsaved_modal;

pub(super) fn apply_menu_intents(
    mut menu_intents: MessageReader<MenuIntent>,
    mut option_query: Query<(
        Entity,
        &Selectable,
        &MenuOptionCommand,
        &mut Clickable<SystemMenuActions>,
    ), Without<VideoModalButton>>,
    mut modal_button_query: Query<
        (Entity, &VideoModalButton, &mut Clickable<SystemMenuActions>),
        Without<MenuOptionCommand>,
    >,
) {
    // Query contract:
    // - `option_query` excludes modal buttons.
    // - `modal_button_query` excludes menu option commands.
    // This keeps mutable `Clickable<SystemMenuActions>` access disjoint.
    let mut command_intents: Vec<(Entity, MenuCommand)> = Vec::new();
    let mut modal_button_intents: Vec<VideoModalButton> = Vec::new();

    for intent in menu_intents.read() {
        match intent {
            MenuIntent::TriggerCommand {
                menu_entity,
                command,
            } => command_intents.push((*menu_entity, command.clone())),
            MenuIntent::TriggerModalButton(button) => modal_button_intents.push(*button),
        }
    }

    for (menu_entity, command) in command_intents {
        let mut best_target: Option<(usize, u64, Entity)> = None;
        for (entity, selectable, option_command, _) in option_query.iter_mut() {
            if selectable.menu_entity != menu_entity || option_command.0 != command {
                continue;
            }
            let candidate = (selectable.index, entity.to_bits(), entity);
            let replace = match best_target {
                Some((best_index, best_rank, _)) => {
                    candidate.0 < best_index || (candidate.0 == best_index && candidate.1 > best_rank)
                }
                None => true,
            };
            if replace {
                best_target = Some(candidate);
            }
        }
        let Some((_, _, target_entity)) = best_target else {
            continue;
        };
        if let Ok((_, _, _, mut clickable)) = option_query.get_mut(target_entity) {
            clickable.triggered = true;
        }
    }

    for target_button in modal_button_intents {
        let mut best_target: Option<(u64, Entity)> = None;
        for (entity, button, _) in modal_button_query.iter_mut() {
            if *button == target_button {
                let rank = entity.to_bits();
                if best_target
                    .as_ref()
                    .is_none_or(|(best_rank, _)| rank > *best_rank)
                {
                    best_target = Some((rank, entity));
                }
            }
        }
        let Some((_, target_entity)) = best_target else {
            continue;
        };
        if let Ok((_, _, mut clickable)) = modal_button_query.get_mut(target_entity) {
            clickable.triggered = true;
        }
    }
}

pub(super) fn play_menu_navigation_sound(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    tabbed_focus: Res<tabbed_menu::TabbedMenuFocusState>,
    ui_layer_query: Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
    layer_menu_query: Query<(
        Entity,
        &MenuStack,
        &SelectableMenu,
        &TransientAudioPallet<SystemMenuSounds>,
    )>,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
) {
    let active_layers =
        layer::active_layers_by_owner_scoped(pause_state.as_ref(), &capture_query, &ui_layer_query);
    let left_pressed = keyboard_input.just_pressed(KeyCode::ArrowLeft);
    let right_pressed = keyboard_input.just_pressed(KeyCode::ArrowRight);
    let tab_pressed = keyboard_input.just_pressed(KeyCode::Tab);
    let horizontal_pressed = left_pressed ^ right_pressed;
    for active_layer in active_layers.values() {
        let Ok((layer_entity, menu_stack, menu, pallet)) = layer_menu_query.get(active_layer.entity) else {
            continue;
        };
        let mut should_play = system_menu::navigation_switch_pressed(menu, &keyboard_input);
        if !should_play && menu_stack.current_page() == Some(MenuPage::Video) {
            let tabs_focused = tabbed_focus.is_tabs_focused(layer_entity);
            if tabs_focused {
                should_play = horizontal_pressed || tab_pressed;
            } else {
                should_play = horizontal_pressed
                    && menu.selected_index >= VIDEO_FOOTER_OPTION_START_INDEX
                    && menu.selected_index < VIDEO_FOOTER_OPTION_START_INDEX + VIDEO_FOOTER_OPTION_COUNT;
            }
        }
        if !should_play {
            continue;
        }
        TransientAudioPallet::play_transient_audio(
            layer_entity,
            &mut commands,
            pallet,
            SystemMenuSounds::Switch,
            dilation.0,
            &mut audio_query,
        );
    }
}

pub(super) fn enforce_active_layer_focus(
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    ui_layer_query: Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
    layer_meta_query: Query<&UiLayer>,
    mut layer_menu_query: Query<(Entity, &UiLayer, &mut SelectableMenu)>,
    mut option_query: Query<
        (
            &Selectable,
            &mut InteractionVisualState,
            &mut Clickable<SystemMenuActions>,
        ),
        With<MenuOptionCommand>,
    >,
    mut cached_indices: ResMut<InactiveLayerSelectionCache>,
) {
    // Query contract:
    // - Layer-level `SelectableMenu` mutations are isolated from option-level
    //   visual/clickable mutations (`MenuOptionCommand` entities).
    // - `layer_meta_query` is read-only metadata lookup.
    let active_layers =
        layer::active_layers_by_owner_scoped(pause_state.as_ref(), &capture_query, &ui_layer_query);

    let mut live_layers = HashSet::new();
    for (layer_entity, ui_layer, mut selectable_menu) in layer_menu_query.iter_mut() {
        live_layers.insert(layer_entity);
        let is_active =
            layer::is_active_layer_entity_for_owner(&active_layers, ui_layer.owner, layer_entity);

        if is_active {
            cached_indices
                .by_layer
                .insert(layer_entity, selectable_menu.selected_index);
            continue;
        }

        if let Some(cached_index) = cached_indices.by_layer.get(&layer_entity).copied() {
            selectable_menu.selected_index = cached_index;
        } else {
            cached_indices
                .by_layer
                .insert(layer_entity, selectable_menu.selected_index);
        }
    }
    cached_indices
        .by_layer
        .retain(|layer_entity, _| live_layers.contains(layer_entity));

    for (selectable, mut visual_state, mut clickable) in option_query.iter_mut() {
        let Ok(ui_layer) = layer_meta_query.get(selectable.menu_entity) else {
            continue;
        };
        let is_active = layer::is_active_layer_entity_for_owner(
            &active_layers,
            ui_layer.owner,
            selectable.menu_entity,
        );
        if is_active {
            continue;
        }

        clickable.triggered = false;
        visual_state.selected = false;
        visual_state.hovered = false;
        visual_state.pressed = false;
        visual_state.keyboard_locked = false;
    }
}

pub(super) fn handle_menu_shortcuts(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    settings: Res<VideoSettingsState>,
    tabbed_focus: Res<tabbed_menu::TabbedMenuFocusState>,
    mut navigation_state: ResMut<MenuNavigationState>,
    ui_layer_query: Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
    menu_query: Query<(Entity, &MenuStack, &MenuRoot, &SelectableMenu)>,
    option_shortcut_query: Query<(&Selectable, &ShortcutKey, &MenuOptionCommand)>,
    option_command_query: Query<(&Selectable, &MenuOptionCommand, Option<&OptionCycler>)>,
    mut menu_intents: MessageWriter<MenuIntent>,
) {
    let escape_pressed = keyboard_input.just_pressed(KeyCode::Escape);
    let right_pressed = keyboard_input.just_pressed(KeyCode::ArrowRight);
    let left_pressed = keyboard_input.just_pressed(KeyCode::ArrowLeft);
    let activate_right = right_pressed && !left_pressed;
    let activate_left = left_pressed && !right_pressed;

    let pause_state = pause_state.as_ref();
    let active_layers =
        layer::active_layers_by_owner_scoped(pause_state, &capture_query, &ui_layer_query);
    let mut handled_escape = false;

    for (menu_entity, menu_stack, menu_root, _) in menu_query.iter() {
        if !interaction_gate_allows_for_owner(
            Some(&menu_root.gate),
            pause_state,
            &capture_query,
            menu_entity,
        ) {
            continue;
        }
        let active_kind = layer::active_layer_kind_for_owner(&active_layers, menu_entity);
        if active_kind != UiLayerKind::Base {
            continue;
        }

        let Some(page) = menu_stack.current_page() else {
            continue;
        };

        if escape_pressed && !handled_escape {
            handled_escape = true;
            let leaving_video_options = matches!(page, MenuPage::Video | MenuPage::Options);
            if settings.initialized && video_settings_dirty(&settings) && leaving_video_options {
                navigation_state.exit_prompt_target_menu = Some(menu_entity);
                navigation_state.exit_prompt_closes_menu_system = true;
                spawn_exit_unsaved_modal(&mut commands, menu_entity, &asset_server, menu_root.gate);
            } else {
                navigation_state.pending_exit_menu = Some(menu_entity);
                navigation_state.pending_exit_closes_menu_system = true;
            }
            continue;
        }
    }

    let mut active_menus = HashSet::new();
    let mut selected_indices_by_menu = HashMap::new();
    let mut footer_horizontal_nav_menus = HashSet::new();
    for (menu_entity, menu_stack, menu_root, selectable_menu) in menu_query.iter() {
        if interaction_gate_allows_for_owner(
            Some(&menu_root.gate),
            pause_state,
            &capture_query,
            menu_entity,
        )
            && layer::active_layer_kind_for_owner(&active_layers, menu_entity) == UiLayerKind::Base
        {
            active_menus.insert(menu_entity);
            selected_indices_by_menu.insert(menu_entity, selectable_menu.selected_index);
            if menu_stack.current_page() == Some(MenuPage::Video)
                && selectable_menu.selected_index >= VIDEO_FOOTER_OPTION_START_INDEX
                && selectable_menu.selected_index
                    < VIDEO_FOOTER_OPTION_START_INDEX + VIDEO_FOOTER_OPTION_COUNT
                && !tabbed_focus.is_tabs_focused(menu_entity)
            {
                footer_horizontal_nav_menus.insert(menu_entity);
            }
        }
    }

    let pending_shortcuts =
        selector::collect_shortcut_commands(&keyboard_input, &active_menus, &option_shortcut_query);
    for (menu_entity, option_command) in pending_shortcuts {
        menu_intents.write(MenuIntent::TriggerCommand {
            menu_entity,
            command: option_command.0,
        });
    }

    if activate_right || activate_left {
        let mut directional_targets = HashSet::new();
        for (selectable, option_command, cycler) in option_command_query.iter() {
            if !active_menus.contains(&selectable.menu_entity) {
                continue;
            }
            if footer_horizontal_nav_menus.contains(&selectable.menu_entity) {
                continue;
            }
            if tabbed_focus.is_tabs_focused(selectable.menu_entity) {
                continue;
            }
            let Some(selected_index) = selected_indices_by_menu.get(&selectable.menu_entity).copied()
            else {
                continue;
            };
            if selectable.index != selected_index {
                continue;
            }

            let is_selector =
                cycler.is_some() || matches!(option_command.0, MenuCommand::ToggleResolutionDropdown);
            let is_back = matches!(option_command.0, MenuCommand::Pop);
            let activate = (activate_right && !is_selector) || (activate_left && is_back);
            if !activate {
                continue;
            }
            if !directional_targets.insert(selectable.menu_entity) {
                continue;
            }

            menu_intents.write(MenuIntent::TriggerCommand {
                menu_entity: selectable.menu_entity,
                command: option_command.0.clone(),
            });
        }
    }
}
