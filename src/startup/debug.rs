use bevy::{
    prelude::*,
    sprite::Anchor,
    text::TextBounds,
    time::Real,
};
use enum_map::enum_map;

use crate::{
    data::states::{MainState, StateVector},
    entities::{
        sprites::{compound::HollowRectangle, window::WindowTitle},
        text::{TextButton, TextPlugin, TextRaw, TextWindow},
    },
    startup::{cursor::CustomCursor, render::{MainCamera, OffscreenCamera}},
    systems::{
        audio::{TransientAudio, TransientAudioPallet},
        colors::{ColorAnchor, HOVERED_BUTTON, SYSTEM_MENU_COLOR},
        interaction::{
            ActionPallet, Clickable, Draggable, InputAction, OverlayMenuActions, OverlayMenuSounds,
            Selectable, SelectableColors, SelectableMenu,
        },
        particles::FireworkLauncher,
    },
};

const DEBUG_OVERLAY_Z: f32 = 900.0;
const DEBUG_LABEL_Z: f32 = 901.0;
const DEBUG_CONTENT_Z: f32 = 910.0;
const DEBUG_MENU_Z: f32 = 920.0;
const DEBUG_OVERLAY_SIZE: f32 = 100_000.0;
const DEBUG_LABEL: &str = "DEBUG MODE\n[F3 or Q to exit]\n[Tab for menu]";
const DEBUG_WINDOW_TITLE: &str = "Debug Lorem Ipsum";
const DEBUG_LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed, dolor. Cras elementum ultrices diam. Maecenas ligula massa, varius a, semper congue, euismod non, mi.";
const DEBUG_MENU_TITLE: &str = "DEBUG OVERLAY MENU";
const DEBUG_MENU_HINT: &str = "[ArrowUp/ArrowDown + Enter]\n[Click also works]";
const DEBUG_MENU_RESUME_TEXT: &str = "[Close Debug Menu]";
const DEBUG_MENU_MAIN_MENU_TEXT: &str = "[Return To Main Menu]";
const DEBUG_MENU_PANEL_WIDTH: f32 = 760.0;
const DEBUG_MENU_PANEL_HEIGHT: f32 = 460.0;
const DEBUG_MENU_BORDER_MARGIN: f32 = 18.0;
const DEBUG_MENU_BORDER_THICKNESS: f32 = 2.0;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, toggle_debug_mode)
            .add_systems(OnEnter(MainState::Debug), setup_debug_scene)
            .add_systems(
                Update,
                (
                    update_debug_overlay_position,
                    toggle_debug_menu_overlay,
                    handle_debug_shortcuts,
                    cleanup_debug_lifetimes,
                )
                    .run_if(in_state(MainState::Debug)),
            );

        if !app.is_plugin_added::<TextPlugin>() {
            app.add_plugins(TextPlugin);
        }
    }
}

#[derive(Component)]
struct DebugOverlay;

#[derive(Component)]
struct DebugMenuOverlay;

#[derive(Component)]
struct DebugLifetime(Timer);

fn toggle_debug_mode(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<MainState>>,
    mut next_state: ResMut<NextState<MainState>>,
) {
    if !keyboard_input.just_pressed(KeyCode::F3) {
        if *current_state.get() == MainState::Debug
            && keyboard_input.just_pressed(KeyCode::KeyQ)
        {
            next_state.set(MainState::Menu);
        }
        return;
    }

    match current_state.get() {
        MainState::Debug => next_state.set(MainState::Menu),
        _ => next_state.set(MainState::Debug),
    }
}

fn setup_debug_scene(
    mut commands: Commands,
    offscreen_camera_query: Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_query: Query<&GlobalTransform, With<MainCamera>>,
) {
    let mut translation = Vec3::new(0.0, 0.0, DEBUG_OVERLAY_Z);
    if let Ok(camera) = offscreen_camera_query.single() {
        translation.x = camera.translation().x;
        translation.y = camera.translation().y;
    } else if let Ok(camera) = main_camera_query.single() {
        translation.x = camera.translation().x;
        translation.y = camera.translation().y;
    }

    commands.spawn((
        Name::new("debug_overlay"),
        DebugOverlay,
        DespawnOnExit(MainState::Debug),
        Sprite::from_color(Color::BLACK, Vec2::splat(DEBUG_OVERLAY_SIZE)),
        Transform::from_xyz(translation.x, translation.y, DEBUG_OVERLAY_Z),
    ));

    commands.spawn((
        Name::new("debug_label"),
        DebugOverlay,
        DespawnOnExit(MainState::Debug),
        TextRaw,
        Text2d::new(DEBUG_LABEL),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Anchor::CENTER,
        Transform::from_xyz(translation.x, translation.y, DEBUG_LABEL_Z),
    ));
}

fn get_debug_camera_center(
    offscreen_camera_query: &Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_query: &Query<&GlobalTransform, With<MainCamera>>,
) -> Option<Vec3> {
    if let Ok(camera) = offscreen_camera_query.single() {
        Some(camera.translation())
    } else if let Ok(camera) = main_camera_query.single() {
        Some(camera.translation())
    } else {
        None
    }
}

fn update_debug_overlay_position(
    offscreen_camera_query: Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut overlay_query: Query<&mut Transform, With<DebugOverlay>>,
) {
    let Some(camera_translation) =
        get_debug_camera_center(&offscreen_camera_query, &main_camera_query)
    else {
        return;
    };

    for mut overlay_transform in &mut overlay_query {
        overlay_transform.translation.x = camera_translation.x;
        overlay_transform.translation.y = camera_translation.y;
    }
}

fn toggle_debug_menu_overlay(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    existing_menu: Query<Entity, With<DebugMenuOverlay>>,
    offscreen_camera_query: Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_query: Query<&GlobalTransform, With<MainCamera>>,
    asset_server: Res<AssetServer>,
) {
    if !keyboard_input.just_pressed(KeyCode::Tab) {
        return;
    }

    if let Some(menu_entity) = existing_menu.iter().next() {
        commands.entity(menu_entity).despawn_related::<Children>();
        commands.entity(menu_entity).despawn();
        return;
    }

    let Some(camera_translation) = get_debug_camera_center(&offscreen_camera_query, &main_camera_query)
    else {
        return;
    };

    spawn_debug_menu_overlay(&mut commands, &asset_server, camera_translation);
}

fn handle_debug_shortcuts(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cursor: Res<CustomCursor>,
    menu_query: Query<(), With<DebugMenuOverlay>>,
) {
    if !menu_query.is_empty() {
        return;
    }

    let ctrl_down = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight);
    if !ctrl_down {
        return;
    }

    let Some(cursor_position) = cursor.position else {
        return;
    };

    if keyboard_input.just_pressed(KeyCode::KeyN) {
        spawn_debug_text_window(&mut commands, cursor_position);
    }

    if keyboard_input.just_pressed(KeyCode::KeyF) {
        spawn_debug_firework(&mut commands, cursor_position);
    }
}

fn spawn_debug_text_window(commands: &mut Commands, cursor_position: Vec2) {
    commands.spawn((
        Name::new("debug_lorem_window"),
        DespawnOnExit(MainState::Debug),
        Draggable::default(),
        TextWindow {
            title: Some(WindowTitle {
                text: DEBUG_WINDOW_TITLE.to_string(),
                ..default()
            }),
            ..default()
        },
        TextRaw,
        Text2d::new(DEBUG_LOREM_IPSUM),
        TextColor(Color::WHITE),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextBounds {
            width: Some(440.0),
            height: None,
        },
        Transform::from_xyz(cursor_position.x, cursor_position.y, DEBUG_CONTENT_Z),
    ));
}

fn spawn_debug_menu_overlay(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    camera_translation: Vec3,
) {
    let click_audio = || {
        TransientAudioPallet::new(vec![(
            OverlayMenuSounds::Click,
            vec![TransientAudio::new(
                asset_server.load("./audio/effects/mech_click.ogg"),
                0.1,
                true,
                1.0,
                true,
            )],
        )])
    };

    let menu_entity = commands
        .spawn((
            Name::new("debug_menu_overlay"),
            DebugOverlay,
            DebugMenuOverlay,
            DespawnOnExit(MainState::Debug),
            SelectableMenu::new(
                0,
                vec![KeyCode::ArrowUp],
                vec![KeyCode::ArrowDown],
                vec![KeyCode::Enter],
                true,
            ),
            Transform::from_xyz(camera_translation.x, camera_translation.y, DEBUG_MENU_Z),
            Visibility::Visible,
        ))
        .id();

    commands.entity(menu_entity).with_children(|parent| {
        parent.spawn((
            Name::new("debug_menu_panel"),
            Sprite::from_color(
                Color::BLACK,
                Vec2::new(DEBUG_MENU_PANEL_WIDTH, DEBUG_MENU_PANEL_HEIGHT),
            ),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        parent.spawn((
            Name::new("debug_menu_panel_border"),
            HollowRectangle {
                dimensions: Vec2::new(
                    DEBUG_MENU_PANEL_WIDTH - 2.0 * DEBUG_MENU_BORDER_MARGIN,
                    DEBUG_MENU_PANEL_HEIGHT - 2.0 * DEBUG_MENU_BORDER_MARGIN,
                ),
                thickness: DEBUG_MENU_BORDER_THICKNESS,
                color: SYSTEM_MENU_COLOR,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.5),
        ));

        parent.spawn((
            Name::new("debug_menu_title"),
            TextRaw,
            Text2d::new(DEBUG_MENU_TITLE),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(SYSTEM_MENU_COLOR),
            Anchor::CENTER,
            Transform::from_xyz(0.0, 150.0, 1.0),
        ));

        parent.spawn((
            Name::new("debug_menu_hint"),
            TextRaw,
            Text2d::new(DEBUG_MENU_HINT),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(SYSTEM_MENU_COLOR),
            Anchor::CENTER,
            Transform::from_xyz(0.0, 110.0, 1.0),
        ));

        parent.spawn((
            Name::new("debug_menu_close_option"),
            TextButton,
            Text2d::new(DEBUG_MENU_RESUME_TEXT),
            TextLayout {
                justify: Justify::Center,
                ..default()
            },
            TextColor(SYSTEM_MENU_COLOR),
            ColorAnchor(SYSTEM_MENU_COLOR),
            Clickable::new(vec![OverlayMenuActions::CloseOverlay]),
            Transform::from_xyz(0.0, 25.0, 1.0),
            Selectable::new(menu_entity, 0),
            SelectableColors::new(SYSTEM_MENU_COLOR, HOVERED_BUTTON),
            ActionPallet::<OverlayMenuActions, OverlayMenuSounds>(enum_map!(
                OverlayMenuActions::CloseOverlay => vec![
                    InputAction::PlaySound(OverlayMenuSounds::Click),
                    InputAction::Despawn(Some(menu_entity)),
                ],
                OverlayMenuActions::ReturnToMenu => vec![],
            )),
            click_audio(),
        ));

        parent.spawn((
            Name::new("debug_menu_main_menu_option"),
            TextButton,
            Text2d::new(DEBUG_MENU_MAIN_MENU_TEXT),
            TextLayout {
                justify: Justify::Center,
                ..default()
            },
            TextColor(SYSTEM_MENU_COLOR),
            ColorAnchor(SYSTEM_MENU_COLOR),
            Clickable::new(vec![OverlayMenuActions::ReturnToMenu]),
            Transform::from_xyz(0.0, -25.0, 1.0),
            Selectable::new(menu_entity, 1),
            SelectableColors::new(SYSTEM_MENU_COLOR, HOVERED_BUTTON),
            ActionPallet::<OverlayMenuActions, OverlayMenuSounds>(enum_map!(
                OverlayMenuActions::CloseOverlay => vec![],
                OverlayMenuActions::ReturnToMenu => vec![
                    InputAction::PlaySound(OverlayMenuSounds::Click),
                    InputAction::ChangeState(StateVector::new(Some(MainState::Menu), None, None)),
                ],
            )),
            click_audio(),
        ));
    });
}

fn spawn_debug_firework(commands: &mut Commands, cursor_position: Vec2) {
    commands.spawn((
        Name::new("debug_firework_launcher"),
        DespawnOnExit(MainState::Debug),
        DebugLifetime(Timer::from_seconds(3.0, TimerMode::Once)),
        FireworkLauncher::one_shot(0.0),
        Transform::from_xyz(cursor_position.x, cursor_position.y, DEBUG_CONTENT_Z),
    ));
}

fn cleanup_debug_lifetimes(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut query: Query<(Entity, &mut DebugLifetime)>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.0.tick(time.delta());
        if lifetime.0.just_finished() {
            commands.entity(entity).despawn_related::<Children>();
            commands.entity(entity).despawn();
        }
    }
}
