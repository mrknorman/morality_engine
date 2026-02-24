use std::collections::HashMap;

use bevy::{prelude::*, sprite::Anchor};

use super::{
    level_select_catalog::{
        self, LevelSelectExpansionState, LevelSelectNodeId, LevelSelectVisibleRow,
        LevelSelectVisibleRowKind,
    },
    *,
};
use crate::{
    entities::{
        sprites::compound::HollowRectangle,
        text::{scaled_font_size, TextRaw},
    },
    scenes::{dilemma::content::DilemmaScene, Scene, SceneFlowMode, SceneQueue},
    startup::system_menu,
    systems::{
        interaction::{Draggable, UiInputCaptureOwner, UiInputCaptureToken, UiInputPolicy},
        ui::{
            scroll::{
                focus_scroll_offset_to_row, row_visible_in_viewport, ScrollAxis,
                ScrollFocusFollowLock, ScrollState, ScrollableRoot,
            },
            search_box::{SearchBox, SearchBoxConfig, SearchBoxQueryChanged},
            text_input_box::TextInputBoxStyle,
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

const LEVEL_SELECT_SEARCH_BOX_Y: f32 = 205.0;
const LEVEL_SELECT_SEARCH_BOX_SIZE: Vec2 = Vec2::new(640.0, 24.0);
const LEVEL_SELECT_SEARCH_FONT_SIZE: f32 = scaled_font_size(11.0);

const LEVEL_SELECT_LIST_X: f32 = -320.0;
const LEVEL_SELECT_LIST_ROW_STEP: f32 = 22.0;
const LEVEL_SELECT_TREE_INDENT: f32 = 14.0;
const LEVEL_SELECT_ROW_REGION: Vec2 = Vec2::new(630.0, 20.0);
const LEVEL_SELECT_LIST_CONTENT_TOP_Y: f32 = 146.0;
const LEVEL_SELECT_WINDOW_SCROLL_LEADING_PADDING: f32 =
    LEVEL_SELECT_WINDOW_SIZE.y * 0.5 - LEVEL_SELECT_LIST_CONTENT_TOP_Y;

#[derive(Component)]
pub(super) struct LevelSelectOverlay;

#[derive(Component, Clone, Copy)]
pub(super) struct LevelSelectScrollRow {
    index: usize,
    folder_id: Option<LevelSelectNodeId>,
}

#[derive(Component)]
pub(super) struct LevelSelectSearchBox {
    owner: Entity,
}

#[derive(Component)]
pub(super) struct LevelSelectRuntime {
    expansion: LevelSelectExpansionState,
    visible_rows: Vec<LevelSelectVisibleRow>,
    query_normalized: String,
    window_entity: Entity,
    rows_root: Entity,
    dirty: bool,
}

#[derive(Resource, Default)]
pub(super) struct LevelUnlockState {
    reached_in_campaign: Vec<DilemmaScene>,
}

fn level_select_scene_unlocked(scene: DilemmaScene, unlock_state: &LevelUnlockState) -> bool {
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
    let content_top = LEVEL_SELECT_SEARCH_BOX_Y + LEVEL_SELECT_SEARCH_BOX_SIZE.y * 0.5;
    let content_bottom = level_select_last_row_bottom_y(row_count);
    let height = (content_top - content_bottom + 16.0).max(LEVEL_SELECT_WINDOW_SIZE.y);
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
                (
                    if unlocked {
                        row.label.to_string()
                    } else {
                        format!("{} [locked]", row.label)
                    },
                    None,
                    if unlocked {
                        MenuCommand::StartSingleLevel(file.scene)
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
            system_menu::SystemMenuOptionVisualStyle::default(),
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

    let initial_expansion = {
        let root = level_select_catalog::level_select_catalog_root();
        LevelSelectExpansionState::all_expanded(&root)
    };
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
            Name::new("level_select_search_box"),
            LevelSelectSearchBox {
                owner: overlay_entity,
            },
            SearchBox::new(overlay_entity, UiLayerKind::Base),
            SearchBoxConfig {
                placeholder: "Search files and folders...".to_string(),
                ..default()
            },
            TextInputBoxStyle {
                size: LEVEL_SELECT_SEARCH_BOX_SIZE,
                font_size: LEVEL_SELECT_SEARCH_FONT_SIZE,
                padding: Vec2::new(10.0, 4.0),
                ..default()
            },
            UiInputPolicy::CapturedOnly,
            Transform::from_xyz(0.0, LEVEL_SELECT_SEARCH_BOX_Y, 0.24),
        ));
        content.spawn((
            Name::new("level_select_hint"),
            TextRaw,
            Text2d::new("Select a .dilem file to run a single scenario."),
            TextFont {
                font_size: scaled_font_size(12.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Anchor::CENTER_LEFT,
            Transform::from_xyz(-320.0, 182.0, 0.2),
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

pub(super) fn handle_level_select_folder_activation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    overlay_query: Query<(Entity, &SelectableMenu), With<LevelSelectOverlay>>,
    row_query: Query<
        (
            Entity,
            &Selectable,
            &LevelSelectScrollRow,
            &Clickable<SystemMenuActions>,
        ),
        With<MenuOptionCommand>,
    >,
    mut runtime_query: Query<&mut LevelSelectRuntime, With<LevelSelectOverlay>>,
) {
    for (overlay_entity, menu) in overlay_query.iter() {
        let activate_requested = menu
            .activate_keys
            .iter()
            .any(|key| keyboard_input.just_pressed(*key));
        let mut folder_to_toggle = None;
        let mut click_rank = 0;

        for (row_entity, selectable, row, clickable) in row_query.iter() {
            if selectable.menu_entity != overlay_entity {
                continue;
            }
            let Some(folder_id) = row.folder_id else {
                continue;
            };

            if clickable.triggered {
                let rank = row_entity.to_bits();
                if rank >= click_rank {
                    click_rank = rank;
                    folder_to_toggle = Some(folder_id);
                }
                continue;
            }

            if activate_requested && selectable.index == menu.selected_index {
                folder_to_toggle = Some(folder_id);
            }
        }

        let Some(folder_id) = folder_to_toggle else {
            continue;
        };
        let Ok(mut runtime) = runtime_query.get_mut(overlay_entity) else {
            continue;
        };
        runtime.expansion.toggle(folder_id);
        runtime.dirty = true;
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
    }
}

pub(super) fn track_campaign_reached_dilemmas(
    scene_queue: Res<SceneQueue>,
    mut unlock_state: ResMut<LevelUnlockState>,
) {
    if scene_queue.flow_mode() != SceneFlowMode::Campaign {
        return;
    }
    let Scene::Dilemma(scene) = scene_queue.current_scene() else {
        return;
    };
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
    overlay_query: Query<(Entity, &SelectableMenu, &LevelSelectRuntime), With<LevelSelectOverlay>>,
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

    for (overlay_entity, menu, runtime) in overlay_query.iter() {
        let row_count = runtime.visible_rows.len();
        let selection_changed =
            previous_selection_by_owner.insert(overlay_entity, menu.selected_index)
                != Some(menu.selected_index);
        if menu.selected_index >= row_count {
            continue;
        }

        for (root, mut state, mut focus_lock) in root_query.iter_mut() {
            if root.owner != overlay_entity || root.axis != ScrollAxis::Vertical {
                continue;
            }
            if keyboard_navigation {
                focus_lock.manual_override = false;
            }
            if focus_lock.manual_override {
                break;
            }
            if !keyboard_navigation && !selection_changed {
                break;
            }

            focus_scroll_offset_to_row(
                &mut state,
                menu.selected_index,
                LEVEL_SELECT_LIST_ROW_STEP,
                LEVEL_SELECT_WINDOW_SCROLL_LEADING_PADDING,
            );
            break;
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

#[cfg(test)]
mod tests {
    use bevy::ecs::system::IntoSystem;

    use super::*;

    #[test]
    fn level_select_scroll_content_height_matches_row_geometry() {
        let row_count = 19;
        let preferred_inner = level_select_preferred_inner_size(row_count);

        assert!(preferred_inner.y > LEVEL_SELECT_WINDOW_SIZE.y);
        assert!(preferred_inner.x >= LEVEL_SELECT_WINDOW_SIZE.x);
    }

    #[test]
    fn level_select_scroll_systems_initialize_without_query_alias_panics() {
        let mut world = World::new();

        let mut search_query_system = IntoSystem::into_system(sync_level_select_search_query);
        search_query_system.initialize(&mut world);

        let mut folder_activation_system =
            IntoSystem::into_system(handle_level_select_folder_activation);
        folder_activation_system.initialize(&mut world);

        let mut rebuild_system = IntoSystem::into_system(rebuild_level_select_rows);
        rebuild_system.initialize(&mut world);

        let mut focus_follow_system = IntoSystem::into_system(sync_level_select_scroll_focus_follow);
        focus_follow_system.initialize(&mut world);

        let mut hit_region_system =
            IntoSystem::into_system(sync_level_select_option_hit_regions_to_viewport);
        hit_region_system.initialize(&mut world);
    }
}
