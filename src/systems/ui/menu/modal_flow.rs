use super::*;
use crate::{
    entities::{
        sprites::compound::HollowRectangle,
        text::{scaled_font_size, TextRaw},
    },
    systems::colors::SYSTEM_MENU_COLOR,
};
use bevy::sprite::Anchor;
use std::collections::HashMap;

pub(super) fn any_video_modal_open(modal_query: &Query<(), With<VideoModalRoot>>) -> bool {
    !modal_query.is_empty()
}

fn spawn_video_modal_base(
    commands: &mut Commands,
    menu_entity: Entity,
    name: &str,
    asset_server: &Res<AssetServer>,
    gate: UiInputPolicy,
) -> Entity {
    let mut modal_entity = None;
    commands.entity(menu_entity).with_children(|parent| {
        modal_entity = Some(
            parent
                .spawn((
                    Name::new(name.to_string()),
                    MenuPageContent,
                    VideoModalRoot,
                    MenuSurface::new(menu_entity).with_layer(UiLayerKind::Modal),
                    gate,
                    system_menu::switch_audio_pallet(asset_server, SystemMenuSounds::Switch),
                    SelectableMenu::new(
                        0,
                        vec![KeyCode::ArrowLeft, KeyCode::ArrowUp],
                        vec![KeyCode::ArrowRight, KeyCode::ArrowDown],
                        vec![KeyCode::Enter],
                        true,
                    ),
                    Transform::from_xyz(0.0, 0.0, VIDEO_MODAL_PANEL_Z),
                ))
                .with_children(|modal| {
                    modal.spawn((
                        Name::new("video_modal_underlay_dimmer"),
                        Sprite::from_color(
                            Color::srgba(0.0, 0.0, 0.0, VIDEO_MODAL_DIM_ALPHA),
                            Vec2::splat(VIDEO_MODAL_DIM_SIZE),
                        ),
                        Transform::from_xyz(0.0, 0.0, VIDEO_MODAL_DIM_Z),
                    ));
                    modal.spawn((
                        Name::new("video_modal_panel"),
                        Sprite::from_color(Color::BLACK, VIDEO_MODAL_PANEL_SIZE),
                        Transform::from_xyz(0.0, 0.0, 0.0),
                    ));
                    modal.spawn((
                        Name::new("video_modal_border"),
                        HollowRectangle {
                            dimensions: VIDEO_MODAL_PANEL_SIZE - Vec2::splat(14.0),
                            thickness: 2.0,
                            color: SYSTEM_MENU_COLOR,
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, VIDEO_MODAL_BORDER_Z),
                    ));
                })
                .id(),
        );
    });
    modal_entity.expect("video modal entity should be spawned")
}

fn spawn_video_modal_option(
    commands: &mut Commands,
    modal_entity: Entity,
    gate: UiInputPolicy,
    asset_server: &Res<AssetServer>,
    button: VideoModalButton,
    index: usize,
    x: f32,
    label: &'static str,
) {
    commands.entity(modal_entity).with_children(|modal| {
        let option_entity = system_menu::spawn_option(
            modal,
            label,
            x,
            VIDEO_MODAL_OPTIONS_Y,
            modal_entity,
            index,
            system_menu::SystemMenuOptionVisualStyle::default()
                .with_indicator_offset(VIDEO_MODAL_OPTION_INDICATOR_X),
        );
        modal.commands().entity(option_entity).insert((
            Name::new(format!("video_modal_option_{index}")),
            MenuPageContent,
            gate,
            button,
            Clickable::with_region(vec![SystemMenuActions::Activate], VIDEO_MODAL_OPTION_REGION),
            system_menu::click_audio_pallet(asset_server, SystemMenuSounds::Click),
        ));
    });
}

pub(super) fn spawn_apply_confirm_modal(
    commands: &mut Commands,
    menu_entity: Entity,
    asset_server: &Res<AssetServer>,
    gate: UiInputPolicy,
) {
    let modal_entity = spawn_video_modal_base(
        commands,
        menu_entity,
        "video_apply_confirm_modal",
        asset_server,
        gate,
    );
    commands.entity(modal_entity).insert(VideoApplyConfirmModal);

    commands.entity(modal_entity).with_children(|modal| {
        modal.spawn((
            Name::new("video_apply_confirm_title"),
            TextRaw,
            Text2d::new("Apply these video settings?"),
            TextFont {
                font_size: scaled_font_size(20.0),
                weight: FontWeight::BOLD,
                ..default()
            },
            TextColor(SYSTEM_MENU_COLOR),
            Anchor::CENTER,
            Transform::from_xyz(0.0, 48.0, VIDEO_MODAL_TEXT_Z),
        ));
        modal.spawn((
            Name::new("video_apply_confirm_countdown"),
            VideoApplyCountdownText,
            TextRaw,
            Text2d::new("Reverting in 30"),
            TextFont {
                font_size: scaled_font_size(16.0),
                ..default()
            },
            TextColor(SYSTEM_MENU_COLOR),
            Anchor::CENTER,
            Transform::from_xyz(0.0, 8.0, VIDEO_MODAL_TEXT_Z),
        ));
    });

    spawn_video_modal_option(
        commands,
        modal_entity,
        gate,
        asset_server,
        VideoModalButton::ApplyKeep,
        0,
        -VIDEO_MODAL_OPTIONS_SPREAD_X,
        "KEEP [y]",
    );
    spawn_video_modal_option(
        commands,
        modal_entity,
        gate,
        asset_server,
        VideoModalButton::ApplyRevert,
        1,
        VIDEO_MODAL_OPTIONS_SPREAD_X,
        "REVERT [N/⌫]",
    );
}

pub(super) fn spawn_exit_unsaved_modal(
    commands: &mut Commands,
    menu_entity: Entity,
    asset_server: &Res<AssetServer>,
    gate: UiInputPolicy,
) {
    let modal_entity = spawn_video_modal_base(
        commands,
        menu_entity,
        "video_unsaved_exit_modal",
        asset_server,
        gate,
    );
    commands.entity(modal_entity).insert(VideoExitUnsavedModal);

    commands.entity(modal_entity).with_children(|modal| {
        modal.spawn((
            Name::new("video_unsaved_exit_title"),
            TextRaw,
            Text2d::new("Exit without saving changes?"),
            TextFont {
                font_size: scaled_font_size(20.0),
                weight: FontWeight::BOLD,
                ..default()
            },
            TextColor(SYSTEM_MENU_COLOR),
            Anchor::CENTER,
            Transform::from_xyz(0.0, 32.0, VIDEO_MODAL_TEXT_Z),
        ));
        modal.spawn((
            Name::new("video_unsaved_exit_hint"),
            TextRaw,
            Text2d::new("Unsaved settings will be discarded."),
            TextFont {
                font_size: scaled_font_size(14.0),
                ..default()
            },
            TextColor(SYSTEM_MENU_COLOR),
            Anchor::CENTER,
            Transform::from_xyz(0.0, -2.0, VIDEO_MODAL_TEXT_Z),
        ));
    });

    spawn_video_modal_option(
        commands,
        modal_entity,
        gate,
        asset_server,
        VideoModalButton::ExitWithoutSaving,
        0,
        -VIDEO_MODAL_OPTIONS_SPREAD_X,
        "EXIT [y]",
    );
    spawn_video_modal_option(
        commands,
        modal_entity,
        gate,
        asset_server,
        VideoModalButton::ExitCancel,
        1,
        VIDEO_MODAL_OPTIONS_SPREAD_X,
        "CANCEL [N/⌫]",
    );
}

fn close_video_modals(commands: &mut Commands, modal_query: &Query<Entity, With<VideoModalRoot>>) {
    for modal_entity in modal_query.iter() {
        commands.entity(modal_entity).despawn_related::<Children>();
        commands.entity(modal_entity).despawn();
    }
}

pub(super) fn handle_video_modal_shortcuts(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    interaction_state: Res<UiInteractionState>,
    modal_query: Query<
        (
            Entity,
            &UiLayer,
            Option<&VideoApplyConfirmModal>,
            Option<&VideoExitUnsavedModal>,
        ),
        With<VideoModalRoot>,
    >,
    mut menu_intents: MessageWriter<MenuIntent>,
) {
    let yes_pressed = keyboard_input.just_pressed(KeyCode::KeyY);
    let no_pressed = keyboard_input.just_pressed(KeyCode::KeyN);
    let cancel_pressed = keyboard_input.just_pressed(KeyCode::Backspace)
        || keyboard_input.just_pressed(KeyCode::Escape);

    if !(yes_pressed || no_pressed || cancel_pressed) {
        return;
    }

    let active_layers = &interaction_state.active_layers_by_owner;
    let mut modal_kind_by_owner: HashMap<Entity, (bool, bool)> = HashMap::new();
    for (modal_entity, ui_layer, is_apply_modal, is_exit_modal) in modal_query.iter() {
        if layer::active_layer_kind_for_owner(active_layers, ui_layer.owner) != UiLayerKind::Modal
            || !layer::is_active_layer_entity_for_owner(active_layers, ui_layer.owner, modal_entity)
        {
            continue;
        }
        modal_kind_by_owner.insert(
            ui_layer.owner,
            (is_apply_modal.is_some(), is_exit_modal.is_some()),
        );
    }
    for owner in layer::ordered_active_owners_by_kind(active_layers, UiLayerKind::Modal) {
        let Some((is_apply_modal, is_exit_modal)) = modal_kind_by_owner.get(&owner).copied() else {
            continue;
        };

        let target_button = if is_apply_modal {
            if yes_pressed {
                Some(VideoModalButton::ApplyKeep)
            } else if no_pressed || cancel_pressed {
                Some(VideoModalButton::ApplyRevert)
            } else {
                None
            }
        } else if is_exit_modal {
            if yes_pressed {
                Some(VideoModalButton::ExitWithoutSaving)
            } else if no_pressed || cancel_pressed {
                Some(VideoModalButton::ExitCancel)
            } else {
                None
            }
        } else {
            None
        };

        let Some(target_button) = target_button else {
            continue;
        };

        menu_intents.write(MenuIntent::TriggerModalButton(target_button));
        break;
    }
}

pub(super) fn handle_video_modal_button_commands(
    mut commands: Commands,
    interaction_state: Res<UiInteractionState>,
    modal_layer_query: Query<&UiLayer, With<VideoModalRoot>>,
    mut settings: ResMut<VideoSettingsState>,
    mut crt_settings: ResMut<CrtSettings>,
    mut screen_shake: ResMut<ScreenShakeState>,
    mut navigation_state: ResMut<MenuNavigationState>,
    mut button_query: Query<(
        Entity,
        &Selectable,
        &VideoModalButton,
        &mut Clickable<SystemMenuActions>,
        Option<&TransientAudioPallet<SystemMenuSounds>>,
    )>,
    modal_query: Query<Entity, With<VideoModalRoot>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut main_camera_query: Query<
        (
            &mut Bloom,
            &mut Tonemapping,
            &mut DebandDither,
            &mut Fxaa,
            &mut ContrastAdaptiveSharpening,
            &mut ChromaticAberration,
            &mut Msaa,
        ),
        With<MainCamera>,
    >,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
) {
    // Query contract:
    // - modal-layer ownership lookup is read-only (`modal_layer_query`).
    // - button click consumption/mutation is isolated to `button_query`.
    // Active-layer arbitration keeps modal button resolution owner-scoped.
    let active_layers = &interaction_state.active_layers_by_owner;

    let mut selected_button: Option<(Entity, VideoModalButton, usize, u64, Entity)> = None;
    for (entity, selectable, button, mut clickable, _) in button_query.iter_mut() {
        if !clickable.triggered {
            continue;
        }
        clickable.triggered = false;

        let modal_entity = selectable.menu_entity;
        let Ok(modal_layer) = modal_layer_query.get(modal_entity) else {
            continue;
        };
        if layer::active_layer_kind_for_owner(active_layers, modal_layer.owner)
            != UiLayerKind::Modal
        {
            continue;
        }
        if !layer::is_active_layer_entity_for_owner(active_layers, modal_layer.owner, modal_entity)
        {
            continue;
        }

        let candidate = (
            entity,
            *button,
            selectable.index,
            entity.to_bits(),
            modal_layer.owner,
        );
        let replace = match selected_button {
            Some((_, _, best_index, best_rank, best_owner)) => {
                let candidate_owner_rank = candidate.4.index();
                let best_owner_rank = best_owner.index();
                candidate_owner_rank < best_owner_rank
                    || (candidate_owner_rank == best_owner_rank
                        && (candidate.2 < best_index
                            || (candidate.2 == best_index && candidate.3 > best_rank)))
            }
            None => true,
        };
        if replace {
            selected_button = Some(candidate);
        }
    }

    let Some((selected_entity, button, _, _, _)) = selected_button else {
        return;
    };

    if let Ok((_, _, _, _, Some(click_pallet))) = button_query.get_mut(selected_entity) {
        TransientAudioPallet::play_transient_audio(
            selected_entity,
            &mut commands,
            click_pallet,
            SystemMenuSounds::Click,
            dilation.0,
            &mut audio_query,
        );
    }

    match button {
        VideoModalButton::ApplyKeep => {
            settings.saved = settings.pending;
            settings.revert_snapshot = None;
            settings.apply_timer = None;
        }
        VideoModalButton::ApplyRevert => {
            if let Some(snapshot) = settings.revert_snapshot.take() {
                settings.pending = snapshot;
                if let Ok(mut window) = primary_window.single_mut() {
                    apply_snapshot_to_window(&mut window, snapshot);
                }
                apply_snapshot_to_post_processing(
                    snapshot,
                    &mut crt_settings,
                    &mut screen_shake,
                    &mut main_camera_query,
                );
            }
            settings.apply_timer = None;
        }
        VideoModalButton::ExitWithoutSaving => {
            settings.pending = settings.saved;
            navigation_state.pending_exit_menu = navigation_state.exit_prompt_target_menu.take();
            navigation_state.pending_exit_closes_menu_system =
                navigation_state.exit_prompt_closes_menu_system;
            navigation_state.exit_prompt_closes_menu_system = false;
        }
        VideoModalButton::ExitCancel => {
            navigation_state.exit_prompt_target_menu = None;
            navigation_state.pending_exit_menu = None;
            navigation_state.exit_prompt_closes_menu_system = false;
            navigation_state.pending_exit_closes_menu_system = false;
        }
    }

    close_video_modals(&mut commands, &modal_query);
}

pub(super) fn update_apply_confirmation_countdown(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut settings: ResMut<VideoSettingsState>,
    mut crt_settings: ResMut<CrtSettings>,
    mut screen_shake: ResMut<ScreenShakeState>,
    mut countdown_text_query: Query<&mut Text2d, With<VideoApplyCountdownText>>,
    modal_query: Query<Entity, With<VideoModalRoot>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut main_camera_query: Query<
        (
            &mut Bloom,
            &mut Tonemapping,
            &mut DebandDither,
            &mut Fxaa,
            &mut ContrastAdaptiveSharpening,
            &mut ChromaticAberration,
            &mut Msaa,
        ),
        With<MainCamera>,
    >,
) {
    let Some(timer) = settings.apply_timer.as_mut() else {
        return;
    };

    timer.tick(time.delta());
    let remaining = (30.0 - timer.elapsed_secs()).ceil().max(0.0) as i32;
    for mut text in countdown_text_query.iter_mut() {
        text.0 = format!("Reverting in {remaining}");
    }

    if !timer.is_finished() {
        return;
    }

    if let Some(snapshot) = settings.revert_snapshot.take() {
        settings.pending = snapshot;
        if let Ok(mut window) = primary_window.single_mut() {
            apply_snapshot_to_window(&mut window, snapshot);
        }
        apply_snapshot_to_post_processing(
            snapshot,
            &mut crt_settings,
            &mut screen_shake,
            &mut main_camera_query,
        );
    }
    settings.apply_timer = None;
    close_video_modals(&mut commands, &modal_query);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::IntoSystem;

    #[test]
    fn modal_flow_systems_initialize_without_query_alias_panics() {
        let mut world = World::new();

        let mut shortcuts_system = IntoSystem::into_system(handle_video_modal_shortcuts);
        shortcuts_system.initialize(&mut world);

        let mut button_commands_system =
            IntoSystem::into_system(handle_video_modal_button_commands);
        button_commands_system.initialize(&mut world);

        let mut countdown_system = IntoSystem::into_system(update_apply_confirmation_countdown);
        countdown_system.initialize(&mut world);
    }

    #[test]
    fn modal_shortcuts_pick_deterministic_owner_order() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_message::<MenuIntent>();
        app.add_systems(Update, handle_video_modal_shortcuts);

        let owner_low = app.world_mut().spawn_empty().id();
        let owner_high = app.world_mut().spawn_empty().id();
        assert!(owner_low.index() < owner_high.index());

        app.world_mut().spawn((
            VideoModalRoot,
            UiLayer::new(owner_high, UiLayerKind::Modal),
            Visibility::Visible,
            VideoExitUnsavedModal,
        ));
        app.world_mut().spawn((
            VideoModalRoot,
            UiLayer::new(owner_low, UiLayerKind::Modal),
            Visibility::Visible,
            VideoApplyConfirmModal,
        ));

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyY);
        app.update();

        let mut reader = app
            .world_mut()
            .resource_mut::<Messages<MenuIntent>>()
            .get_cursor();
        let intents: Vec<MenuIntent> = reader
            .read(app.world().resource::<Messages<MenuIntent>>())
            .cloned()
            .collect();
        assert_eq!(intents.len(), 1);
        assert!(matches!(
            intents[0],
            MenuIntent::TriggerModalButton(VideoModalButton::ApplyKeep)
        ));
    }
}
