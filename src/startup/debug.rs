use bevy::{
    prelude::*,
    sprite::Anchor,
    text::TextBounds,
    time::Real,
};

use crate::{
    data::states::MainState,
    entities::{
        sprites::window::WindowTitle,
        text::{TextPlugin, TextRaw, TextWindow},
    },
    startup::{cursor::CustomCursor, render::{MainCamera, OffscreenCamera}},
    systems::{interaction::Draggable, particles::FireworkLauncher},
};

const DEBUG_OVERLAY_Z: f32 = 900.0;
const DEBUG_LABEL_Z: f32 = 901.0;
const DEBUG_CONTENT_Z: f32 = 910.0;
const DEBUG_OVERLAY_SIZE: f32 = 100_000.0;
const DEBUG_LABEL: &str = "DEBUG MODE\n[F3 or Q to exit]";
const DEBUG_WINDOW_TITLE: &str = "Debug Lorem Ipsum";
const DEBUG_LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed, dolor. Cras elementum ultrices diam. Maecenas ligula massa, varius a, semper congue, euismod non, mi.";

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

fn update_debug_overlay_position(
    offscreen_camera_query: Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut overlay_query: Query<&mut Transform, With<DebugOverlay>>,
) {
    let camera_translation = if let Ok(camera) = offscreen_camera_query.single() {
        camera.translation()
    } else if let Ok(camera) = main_camera_query.single() {
        camera.translation()
    } else {
        return;
    };

    for mut overlay_transform in &mut overlay_query {
        overlay_transform.translation.x = camera_translation.x;
        overlay_transform.translation.y = camera_translation.y;
    }
}

fn handle_debug_shortcuts(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cursor: Res<CustomCursor>,
) {
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
