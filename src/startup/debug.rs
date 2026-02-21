use bevy::{audio::Volume, prelude::*, sprite::Anchor, text::TextBounds, time::Real};

use crate::{
    data::states::MainState,
    entities::{
        text::{scaled_font_size, TextPlugin, TextRaw, TextWindow},
    },
    startup::{
        cursor::CustomCursor,
        render::{MainCamera, OffscreenCamera},
        system_menu,
    },
    systems::{
        audio::{continuous_audio, MusicAudio},
        interaction::{Draggable, InteractionGate},
        particles::FireworkLauncher,
        ui::{
            menu::{self, MenuHost, MenuPage},
            window::UiWindowTitle,
        },
    },
};

const DEBUG_OVERLAY_Z: f32 = 900.0;
const DEBUG_LABEL_Z: f32 = 901.0;
const DEBUG_CONTENT_Z: f32 = 910.0;
const DEBUG_OVERLAY_SIZE: f32 = 100_000.0;
const DEBUG_LABEL: &str = "DEBUG MODE\n[F3 or Q to exit]\n[Tab for menu]";
const DEBUG_WINDOW_TITLE: &str = "Debug Lorem Ipsum";
const DEBUG_LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed, dolor. Cras elementum ultrices diam. Maecenas ligula massa, varius a, semper congue, euismod non, mi.";
const SYSTEM_MUSIC_PATH: &str = "./audio/music/suspended_systems.ogg";

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_debug_mode)
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
        if *current_state.get() == MainState::Debug && keyboard_input.just_pressed(KeyCode::KeyQ) {
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
    asset_server: Res<AssetServer>,
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
            font_size: scaled_font_size(36.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Anchor::CENTER,
        Transform::from_xyz(translation.x, translation.y, DEBUG_LABEL_Z),
    ));

    commands.spawn((
        Name::new("debug_system_music"),
        DespawnOnExit(MainState::Debug),
        MusicAudio,
        AudioPlayer::<AudioSource>(asset_server.load(SYSTEM_MUSIC_PATH)),
        PlaybackSettings {
            volume: Volume::Linear(0.3),
            ..continuous_audio()
        },
    ));

    spawn_debug_menu_overlay(&mut commands, &asset_server, translation);
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

    let Some(camera_translation) =
        get_debug_camera_center(&offscreen_camera_query, &main_camera_query)
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
            title: Some(UiWindowTitle {
                text: DEBUG_WINDOW_TITLE.to_string(),
                ..default()
            }),
            ..default()
        },
        TextRaw,
        Text2d::new(DEBUG_LOREM_IPSUM),
        TextColor(Color::WHITE),
        TextFont {
            font_size: scaled_font_size(14.0),
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
    let menu_entity = menu::spawn_menu_root(
        commands,
        asset_server,
        MenuHost::Debug,
        "debug_menu_overlay",
        Vec3::new(
            camera_translation.x,
            camera_translation.y,
            system_menu::MENU_Z,
        ),
        MenuPage::DebugRoot,
        InteractionGate::GameplayOnly,
    );
    commands.entity(menu_entity).insert((
        DebugOverlay,
        DebugMenuOverlay,
        DespawnOnExit(MainState::Debug),
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
