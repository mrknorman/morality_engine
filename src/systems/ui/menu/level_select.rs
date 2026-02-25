use std::collections::HashMap;

use bevy::{prelude::*, sprite::Anchor};

use super::{
    level_select_catalog::{
        self, LevelSelectExpansionState, LevelSelectNodeId, LevelSelectPlayableScene,
        LevelSelectVisibleRow,
        LevelSelectVisibleRowKind,
    },
    *,
};
use crate::{
    data::{states::{DilemmaPhase, GameState, MainState}, stats::GameStats},
    entities::{
        sprites::compound::HollowRectangle,
        text::{scaled_font_size, TextRaw},
    },
    scenes::{runtime::SceneNavigator, dilemma::content::DilemmaScene, Scene, SceneFlowMode, SceneQueue},
    startup::system_menu,
    systems::{
        interaction::{
            Draggable, InteractionVisualState, UiInputCaptureOwner, UiInputCaptureToken,
            UiInputPolicy, Hoverable,
        },
        ui::{
            scroll::{
                focus_scroll_offset_to_row, row_visible_in_viewport, ScrollAxis,
                ScrollFocusFollowLock, ScrollState, ScrollableRoot,
            },
            search_box::{SearchBox, SearchBoxConfig, SearchBoxQueryChanged},
            text_input_box::{TextInputBoxCaretState, TextInputBoxFocus, TextInputBoxStyle},
            window::{
                UiWindow, UiWindowContent, UiWindowContentMetrics, UiWindowOverflowPolicy,
                UiWindowTitle,
            },
        },
    },
};

const LEVEL_SELECT_OVERLAY_DIM_ALPHA: f32 = 0.8;
const LEVEL_SELECT_OVERLAY_DIM_SIZE: f32 = 6000.0;
const LEVEL_SELECT_OVERLAY_DIM_Z: f32 = -5.0;

const LEVEL_SELECT_WINDOW_SIZE: Vec2 = Vec2::new(690.0, 440.0);
const LEVEL_SELECT_WINDOW_Z: f32 = 0.4;

const LEVEL_SELECT_SEARCH_ROW_Y: f32 = 186.0;
const LEVEL_SELECT_SEARCH_HINT_X: f32 = -320.0;
const LEVEL_SELECT_SEARCH_BOX_X: f32 = 128.0;
const LEVEL_SELECT_SEARCH_BOX_SIZE: Vec2 = Vec2::new(384.0, 24.0);
const LEVEL_SELECT_SEARCH_FONT_SIZE: f32 = scaled_font_size(11.0);

const LEVEL_SELECT_LIST_X: f32 = -320.0;
const LEVEL_SELECT_LIST_ROW_STEP: f32 = 22.0;
const LEVEL_SELECT_TREE_INDENT: f32 = 14.0;
const LEVEL_SELECT_ROW_REGION: Vec2 = Vec2::new(630.0, 20.0);
const LEVEL_SELECT_SELECTION_INDICATOR_OFFSET_X: f32 = 500.0;
const LEVEL_SELECT_FONT_SIZE: f32 = scaled_font_size(16.0);
const LEVEL_SELECT_SELECTED_FONT_SIZE: f32 = scaled_font_size(19.0);
const LEVEL_SELECT_LIST_CONTENT_TOP_Y: f32 = 146.0;
const LEVEL_SELECT_WINDOW_SCROLL_LEADING_PADDING: f32 =
    LEVEL_SELECT_WINDOW_SIZE.y * 0.5 - LEVEL_SELECT_LIST_CONTENT_TOP_Y;
const LEVEL_SELECT_WINDOW_SCROLL_TRAILING_PADDING: f32 = 12.0;
const LEVEL_SELECT_LAUNCH_MODAL_SIZE: Vec2 = Vec2::new(560.0, 250.0);
const LEVEL_SELECT_LAUNCH_MODAL_Z: f32 = 2.0;
const LEVEL_SELECT_LAUNCH_MODAL_DIM_Z: f32 = -0.05;
const LEVEL_SELECT_LAUNCH_MODAL_OPTIONS_Y: f32 = -62.0;
const LEVEL_SELECT_LAUNCH_MODAL_OPTIONS_SPREAD_X: f32 = 150.0;
const LEVEL_SELECT_LAUNCH_MODAL_OPTION_REGION: Vec2 = Vec2::new(220.0, 38.0);
const LEVEL_SELECT_LAUNCH_MODAL_OPTION_INDICATOR_X: f32 = 84.0;

#[derive(Component)]
pub(super) struct LevelSelectOverlay;

#[derive(Component, Clone, Copy)]
pub(super) struct LevelSelectScrollRow {
    pub(super) index: usize,
    pub(super) folder_id: Option<LevelSelectNodeId>,
}

#[derive(Component)]
pub(super) struct LevelSelectSearchBox {
    owner: Entity,
}

#[derive(Component, Clone, Copy)]
pub(super) struct LevelSelectLaunchModal {
    owner: Entity,
    scene: DilemmaScene,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum LevelSelectLaunchModalOption {
    ContinueCampaign,
    PlayOnce,
    Cancel,
}

#[derive(Component)]
pub(super) struct LevelSelectRuntime {
    expansion: LevelSelectExpansionState,
    visible_rows: Vec<LevelSelectVisibleRow>,
    query_normalized: String,
    window_entity: Entity,
    rows_root: Entity,
    dirty: bool,
    scroll_sync_pending: bool,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LevelSelectFolderToggleRequested {
    pub overlay_entity: Entity,
    pub folder_id: LevelSelectNodeId,
}

#[derive(Resource, Default)]
pub(super) struct LevelUnlockState {
    reached_in_campaign: Vec<Scene>,
}

fn level_select_scene_unlocked(scene: LevelSelectPlayableScene, unlock_state: &LevelUnlockState) -> bool {
    let scene = match scene {
        LevelSelectPlayableScene::Dilemma(scene) => Scene::Dilemma(scene),
        LevelSelectPlayableScene::Dialogue(scene) => Scene::Dialogue(scene),
    };
    cfg!(debug_assertions)
        || unlock_state
            .reached_in_campaign
            .iter()
            .any(|reached| *reached == scene)
}

fn level_select_row_center_y(index: usize) -> f32 {
    LEVEL_SELECT_LIST_CONTENT_TOP_Y - (index as f32 + 0.5) * LEVEL_SELECT_LIST_ROW_STEP
}

fn level_select_last_row_bottom_y(row_count: usize) -> f32 {
    if row_count == 0 {
        LEVEL_SELECT_LIST_CONTENT_TOP_Y - LEVEL_SELECT_ROW_REGION.y * 0.5
    } else {
        level_select_row_center_y(row_count - 1) - LEVEL_SELECT_ROW_REGION.y * 0.5
    }
}

fn level_select_preferred_inner_size(row_count: usize) -> Vec2 {
    let content_top = LEVEL_SELECT_SEARCH_ROW_Y + LEVEL_SELECT_SEARCH_BOX_SIZE.y * 0.5;
    let content_bottom = level_select_last_row_bottom_y(row_count);
    let measured_height = content_top - content_bottom + 16.0;
    let scroll_required_height = LEVEL_SELECT_WINDOW_SCROLL_LEADING_PADDING
        + row_count as f32 * LEVEL_SELECT_LIST_ROW_STEP
        + LEVEL_SELECT_WINDOW_SCROLL_TRAILING_PADDING;
    let height = measured_height
        .max(scroll_required_height)
        .max(LEVEL_SELECT_WINDOW_SIZE.y);
    Vec2::new(LEVEL_SELECT_WINDOW_SIZE.x, height)
}

fn level_select_window_content_metrics(row_count: usize) -> UiWindowContentMetrics {
    UiWindowContentMetrics {
        min_inner: LEVEL_SELECT_WINDOW_SIZE,
        preferred_inner: level_select_preferred_inner_size(row_count),
        max_inner: None,
    }
}

fn level_select_visible_rows(
    expansion: &LevelSelectExpansionState,
    normalized_query: &str,
) -> Vec<LevelSelectVisibleRow> {
    let root = level_select_catalog::level_select_catalog_root();
    level_select_catalog::visible_rows_for_query(&root, expansion, normalized_query)
}

fn level_select_folder_label(
    row: &LevelSelectVisibleRow,
    expansion: &LevelSelectExpansionState,
    query_active: bool,
) -> String {
    let expanded = query_active || expansion.is_expanded(row.id);
    if expanded {
        format!("[-] {}/", row.label)
    } else {
        format!("[+] {}/", row.label)
    }
}

fn spawn_level_select_rows(
    rows_parent: &mut ChildSpawnerCommands<'_>,
    asset_server: &Res<AssetServer>,
    overlay_entity: Entity,
    unlock_state: &LevelUnlockState,
    rows: &[LevelSelectVisibleRow],
    expansion: &LevelSelectExpansionState,
    query_active: bool,
) {
    for (index, row) in rows.iter().enumerate() {
        let (label, folder_id, command) = match row.kind {
            LevelSelectVisibleRowKind::Folder => (
                level_select_folder_label(row, expansion, query_active),
                Some(row.id),
                MenuCommand::None,
            ),
            LevelSelectVisibleRowKind::File(file) => {
                let unlocked = level_select_scene_unlocked(file.scene, unlock_state);
                let base_label = match file.scene {
                    LevelSelectPlayableScene::Dilemma(_) => file.file_name.to_string(),
                    LevelSelectPlayableScene::Dialogue(_) => format!("{}.log", file.file_name),
                };
                (
                    if unlocked {
                        base_label
                    } else {
                        format!("{base_label} [locked]")
                    },
                    None,
                    if unlocked {
                        match file.scene {
                            LevelSelectPlayableScene::Dilemma(scene) => {
                                MenuCommand::LaunchDilemmaFromLevelSelect(scene)
                            }
                            LevelSelectPlayableScene::Dialogue(scene) => {
                                MenuCommand::StartSingleDialogue(scene)
                            }
                        }
                    } else {
                        MenuCommand::None
                    },
                )
            }
        };

        let option_entity = system_menu::spawn_option(
            rows_parent,
            label,
            LEVEL_SELECT_LIST_X + row.depth as f32 * LEVEL_SELECT_TREE_INDENT,
            level_select_row_center_y(index),
            overlay_entity,
            index,
            system_menu::SystemMenuOptionVisualStyle::default()
                .with_indicator_offset(LEVEL_SELECT_SELECTION_INDICATOR_OFFSET_X),
        );

        rows_parent.commands().entity(option_entity).insert((
            Name::new(format!("level_select_row_{index}")),
            LevelSelectScrollRow { index, folder_id },
            Clickable::with_region(vec![SystemMenuActions::Activate], LEVEL_SELECT_ROW_REGION),
            MenuOptionCommand(command),
            system_menu::click_audio_pallet(asset_server, SystemMenuSounds::Click),
            UiInputPolicy::CapturedOnly,
            Anchor::CENTER_LEFT,
            TextLayout {
                justify: Justify::Left,
                ..default()
            },
        ));
    }
}

fn spawn_level_select_launch_modal_option(
    commands: &mut Commands,
    modal_entity: Entity,
    gate: UiInputPolicy,
    asset_server: &Res<AssetServer>,
    option: LevelSelectLaunchModalOption,
    index: usize,
    x: f32,
    label: &'static str,
) {
    commands.entity(modal_entity).with_children(|modal| {
        let option_entity = system_menu::spawn_option(
            modal,
            label,
            x,
            LEVEL_SELECT_LAUNCH_MODAL_OPTIONS_Y,
            modal_entity,
            index,
            system_menu::SystemMenuOptionVisualStyle::default()
                .with_indicator_offset(LEVEL_SELECT_LAUNCH_MODAL_OPTION_INDICATOR_X),
        );
        modal.commands().entity(option_entity).insert((
            Name::new(format!("level_select_launch_modal_option_{index}")),
            MenuPageContent,
            gate,
            option,
            Clickable::with_region(
                vec![SystemMenuActions::Activate],
                LEVEL_SELECT_LAUNCH_MODAL_OPTION_REGION,
            ),
            system_menu::click_audio_pallet(asset_server, SystemMenuSounds::Click),
        ));
    });
}

pub(super) fn spawn_level_select_launch_modal(
    commands: &mut Commands,
    owner: Entity,
    scene: DilemmaScene,
    gate: UiInputPolicy,
    asset_server: &Res<AssetServer>,
    existing_modal_query: &Query<(), With<LevelSelectLaunchModal>>,
) {
    if !cfg!(debug_assertions) || !existing_modal_query.is_empty() {
        return;
    }

    let mut modal_entity = None;
    commands.entity(owner).with_children(|parent| {
        modal_entity = Some(
            parent
                .spawn((
                    Name::new("level_select_launch_modal"),
                    MenuPageContent,
                    LevelSelectLaunchModal { owner, scene },
                    MenuSurface::new(owner).with_layer(UiLayerKind::Modal),
                    gate,
                    SelectableMenu::new(
                        0,
                        vec![KeyCode::ArrowLeft, KeyCode::ArrowUp],
                        vec![KeyCode::ArrowRight, KeyCode::ArrowDown],
                        vec![KeyCode::Enter],
                        true,
                    ),
                    Transform::from_xyz(0.0, 0.0, LEVEL_SELECT_LAUNCH_MODAL_Z),
                ))
                .with_children(|modal| {
                    modal.spawn((
                        Name::new("level_select_launch_modal_dimmer"),
                        Sprite::from_color(
                            Color::srgba(0.0, 0.0, 0.0, LEVEL_SELECT_OVERLAY_DIM_ALPHA),
                            Vec2::splat(LEVEL_SELECT_OVERLAY_DIM_SIZE),
                        ),
                        Transform::from_xyz(0.0, 0.0, LEVEL_SELECT_LAUNCH_MODAL_DIM_Z),
                    ));
                    modal.spawn((
                        Name::new("level_select_launch_modal_panel"),
                        Sprite::from_color(Color::BLACK, LEVEL_SELECT_LAUNCH_MODAL_SIZE),
                        Transform::from_xyz(0.0, 0.0, 0.0),
                    ));
                    modal.spawn((
                        Name::new("level_select_launch_modal_border"),
                        HollowRectangle {
                            dimensions: LEVEL_SELECT_LAUNCH_MODAL_SIZE - Vec2::splat(14.0),
                            thickness: 2.0,
                            color: Color::WHITE,
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 0.01),
                    ));
                    modal.spawn((
                        Name::new("level_select_launch_modal_title"),
                        TextRaw,
                        Text2d::new("Launch selected dilemma"),
                        TextFont {
                            font_size: scaled_font_size(20.0),
                            weight: FontWeight::BOLD,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Anchor::CENTER,
                        Transform::from_xyz(0.0, 52.0, 0.02),
                    ));
                    modal.spawn((
                        Name::new("level_select_launch_modal_hint"),
                        TextRaw,
                        Text2d::new("Choose launch mode"),
                        TextFont {
                            font_size: scaled_font_size(13.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Anchor::CENTER,
                        Transform::from_xyz(0.0, 18.0, 0.02),
                    ));
                })
                .id(),
        );
    });

    let Some(modal_entity) = modal_entity else {
        return;
    };

    spawn_level_select_launch_modal_option(
        commands,
        modal_entity,
        gate,
        asset_server,
        LevelSelectLaunchModalOption::ContinueCampaign,
        0,
        -LEVEL_SELECT_LAUNCH_MODAL_OPTIONS_SPREAD_X,
        "CONTINUE [c]",
    );
    spawn_level_select_launch_modal_option(
        commands,
        modal_entity,
        gate,
        asset_server,
        LevelSelectLaunchModalOption::PlayOnce,
        1,
        0.0,
        "PLAY ONCE [p]",
    );
    spawn_level_select_launch_modal_option(
        commands,
        modal_entity,
        gate,
        asset_server,
        LevelSelectLaunchModalOption::Cancel,
        2,
        LEVEL_SELECT_LAUNCH_MODAL_OPTIONS_SPREAD_X,
        "CANCEL [esc]",
    );
}

pub(super) fn spawn_level_select_overlay(
    commands: &mut Commands,
    menu_root: &MenuRoot,
    asset_server: &Res<AssetServer>,
    unlock_state: &Res<LevelUnlockState>,
    existing_overlay_query: &Query<(), With<LevelSelectOverlay>>,
    offscreen_camera_query: &Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_transform_query: &Query<&GlobalTransform, With<MainCamera>>,
) {
    if menu_root.host != MenuHost::Main || !existing_overlay_query.is_empty() {
        return;
    }

    let Some(camera_translation) =
        super::camera::menu_camera_center(offscreen_camera_query, main_camera_transform_query)
    else {
        return;
    };

    let overlay_entity = system_menu::spawn_selectable_root(
        commands,
        asset_server,
        "main_menu_level_select_overlay",
        Vec3::new(camera_translation.x, camera_translation.y, system_menu::MENU_Z),
        SystemMenuSounds::Switch,
        SelectableMenu::new(
            0,
            vec![KeyCode::ArrowUp],
            vec![KeyCode::ArrowDown],
            vec![KeyCode::Enter, KeyCode::ArrowRight],
            true,
        )
        .with_click_activation(SelectableClickActivation::HoveredOnly),
    );
    commands.entity(overlay_entity).insert((
        Name::new("main_menu_level_select_overlay_root"),
        LevelSelectOverlay,
        MenuRoot {
            host: MenuHost::Main,
            gate: UiInputPolicy::CapturedOnly,
        },
        MenuStack::new(MenuPage::PauseRoot),
        UiInputPolicy::CapturedOnly,
        MenuSurface::new(overlay_entity)
            .with_layer(UiLayerKind::Base)
            .with_click_activation(SelectableClickActivation::HoveredOnly),
        DespawnOnExit(MainState::Menu),
    ));

    commands.entity(overlay_entity).with_children(|parent| {
        parent.spawn((
            Name::new("main_menu_level_select_dimmer"),
            UiInputCaptureToken,
            UiInputCaptureOwner::new(overlay_entity),
            Sprite::from_color(
                Color::srgba(0.0, 0.0, 0.0, LEVEL_SELECT_OVERLAY_DIM_ALPHA),
                Vec2::splat(LEVEL_SELECT_OVERLAY_DIM_SIZE),
            ),
            Transform::from_xyz(0.0, 0.0, LEVEL_SELECT_OVERLAY_DIM_Z),
        ));
    });

    let initial_expansion = LevelSelectExpansionState::default();
    let initial_rows = level_select_visible_rows(&initial_expansion, "");

    let window_entity = commands
        .spawn((
            Name::new("level_select_window"),
            Draggable::default(),
            UiWindow::new(
                Some(UiWindowTitle {
                    text: "LEVEL SELECT".to_string(),
                    ..default()
                }),
                HollowRectangle {
                    dimensions: LEVEL_SELECT_WINDOW_SIZE,
                    thickness: 2.0,
                    color: Color::WHITE,
                    ..default()
                },
                22.0,
                true,
                Some(overlay_entity),
            ),
            level_select_window_content_metrics(initial_rows.len()),
            UiWindowOverflowPolicy::AllowOverflow,
            UiInputPolicy::CapturedOnly,
            Transform::from_xyz(0.0, 0.0, LEVEL_SELECT_WINDOW_Z),
        ))
        .id();
    let content_root = commands
        .spawn((
            Name::new("level_select_window_content"),
            UiWindowContent::new(window_entity),
            Transform::default(),
        ))
        .id();
    commands.entity(window_entity).add_child(content_root);
    commands.entity(overlay_entity).add_child(window_entity);

    let mut rows_root = None;
    commands.entity(content_root).with_children(|content| {
        content.spawn((
            Name::new("level_select_search_hint"),
            TextRaw,
            Text2d::new("Search / select .dilem or .log entry:"),
            TextFont {
                font_size: scaled_font_size(12.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Anchor::CENTER_LEFT,
            Transform::from_xyz(LEVEL_SELECT_SEARCH_HINT_X, LEVEL_SELECT_SEARCH_ROW_Y, 0.24),
        ));
        content.spawn((
            Name::new("level_select_search_box"),
            LevelSelectSearchBox {
                owner: overlay_entity,
            },
            SearchBox::new(overlay_entity, UiLayerKind::Base).without_text_input_ui_layer(),
            SearchBoxConfig {
                placeholder: "type to filter folders/files".to_string(),
                ..default()
            },
            TextInputBoxFocus { focused: true },
            TextInputBoxStyle {
                size: LEVEL_SELECT_SEARCH_BOX_SIZE,
                font_size: LEVEL_SELECT_SEARCH_FONT_SIZE,
                padding: Vec2::new(10.0, 4.0),
                background_color: Color::NONE,
                border_color: Color::NONE,
                border_color_hovered: Color::NONE,
                border_color_focused: Color::NONE,
                ..default()
            },
            UiInputPolicy::CapturedOnly,
            Transform::from_xyz(LEVEL_SELECT_SEARCH_BOX_X, LEVEL_SELECT_SEARCH_ROW_Y, 0.24),
        ));
        content.spawn((
            Name::new("level_select_separator"),
            Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.45), Vec2::new(640.0, 1.0)),
            Transform::from_xyz(0.0, 166.0, 0.2),
        ));

        let content_entity = content
            .spawn((
                Name::new("level_select_rows_root"),
                Transform::default(),
            ))
            .id();
        rows_root = Some(content_entity);

        content
            .commands()
            .entity(content_entity)
            .with_children(|rows_parent| {
                spawn_level_select_rows(
                    rows_parent,
                    asset_server,
                    overlay_entity,
                    unlock_state.as_ref(),
                    &initial_rows,
                    &initial_expansion,
                    false,
                );
            });
    });

    let Some(rows_root) = rows_root else {
        return;
    };
    commands.entity(overlay_entity).insert(LevelSelectRuntime {
        expansion: initial_expansion,
        visible_rows: initial_rows,
        query_normalized: String::new(),
        window_entity,
        rows_root,
        dirty: false,
        scroll_sync_pending: false,
    });
}

pub(super) fn sync_level_select_search_query(
    mut query_changes: MessageReader<SearchBoxQueryChanged>,
    search_box_query: Query<&LevelSelectSearchBox>,
    mut runtime_query: Query<&mut LevelSelectRuntime, With<LevelSelectOverlay>>,
) {
    for changed in query_changes.read() {
        let Ok(search_box) = search_box_query.get(changed.entity) else {
            continue;
        };
        let Ok(mut runtime) = runtime_query.get_mut(search_box.owner) else {
            continue;
        };
        if runtime.query_normalized == changed.normalized {
            continue;
        }
        runtime.query_normalized = changed.normalized.clone();
        runtime.dirty = true;
    }
}

pub(super) fn handle_level_select_launch_modal_shortcuts(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    interaction_state: Res<UiInteractionState>,
    modal_query: Query<(Entity, &UiLayer), With<LevelSelectLaunchModal>>,
    mut option_query: Query<
        (
            &Selectable,
            &LevelSelectLaunchModalOption,
            &mut Clickable<SystemMenuActions>,
        ),
        With<LevelSelectLaunchModalOption>,
    >,
) {
    let requested_option = if keyboard_input.just_pressed(KeyCode::KeyC) {
        Some(LevelSelectLaunchModalOption::ContinueCampaign)
    } else if keyboard_input.just_pressed(KeyCode::KeyP) {
        Some(LevelSelectLaunchModalOption::PlayOnce)
    } else if keyboard_input.just_pressed(KeyCode::Escape)
        || keyboard_input.just_pressed(KeyCode::Backspace)
    {
        Some(LevelSelectLaunchModalOption::Cancel)
    } else {
        None
    };
    let Some(requested_option) = requested_option else {
        return;
    };

    let active_layers = &interaction_state.active_layers_by_owner;
    for owner in layer::ordered_active_owners_by_kind(active_layers, UiLayerKind::Modal) {
        let modal_entity = modal_query
            .iter()
            .find_map(|(modal_entity, ui_layer)| {
                if ui_layer.owner != owner {
                    return None;
                }
                if !layer::is_active_layer_entity_for_owner(active_layers, owner, modal_entity) {
                    return None;
                }
                Some(modal_entity)
            });
        let Some(modal_entity) = modal_entity else {
            continue;
        };

        for (selectable, option, mut clickable) in option_query.iter_mut() {
            if selectable.menu_entity != modal_entity || *option != requested_option {
                continue;
            }
            clickable.triggered = true;
            return;
        }
    }
}

pub(super) fn handle_level_select_launch_modal_option_commands(
    mut commands: Commands,
    interaction_state: Res<UiInteractionState>,
    mut option_query: Query<
        (
            Entity,
            &Selectable,
            &LevelSelectLaunchModalOption,
            &mut Clickable<SystemMenuActions>,
            Option<&TransientAudioPallet<SystemMenuSounds>>,
        ),
        With<LevelSelectLaunchModalOption>,
    >,
    modal_query: Query<(Entity, &LevelSelectLaunchModal, &UiLayer), With<LevelSelectLaunchModal>>,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
    mut scene_queue: ResMut<SceneQueue>,
    mut stats: ResMut<GameStats>,
    mut next_main_state: ResMut<NextState<MainState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_sub_state: ResMut<NextState<DilemmaPhase>>,
) {
    let active_layers = &interaction_state.active_layers_by_owner;
    let mut selected: Option<(
        Entity,
        LevelSelectLaunchModalOption,
        LevelSelectLaunchModal,
        Entity,
        u64,
    )> = None;

    for (option_entity, selectable, option, mut clickable, _) in option_query.iter_mut() {
        if !clickable.triggered {
            continue;
        }
        clickable.triggered = false;

        let Ok((modal_entity, modal_data, ui_layer)) = modal_query.get(selectable.menu_entity) else {
            continue;
        };
        if layer::active_layer_kind_for_owner(active_layers, ui_layer.owner) != UiLayerKind::Modal {
            continue;
        }
        if !layer::is_active_layer_entity_for_owner(active_layers, ui_layer.owner, modal_entity) {
            continue;
        }

        let candidate = (
            option_entity,
            *option,
            *modal_data,
            modal_entity,
            option_entity.to_bits(),
        );
        if selected
            .as_ref()
            .is_none_or(|(_, _, _, _, best_rank)| candidate.4 > *best_rank)
        {
            selected = Some(candidate);
        }
    }

    let Some((selected_entity, option, modal_data, modal_entity, _)) = selected else {
        return;
    };

    if let Ok((_, _, _, _, click_pallet)) = option_query.get_mut(selected_entity) {
        if let Some(click_pallet) = click_pallet {
            TransientAudioPallet::play_transient_audio(
                selected_entity,
                &mut commands,
                click_pallet,
                SystemMenuSounds::Click,
                dilation.0,
                &mut audio_query,
            );
        }
    }

    match option {
        LevelSelectLaunchModalOption::ContinueCampaign => {
            *stats = GameStats::default();
            scene_queue.configure_campaign_from_dilemma(modal_data.scene);
            SceneNavigator::next_state_vector_or_fallback(&mut scene_queue).set_state(
                &mut next_main_state,
                &mut next_game_state,
                &mut next_sub_state,
            );
            commands.entity(modal_data.owner).despawn_related::<Children>();
            commands.entity(modal_data.owner).despawn();
        }
        LevelSelectLaunchModalOption::PlayOnce => {
            *stats = GameStats::default();
            scene_queue.configure_single_level(modal_data.scene);
            SceneNavigator::next_state_vector_or_fallback(&mut scene_queue).set_state(
                &mut next_main_state,
                &mut next_game_state,
                &mut next_sub_state,
            );
            commands.entity(modal_data.owner).despawn_related::<Children>();
            commands.entity(modal_data.owner).despawn();
        }
        LevelSelectLaunchModalOption::Cancel => {
            commands.entity(modal_entity).despawn_related::<Children>();
            commands.entity(modal_entity).despawn();
        }
    }
}

pub(super) fn apply_level_select_folder_toggle_requests(
    mut toggle_requests: MessageReader<LevelSelectFolderToggleRequested>,
    mut runtime_query: Query<&mut LevelSelectRuntime, With<LevelSelectOverlay>>,
) {
    for request in toggle_requests.read() {
        let Ok(mut runtime) = runtime_query.get_mut(request.overlay_entity) else {
            continue;
        };
        runtime.expansion.toggle(request.folder_id);
        runtime.dirty = true;
    }
}

pub(super) fn focus_level_select_search_on_typed_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut search_box_query: Query<
        (
            &LevelSelectSearchBox,
            &mut TextInputBoxFocus,
            Option<&mut TextInputBoxCaretState>,
        ),
        With<SearchBox>,
    >,
    overlay_query: Query<(), With<LevelSelectOverlay>>,
) {
    if overlay_query.is_empty() {
        return;
    }

    fn is_typing_key(keycode: KeyCode) -> bool {
        matches!(
            keycode,
            KeyCode::KeyA
                | KeyCode::KeyB
                | KeyCode::KeyC
                | KeyCode::KeyD
                | KeyCode::KeyE
                | KeyCode::KeyF
                | KeyCode::KeyG
                | KeyCode::KeyH
                | KeyCode::KeyI
                | KeyCode::KeyJ
                | KeyCode::KeyK
                | KeyCode::KeyL
                | KeyCode::KeyM
                | KeyCode::KeyN
                | KeyCode::KeyO
                | KeyCode::KeyP
                | KeyCode::KeyQ
                | KeyCode::KeyR
                | KeyCode::KeyS
                | KeyCode::KeyT
                | KeyCode::KeyU
                | KeyCode::KeyV
                | KeyCode::KeyW
                | KeyCode::KeyX
                | KeyCode::KeyY
                | KeyCode::KeyZ
                | KeyCode::Digit0
                | KeyCode::Digit1
                | KeyCode::Digit2
                | KeyCode::Digit3
                | KeyCode::Digit4
                | KeyCode::Digit5
                | KeyCode::Digit6
                | KeyCode::Digit7
                | KeyCode::Digit8
                | KeyCode::Digit9
                | KeyCode::Space
                | KeyCode::Minus
                | KeyCode::Equal
                | KeyCode::Backspace
                | KeyCode::Delete
        )
    }

    let typed_input = keyboard_input.get_just_pressed().any(|key| is_typing_key(*key));
    if !typed_input {
        return;
    }

    for (search_box, mut focus, caret_state) in search_box_query.iter_mut() {
        if overlay_query.get(search_box.owner).is_err() {
            continue;
        }
        if !focus.focused {
            focus.focused = true;
        }
        if let Some(mut caret_state) = caret_state {
            caret_state.visible = true;
            caret_state.blink_timer.reset();
        }
    }
}

pub(super) fn rebuild_level_select_rows(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    unlock_state: Res<LevelUnlockState>,
    mut overlay_query: Query<
        (Entity, &mut LevelSelectRuntime, &mut SelectableMenu),
        With<LevelSelectOverlay>,
    >,
    children_query: Query<&Children>,
    mut window_metrics_query: Query<&mut UiWindowContentMetrics>,
) {
    for (overlay_entity, mut runtime, mut menu) in overlay_query.iter_mut() {
        if !runtime.dirty {
            continue;
        }
        runtime.dirty = false;

        let selected_row_id = runtime.visible_rows.get(menu.selected_index).map(|row| row.id);
        let next_rows = level_select_visible_rows(&runtime.expansion, &runtime.query_normalized);
        let query_active = !runtime.query_normalized.trim().is_empty();
        let row_count = next_rows.len();

        if let Ok(children) = children_query.get(runtime.rows_root) {
            for child in children.iter() {
                commands.entity(child).despawn_related::<Children>();
                commands.entity(child).despawn();
            }
        }

        let expansion = runtime.expansion.clone();
        commands
            .entity(runtime.rows_root)
            .with_children(|rows_parent| {
                spawn_level_select_rows(
                    rows_parent,
                    &asset_server,
                    overlay_entity,
                    unlock_state.as_ref(),
                    &next_rows,
                    &expansion,
                    query_active,
                );
            });

        if let Ok(mut metrics) = window_metrics_query.get_mut(runtime.window_entity) {
            metrics.preferred_inner = level_select_preferred_inner_size(row_count);
        }

        menu.selected_index = if row_count == 0 {
            0
        } else {
            let fallback = menu.selected_index.min(row_count - 1);
            selected_row_id
                .and_then(|selected_row_id| {
                    next_rows
                        .iter()
                        .position(|row| row.id == selected_row_id)
                })
                .unwrap_or(fallback)
        };

        runtime.visible_rows = next_rows;
        runtime.scroll_sync_pending = true;
    }
}

pub(super) fn track_campaign_reached_dilemmas(
    scene_queue: Res<SceneQueue>,
    mut unlock_state: ResMut<LevelUnlockState>,
) {
    if scene_queue.flow_mode() != SceneFlowMode::Campaign {
        return;
    }
    let scene = scene_queue.current_scene();
    if !matches!(scene, Scene::Dilemma(_) | Scene::Dialogue(_)) {
        return;
    }
    if !unlock_state
        .reached_in_campaign
        .iter()
        .any(|reached| *reached == scene)
    {
        unlock_state.reached_in_campaign.push(scene);
    }
}

pub(super) fn mark_level_select_dirty_on_unlock_change(
    unlock_state: Res<LevelUnlockState>,
    mut runtime_query: Query<&mut LevelSelectRuntime, With<LevelSelectOverlay>>,
) {
    if !unlock_state.is_changed() || cfg!(debug_assertions) {
        return;
    }
    for mut runtime in runtime_query.iter_mut() {
        runtime.dirty = true;
    }
}

pub(super) fn sync_level_select_scroll_focus_follow(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut overlay_query: Query<
        (Entity, &SelectableMenu, &mut LevelSelectRuntime),
        With<LevelSelectOverlay>,
    >,
    mut root_query: Query<
        (&ScrollableRoot, &mut ScrollState, &mut ScrollFocusFollowLock),
        With<ScrollableRoot>,
    >,
    mut previous_selection_by_owner: Local<HashMap<Entity, usize>>,
) {
    let keyboard_navigation = keyboard_input.just_pressed(KeyCode::ArrowUp)
        || keyboard_input.just_pressed(KeyCode::ArrowDown)
        || keyboard_input.just_pressed(KeyCode::PageUp)
        || keyboard_input.just_pressed(KeyCode::PageDown)
        || keyboard_input.just_pressed(KeyCode::Home)
        || keyboard_input.just_pressed(KeyCode::End);

    for (overlay_entity, menu, mut runtime) in overlay_query.iter_mut() {
        let row_count = runtime.visible_rows.len();
        let force_sync = runtime.scroll_sync_pending;
        let selection_changed =
            previous_selection_by_owner.insert(overlay_entity, menu.selected_index)
                != Some(menu.selected_index);
        if menu.selected_index >= row_count {
            if force_sync {
                runtime.scroll_sync_pending = false;
            }
            continue;
        }

        let mut synced = false;
        for (root, mut state, mut focus_lock) in root_query.iter_mut() {
            if root.owner != overlay_entity || root.axis != ScrollAxis::Vertical {
                continue;
            }
            if keyboard_navigation {
                focus_lock.manual_override = false;
            }
            if force_sync {
                focus_lock.manual_override = false;
            }
            if focus_lock.manual_override {
                break;
            }
            if !keyboard_navigation && !selection_changed && !force_sync {
                break;
            }

            focus_scroll_offset_to_row(
                &mut state,
                menu.selected_index,
                LEVEL_SELECT_LIST_ROW_STEP,
                LEVEL_SELECT_WINDOW_SCROLL_LEADING_PADDING,
            );
            synced = true;
            break;
        }
        if synced || force_sync {
            runtime.scroll_sync_pending = false;
        }
    }
}

pub(super) fn sync_level_select_option_hit_regions_to_viewport(
    root_query: Query<(&ScrollableRoot, &ScrollState), With<ScrollableRoot>>,
    mut option_query: Query<
        (
            &Selectable,
            &LevelSelectScrollRow,
            &mut Clickable<SystemMenuActions>,
        ),
        With<MenuOptionCommand>,
    >,
) {
    let mut adapter_state_by_owner = HashMap::new();
    for (root, state) in root_query.iter() {
        if root.axis != ScrollAxis::Vertical {
            continue;
        }
        adapter_state_by_owner.insert(root.owner, *state);
    }

    for (selectable, row, mut clickable) in option_query.iter_mut() {
        let Some(state) = adapter_state_by_owner.get(&selectable.menu_entity).copied() else {
            clickable.region = Some(LEVEL_SELECT_ROW_REGION);
            continue;
        };

        let visible = row_visible_in_viewport(
            &state,
            row.index,
            LEVEL_SELECT_LIST_ROW_STEP,
            LEVEL_SELECT_WINDOW_SCROLL_LEADING_PADDING,
        );
        clickable.region = if visible {
            Some(LEVEL_SELECT_ROW_REGION)
        } else {
            None
        };
    }
}

pub(super) fn sync_level_select_selection_font_growth(
    mut option_query: Query<
        (&InteractionVisualState, &Hoverable, &mut TextFont),
        (
            With<LevelSelectScrollRow>,
            With<system_menu::SystemMenuOption>,
            Without<VideoModalButton>,
        ),
    >,
) {
    for (state, hoverable, mut font) in option_query.iter_mut() {
        let target_size = if state.selected || hoverable.hovered {
            LEVEL_SELECT_SELECTED_FONT_SIZE
        } else {
            LEVEL_SELECT_FONT_SIZE
        };
        if (font.font_size - target_size).abs() > 0.001 {
            font.font_size = target_size;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::IntoSystem;

    use super::*;

    #[test]
    fn level_select_scroll_content_height_matches_row_geometry() {
        let row_count = 19;
        let preferred_inner = level_select_preferred_inner_size(row_count);
        let required_scroll_height = LEVEL_SELECT_WINDOW_SCROLL_LEADING_PADDING
            + row_count as f32 * LEVEL_SELECT_LIST_ROW_STEP
            + LEVEL_SELECT_WINDOW_SCROLL_TRAILING_PADDING;

        assert!(preferred_inner.y > LEVEL_SELECT_WINDOW_SIZE.y);
        assert!(preferred_inner.x >= LEVEL_SELECT_WINDOW_SIZE.x);
        assert!(preferred_inner.y >= required_scroll_height);
    }

    #[test]
    fn level_select_scroll_systems_initialize_without_query_alias_panics() {
        let mut world = World::new();

        let mut search_query_system = IntoSystem::into_system(sync_level_select_search_query);
        search_query_system.initialize(&mut world);

        let mut folder_toggle_system =
            IntoSystem::into_system(apply_level_select_folder_toggle_requests);
        folder_toggle_system.initialize(&mut world);

        let mut rebuild_system = IntoSystem::into_system(rebuild_level_select_rows);
        rebuild_system.initialize(&mut world);

        let mut focus_follow_system = IntoSystem::into_system(sync_level_select_scroll_focus_follow);
        focus_follow_system.initialize(&mut world);

        let mut hit_region_system =
            IntoSystem::into_system(sync_level_select_option_hit_regions_to_viewport);
        hit_region_system.initialize(&mut world);
    }

    #[test]
    fn folder_toggle_requests_mark_runtime_dirty_and_toggle_expansion() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<LevelSelectFolderToggleRequested>();
        let overlay = app
            .world_mut()
            .spawn((
                LevelSelectOverlay,
                SelectableMenu::new(0, vec![], vec![], vec![], true),
            ))
            .id();

        let root = level_select_catalog::level_select_catalog_root();
        let expansion = LevelSelectExpansionState::all_expanded(&root);
        let folder_id = LevelSelectNodeId("path_inaction");
        let window_entity = app.world_mut().spawn_empty().id();
        let rows_root = app.world_mut().spawn_empty().id();
        app.world_mut().entity_mut(overlay).insert(LevelSelectRuntime {
            expansion,
            visible_rows: vec![],
            query_normalized: String::new(),
            window_entity,
            rows_root,
            dirty: false,
            scroll_sync_pending: false,
        });

        app.world_mut().write_message(LevelSelectFolderToggleRequested {
            overlay_entity: overlay,
            folder_id,
        });

        let mut system = IntoSystem::into_system(apply_level_select_folder_toggle_requests);
        system.initialize(app.world_mut());
        system
            .run((), app.world_mut())
            .expect("folder toggle should run");
        system.apply_deferred(app.world_mut());

        let runtime = app
            .world()
            .entity(overlay)
            .get::<LevelSelectRuntime>()
            .expect("level select runtime");
        assert!(runtime.dirty);
        assert!(!runtime.expansion.is_expanded(folder_id));
    }

    #[test]
    fn scroll_focus_follow_honors_pending_sync_without_keyboard_navigation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ButtonInput<KeyCode>>();

        let overlay = app
            .world_mut()
            .spawn((
                LevelSelectOverlay,
                SelectableMenu::new(
                    3,
                    vec![KeyCode::ArrowUp],
                    vec![KeyCode::ArrowDown],
                    vec![KeyCode::Enter],
                    true,
                ),
            ))
            .id();

        let window_entity = app.world_mut().spawn_empty().id();
        let rows_root = app.world_mut().spawn_empty().id();
        app.world_mut().entity_mut(overlay).insert(LevelSelectRuntime {
            expansion: LevelSelectExpansionState::default(),
            visible_rows: vec![
                LevelSelectVisibleRow {
                    id: LevelSelectNodeId("a"),
                    label: "a",
                    depth: 0,
                    kind: LevelSelectVisibleRowKind::Folder,
                },
                LevelSelectVisibleRow {
                    id: LevelSelectNodeId("b"),
                    label: "b",
                    depth: 0,
                    kind: LevelSelectVisibleRowKind::Folder,
                },
                LevelSelectVisibleRow {
                    id: LevelSelectNodeId("c"),
                    label: "c",
                    depth: 0,
                    kind: LevelSelectVisibleRowKind::Folder,
                },
                LevelSelectVisibleRow {
                    id: LevelSelectNodeId("d"),
                    label: "d",
                    depth: 0,
                    kind: LevelSelectVisibleRowKind::Folder,
                },
            ],
            query_normalized: String::new(),
            window_entity,
            rows_root,
            dirty: false,
            scroll_sync_pending: false,
        });

        let scroll_root = app
            .world_mut()
            .spawn((
                ScrollableRoot::new(overlay, ScrollAxis::Vertical),
                ScrollState {
                    offset_px: 0.0,
                    content_extent: 500.0,
                    viewport_extent: 120.0,
                    max_offset: 380.0,
                },
                ScrollFocusFollowLock {
                    manual_override: true,
                },
            ))
            .id();

        let mut system = IntoSystem::into_system(sync_level_select_scroll_focus_follow);
        system.initialize(app.world_mut());
        system
            .run((), app.world_mut())
            .expect("focus follow should initialize previous selection");
        system.apply_deferred(app.world_mut());

        if let Some(mut runtime) = app
            .world_mut()
            .entity_mut(overlay)
            .get_mut::<LevelSelectRuntime>()
        {
            runtime.scroll_sync_pending = true;
        }

        system
            .run((), app.world_mut())
            .expect("focus follow should run with pending sync");
        system.apply_deferred(app.world_mut());

        let state = app
            .world()
            .entity(scroll_root)
            .get::<ScrollState>()
            .copied()
            .expect("scroll state");
        let lock = app
            .world()
            .entity(scroll_root)
            .get::<ScrollFocusFollowLock>()
            .copied()
            .expect("focus lock");

        assert!(state.offset_px > 0.0);
        assert!(!lock.manual_override);
    }
}
