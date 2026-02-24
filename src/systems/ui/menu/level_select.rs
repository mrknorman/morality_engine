use std::collections::HashMap;

use bevy::{prelude::*, sprite::Anchor};

use super::*;
use crate::{
    entities::{
        sprites::compound::HollowRectangle,
        text::{scaled_font_size, TextRaw},
    },
    startup::system_menu,
    systems::{
        interaction::{Draggable, UiInputCaptureOwner, UiInputCaptureToken, UiInputPolicy},
        ui::scroll::{
            focus_scroll_offset_to_row, row_visible_in_viewport, ScrollAxis, ScrollBar,
            ScrollFocusFollowLock, ScrollState, ScrollableContent, ScrollableContentExtent,
            ScrollableItem, ScrollableRoot, ScrollableTableAdapter, ScrollableViewport,
        },
        ui::window::{
            UiWindow, UiWindowContent, UiWindowContentMetrics, UiWindowOverflowPolicy,
            UiWindowTitle,
        },
    },
};
use super::level_select_catalog::{self, LevelSelectVisibleRowKind};

const LEVEL_SELECT_OVERLAY_DIM_ALPHA: f32 = 0.8;
const LEVEL_SELECT_OVERLAY_DIM_SIZE: f32 = 6000.0;
const LEVEL_SELECT_OVERLAY_DIM_Z: f32 = -5.0;
const LEVEL_SELECT_WINDOW_SIZE: Vec2 = Vec2::new(690.0, 440.0);
const LEVEL_SELECT_WINDOW_Z: f32 = 0.4;
const LEVEL_SELECT_LIST_X: f32 = -320.0;
const LEVEL_SELECT_LIST_ROW_STEP: f32 = 22.0;
const LEVEL_SELECT_TREE_INDENT: f32 = 14.0;
const LEVEL_SELECT_ROW_REGION: Vec2 = Vec2::new(630.0, 20.0);
const LEVEL_SELECT_LIST_VIEWPORT_TOP_Y: f32 = 154.0;
const LEVEL_SELECT_LIST_VIEWPORT_BOTTOM_Y: f32 = -196.0;
const LEVEL_SELECT_LIST_VIEWPORT_HEIGHT: f32 =
    LEVEL_SELECT_LIST_VIEWPORT_TOP_Y - LEVEL_SELECT_LIST_VIEWPORT_BOTTOM_Y;
const LEVEL_SELECT_LIST_CENTER_Y: f32 =
    (LEVEL_SELECT_LIST_VIEWPORT_TOP_Y + LEVEL_SELECT_LIST_VIEWPORT_BOTTOM_Y) * 0.5;
const LEVEL_SELECT_LIST_CONTENT_TOP_Y: f32 = 146.0;
const LEVEL_SELECT_LIST_LEADING_PADDING: f32 =
    LEVEL_SELECT_LIST_VIEWPORT_TOP_Y - LEVEL_SELECT_LIST_CONTENT_TOP_Y;
const LEVEL_SELECT_LIST_VIEWPORT_SIZE: Vec2 = Vec2::new(640.0, LEVEL_SELECT_LIST_VIEWPORT_HEIGHT);

#[derive(Component)]
pub(super) struct LevelSelectScrollRoot;

#[derive(Component, Clone, Copy)]
pub(super) struct LevelSelectScrollRow {
    index: usize,
}

fn level_select_row_center_y(index: usize) -> f32 {
    LEVEL_SELECT_LIST_CONTENT_TOP_Y - (index as f32 + 0.5) * LEVEL_SELECT_LIST_ROW_STEP
}

fn level_select_scroll_local_y(world_y: f32) -> f32 {
    world_y - LEVEL_SELECT_LIST_CENTER_Y
}

fn level_select_scroll_content_height(row_count: usize) -> f32 {
    LEVEL_SELECT_LIST_LEADING_PADDING + row_count as f32 * LEVEL_SELECT_LIST_ROW_STEP
}

#[derive(Component)]
pub(super) struct LevelSelectOverlay;

pub(super) fn spawn_level_select_overlay(
    commands: &mut Commands,
    menu_root: &MenuRoot,
    asset_server: &Res<AssetServer>,
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
        Vec3::new(
            camera_translation.x,
            camera_translation.y,
            system_menu::MENU_Z,
        ),
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
            UiWindowContentMetrics::from_min_inner(LEVEL_SELECT_WINDOW_SIZE),
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

    commands.entity(content_root).with_children(|content| {
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

        let file_rows: Vec<_> = level_select_catalog::default_level_select_file_rows();

        let scroll_root = content
            .spawn((
                Name::new("level_select_list_scroll_root"),
                LevelSelectScrollRoot,
                UiInputPolicy::CapturedOnly,
                ScrollableRoot::new(overlay_entity, ScrollAxis::Vertical),
                ScrollableViewport::new(LEVEL_SELECT_LIST_VIEWPORT_SIZE),
                ScrollableContentExtent(level_select_scroll_content_height(file_rows.len())),
                ScrollableTableAdapter::new(
                    overlay_entity,
                    file_rows.len(),
                    LEVEL_SELECT_LIST_ROW_STEP,
                    LEVEL_SELECT_LIST_LEADING_PADDING,
                ),
                Transform::from_xyz(0.0, LEVEL_SELECT_LIST_CENTER_Y, 0.22),
            ))
            .id();
        let scroll_content = content
            .spawn((
                Name::new("level_select_list_scroll_content"),
                ScrollableContent,
                Transform::default(),
            ))
            .id();
        content.commands().entity(scroll_root).add_child(scroll_content);
        content
            .commands()
            .entity(scroll_root)
            .with_children(|scroll_root_parent| {
                scroll_root_parent.spawn((
                    Name::new("level_select_list_scrollbar"),
                    ScrollBar::new(scroll_root),
                    Transform::from_xyz(0.0, 0.0, 0.12),
                ));
            });

        content
            .commands()
            .entity(scroll_content)
            .with_children(|rows_parent| {
                for (index, row) in file_rows.iter().enumerate() {
                    let LevelSelectVisibleRowKind::File(file) = row.kind else {
                        continue;
                    };
                    let option_entity = system_menu::spawn_option(
                        rows_parent,
                        row.label,
                        LEVEL_SELECT_LIST_X + row.depth as f32 * LEVEL_SELECT_TREE_INDENT,
                        level_select_scroll_local_y(level_select_row_center_y(index)),
                        overlay_entity,
                        index,
                        system_menu::SystemMenuOptionVisualStyle::default(),
                    );

                    rows_parent.commands().entity(option_entity).insert((
                        Name::new(format!("level_select_file_{index}")),
                        LevelSelectScrollRow { index },
                        ScrollableItem::new(index as u64 + 1, index, LEVEL_SELECT_LIST_ROW_STEP),
                        Clickable::with_region(
                            vec![SystemMenuActions::Activate],
                            LEVEL_SELECT_ROW_REGION,
                        ),
                        MenuOptionCommand(MenuCommand::StartSingleLevel(file.scene)),
                        system_menu::click_audio_pallet(asset_server, SystemMenuSounds::Click),
                        UiInputPolicy::CapturedOnly,
                        Anchor::CENTER_LEFT,
                        TextLayout {
                            justify: Justify::Left,
                            ..default()
                        },
                    ));
                }
            });
    });
}

pub(super) fn sync_level_select_scroll_focus_follow(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    menu_query: Query<&SelectableMenu, With<LevelSelectOverlay>>,
    mut root_query: Query<
        (
            &ScrollableTableAdapter,
            &mut ScrollState,
            &mut ScrollFocusFollowLock,
        ),
        With<LevelSelectScrollRoot>,
    >,
    mut previous_selection_by_owner: Local<HashMap<Entity, usize>>,
) {
    let keyboard_navigation = keyboard_input.just_pressed(KeyCode::ArrowUp)
        || keyboard_input.just_pressed(KeyCode::ArrowDown)
        || keyboard_input.just_pressed(KeyCode::PageUp)
        || keyboard_input.just_pressed(KeyCode::PageDown)
        || keyboard_input.just_pressed(KeyCode::Home)
        || keyboard_input.just_pressed(KeyCode::End);

    for (adapter, mut state, mut focus_lock) in root_query.iter_mut() {
        let Ok(menu) = menu_query.get(adapter.owner) else {
            continue;
        };
        let selection_changed =
            previous_selection_by_owner.insert(adapter.owner, menu.selected_index)
                != Some(menu.selected_index);
        if menu.selected_index >= adapter.row_count {
            continue;
        }

        if keyboard_navigation {
            focus_lock.manual_override = false;
        }
        if focus_lock.manual_override {
            continue;
        }
        if !keyboard_navigation && !selection_changed {
            continue;
        }

        focus_scroll_offset_to_row(
            &mut state,
            menu.selected_index,
            adapter.row_extent,
            adapter.leading_padding,
        );
    }
}

pub(super) fn sync_level_select_option_hit_regions_to_viewport(
    root_query: Query<(&ScrollableTableAdapter, &ScrollState), With<LevelSelectScrollRoot>>,
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
    for (adapter, state) in root_query.iter() {
        adapter_state_by_owner.insert(adapter.owner, (*adapter, *state));
    }

    for (selectable, row, mut clickable) in option_query.iter_mut() {
        let Some((adapter, state)) =
            adapter_state_by_owner.get(&selectable.menu_entity).copied()
        else {
            clickable.region = Some(LEVEL_SELECT_ROW_REGION);
            continue;
        };

        let visible = row_visible_in_viewport(
            &state,
            row.index,
            adapter.row_extent,
            adapter.leading_padding,
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
        let content_height = level_select_scroll_content_height(row_count);
        let max_offset = (content_height - LEVEL_SELECT_LIST_VIEWPORT_HEIGHT).max(0.0);
        let last_row_bottom = LEVEL_SELECT_LIST_LEADING_PADDING
            + row_count as f32 * LEVEL_SELECT_LIST_ROW_STEP;

        assert!(content_height > LEVEL_SELECT_LIST_VIEWPORT_HEIGHT);
        assert!((last_row_bottom - (max_offset + LEVEL_SELECT_LIST_VIEWPORT_HEIGHT)).abs() < 0.001);
    }

    #[test]
    fn level_select_scroll_systems_initialize_without_query_alias_panics() {
        let mut world = World::new();

        let mut focus_follow_system =
            IntoSystem::into_system(sync_level_select_scroll_focus_follow);
        focus_follow_system.initialize(&mut world);

        let mut hit_region_system =
            IntoSystem::into_system(sync_level_select_option_hit_regions_to_viewport);
        hit_region_system.initialize(&mut world);
    }
}
