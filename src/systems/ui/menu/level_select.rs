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
const LEVEL_SELECT_LIST_START_Y: f32 = 146.0;
const LEVEL_SELECT_LIST_ROW_STEP: f32 = 22.0;
const LEVEL_SELECT_TREE_INDENT: f32 = 14.0;
const LEVEL_SELECT_ROW_REGION: Vec2 = Vec2::new(630.0, 20.0);

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
        for (index, row) in file_rows.iter().enumerate() {
            let LevelSelectVisibleRowKind::File(file) = row.kind else {
                continue;
            };
            let option_entity = system_menu::spawn_option(
                content,
                row.label,
                LEVEL_SELECT_LIST_X + row.depth as f32 * LEVEL_SELECT_TREE_INDENT,
                LEVEL_SELECT_LIST_START_Y - index as f32 * LEVEL_SELECT_LIST_ROW_STEP,
                overlay_entity,
                index,
                system_menu::SystemMenuOptionVisualStyle::default(),
            );

            content.commands().entity(option_entity).insert((
                Name::new(format!("level_select_file_{index}")),
                Clickable::with_region(vec![SystemMenuActions::Activate], LEVEL_SELECT_ROW_REGION),
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
}
