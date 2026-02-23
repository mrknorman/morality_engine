use bevy::{prelude::*, time::Virtual};

use crate::{
    data::states::{MainState, PauseState},
    entities::text::TextPlugin,
    startup::{
        render::{MainCamera, OffscreenCamera},
        system_menu,
    },
    systems::{
        interaction::{UiInputCaptureOwner, UiInputCaptureToken, UiInputPolicy},
        ui::menu::{self, MenuHost, MenuPage, PauseMenuAudio},
    },
};

const PAUSE_MENU_DIM_ALPHA: f32 = 0.8;
const PAUSE_MENU_DIM_SIZE: f32 = 6000.0;
const PAUSE_MENU_DIM_Z: f32 = -5.0;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            toggle_pause_from_escape.run_if(in_state(MainState::InGame)),
        )
        .add_systems(
            OnEnter(PauseState::Paused),
            (
                pause_virtual_time,
                pause_music_and_narration_audio,
                setup_pause_menu_overlay,
            ),
        )
        .add_systems(
            OnExit(PauseState::Paused),
            (resume_virtual_time, resume_music_and_narration_audio),
        )
        .add_systems(OnEnter(MainState::InGame), reset_pause_state_on_enter_game)
        .add_systems(
            OnExit(MainState::InGame),
            (resume_virtual_time, resume_music_and_narration_audio),
        )
        .add_systems(
            Update,
            update_pause_overlay_position
                .run_if(in_state(MainState::InGame))
                .run_if(in_state(PauseState::Paused)),
        );

        if !app.is_plugin_added::<TextPlugin>() {
            app.add_plugins(TextPlugin);
        }
    }
}

#[derive(Component)]
struct PauseMenuOverlay;

#[derive(Component)]
struct PausedByPauseMenu;

fn toggle_pause_from_escape(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pause_state: Res<State<PauseState>>,
    pause_overlay_query: Query<(), With<PauseMenuOverlay>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Escape) {
        return;
    }

    if *pause_state.get() == PauseState::Paused && !pause_overlay_query.is_empty() {
        return;
    }

    match pause_state.get() {
        PauseState::Unpaused => next_pause_state.set(PauseState::Paused),
        PauseState::Paused => next_pause_state.set(PauseState::Unpaused),
    }
}

fn reset_pause_state_on_enter_game(
    mut next_pause_state: ResMut<NextState<PauseState>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    next_pause_state.set(PauseState::Unpaused);
    virtual_time.unpause();
}

fn pause_virtual_time(mut virtual_time: ResMut<Time<Virtual>>) {
    virtual_time.pause();
}

fn resume_virtual_time(mut virtual_time: ResMut<Time<Virtual>>) {
    virtual_time.unpause();
}

fn pause_music_and_narration_audio(
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut AudioSink),
        (Without<PausedByPauseMenu>, Without<PauseMenuAudio>),
    >,
) {
    for (entity, sink) in &mut query {
        if !sink.is_paused() {
            sink.pause();
            commands.entity(entity).insert(PausedByPauseMenu);
        }
    }
}

fn resume_music_and_narration_audio(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AudioSink), With<PausedByPauseMenu>>,
) {
    for (entity, sink) in &mut query {
        sink.play();
        commands.entity(entity).remove::<PausedByPauseMenu>();
    }
}

fn get_camera_center(
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

fn setup_pause_menu_overlay(
    mut commands: Commands,
    offscreen_camera_query: Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_query: Query<&GlobalTransform, With<MainCamera>>,
    asset_server: Res<AssetServer>,
) {
    let Some(camera_translation) = get_camera_center(&offscreen_camera_query, &main_camera_query)
    else {
        return;
    };

    let menu_entity = menu::spawn_menu_root(
        &mut commands,
        &asset_server,
        MenuHost::Pause,
        "pause_menu_overlay",
        Vec3::new(
            camera_translation.x,
            camera_translation.y,
            system_menu::MENU_Z,
        ),
        MenuPage::PauseRoot,
        UiInputPolicy::CapturedOnly,
    );
    commands
        .entity(menu_entity)
        .insert((PauseMenuOverlay, DespawnOnExit(PauseState::Paused)));

    commands.entity(menu_entity).with_children(|parent| {
        parent.spawn((
            Name::new("pause_menu_dimmer"),
            UiInputCaptureToken,
            UiInputCaptureOwner::new(menu_entity),
            Sprite::from_color(
                Color::srgba(0.0, 0.0, 0.0, PAUSE_MENU_DIM_ALPHA),
                Vec2::splat(PAUSE_MENU_DIM_SIZE),
            ),
            Transform::from_xyz(0.0, 0.0, PAUSE_MENU_DIM_Z),
        ));
    });
}

fn update_pause_overlay_position(
    offscreen_camera_query: Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut overlay_query: Query<&mut Transform, With<PauseMenuOverlay>>,
) {
    let Some(camera_translation) = get_camera_center(&offscreen_camera_query, &main_camera_query)
    else {
        return;
    };

    for mut overlay_transform in &mut overlay_query {
        overlay_transform.translation.x = camera_translation.x;
        overlay_transform.translation.y = camera_translation.y;
    }
}
