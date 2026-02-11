use bevy::{audio::Volume, prelude::*, time::Virtual};
use enum_map::enum_map;

use crate::{
    data::states::{MainState, PauseState, StateVector},
    entities::text::TextPlugin,
    startup::{
        render::{MainCamera, OffscreenCamera},
        system_menu,
    },
    systems::{
        audio::{
            DilatableAudio, TransientAudio, TransientAudioPallet,
            continuous_audio,
        },
        interaction::{
            ActionPallet, Clickable, InputAction, InteractionGate, PauseMenuActions,
            PauseMenuSounds, SelectableMenu,
        },
        time::Dilation,
    },
};

const PAUSE_MENU_TITLE: &str = "PAUSED";
const PAUSE_MENU_HINT: &str = "[ESCAPE TO CONTINUE]\n[ARROW UP/ARROW DOWN + ENTER]";
const PAUSE_MENU_CONTINUE_TEXT: &str = "CONTINUE";
const PAUSE_MENU_OPTIONS_TEXT: &str = "OPTIONS";
const PAUSE_MENU_EXIT_TO_MENU_TEXT: &str = "EXIT TO MENU";
const PAUSE_MENU_EXIT_TO_DESKTOP_TEXT: &str = "EXIT TO DESKTOP";
const PAUSE_MENU_MUSIC_PATH: &str = "./audio/music/suspended_systems.ogg";
const PAUSE_MENU_DIM_ALPHA: f32 = 0.8;
const PAUSE_MENU_DIM_SIZE: f32 = 6000.0;
const PAUSE_MENU_DIM_Z: f32 = -5.0;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
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
                (
                    update_pause_overlay_position,
                    play_pause_menu_navigation_sound,
                    (
                        system_menu::ensure_selection_indicators,
                        system_menu::update_selection_indicators,
                    )
                        .chain(),
                )
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

#[derive(Component)]
struct PauseMenuAudio;

fn toggle_pause_from_escape(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pause_state: Res<State<PauseState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Escape) {
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
        (
            Without<PausedByPauseMenu>,
            Without<PauseMenuAudio>,
        ),
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

    let menu_entity = system_menu::spawn_root(
        &mut commands,
        &asset_server,
        "pause_menu_overlay",
        Vec3::new(
            camera_translation.x,
            camera_translation.y,
            system_menu::MENU_Z,
        ),
        PauseMenuSounds::Switch,
        (
            PauseMenuOverlay,
            InteractionGate::PauseMenuOnly,
            DespawnOnExit(PauseState::Paused),
        ),
    );

    commands.entity(menu_entity).with_children(|parent| {
        parent.spawn((
            Name::new("pause_menu_dimmer"),
            Sprite::from_color(
                Color::srgba(0.0, 0.0, 0.0, PAUSE_MENU_DIM_ALPHA),
                Vec2::splat(PAUSE_MENU_DIM_SIZE),
            ),
            Transform::from_xyz(0.0, 0.0, PAUSE_MENU_DIM_Z),
        ));
    });

    system_menu::spawn_chrome(
        &mut commands,
        menu_entity,
        "pause_menu",
        PAUSE_MENU_TITLE,
        PAUSE_MENU_HINT,
        system_menu::SystemMenuLayout::new(
            Vec2::new(system_menu::PANEL_WIDTH, 630.0),
            182.0,
            140.0,
        ),
    );

    // Keep pause-menu music separate from scene music.
    // `MusicAudio` is singleton-like and would replace the current scene track.
    commands.entity(menu_entity).with_children(|parent| {
        parent.spawn((
            Name::new("pause_menu_music"),
            PauseMenuAudio,
            AudioPlayer::<AudioSource>(asset_server.load(PAUSE_MENU_MUSIC_PATH)),
            PlaybackSettings {
                volume: Volume::Linear(0.25),
                ..continuous_audio()
            },
        ));
    });

    let click_audio = || system_menu::click_audio_pallet(&asset_server, PauseMenuSounds::Click);

    commands.entity(menu_entity).with_children(|parent| {
        parent.spawn((
            Name::new("pause_menu_continue_option"),
            InteractionGate::PauseMenuOnly,
            system_menu::SystemMenuOptionBundle::new(PAUSE_MENU_CONTINUE_TEXT, 50.0, menu_entity, 0),
            Clickable::new(vec![PauseMenuActions::Continue]),
            ActionPallet::<PauseMenuActions, PauseMenuSounds>(enum_map!(
                PauseMenuActions::Continue => vec![
                    InputAction::PlaySound(PauseMenuSounds::Click),
                    InputAction::ChangePauseState(PauseState::Unpaused),
                ],
                PauseMenuActions::OpenOptions => vec![],
                PauseMenuActions::ExitToMenu => vec![],
                PauseMenuActions::ExitToDesktop => vec![],
            )),
            click_audio(),
        ));

        parent.spawn((
            Name::new("pause_menu_options_option"),
            InteractionGate::PauseMenuOnly,
            system_menu::SystemMenuOptionBundle::new(PAUSE_MENU_OPTIONS_TEXT, 5.0, menu_entity, 1),
            Clickable::new(vec![PauseMenuActions::OpenOptions]),
            ActionPallet::<PauseMenuActions, PauseMenuSounds>(enum_map!(
                PauseMenuActions::Continue => vec![],
                PauseMenuActions::OpenOptions => vec![InputAction::PlaySound(PauseMenuSounds::Click)],
                PauseMenuActions::ExitToMenu => vec![],
                PauseMenuActions::ExitToDesktop => vec![],
            )),
            click_audio(),
        ));

        parent.spawn((
            Name::new("pause_menu_exit_to_menu_option"),
            InteractionGate::PauseMenuOnly,
            system_menu::SystemMenuOptionBundle::new(PAUSE_MENU_EXIT_TO_MENU_TEXT, -40.0, menu_entity, 2),
            Clickable::new(vec![PauseMenuActions::ExitToMenu]),
            ActionPallet::<PauseMenuActions, PauseMenuSounds>(enum_map!(
                PauseMenuActions::Continue => vec![],
                PauseMenuActions::OpenOptions => vec![],
                PauseMenuActions::ExitToMenu => vec![
                    InputAction::PlaySound(PauseMenuSounds::Click),
                    InputAction::ChangePauseState(PauseState::Unpaused),
                    InputAction::ChangeState(StateVector::new(Some(MainState::Menu), None, None)),
                ],
                PauseMenuActions::ExitToDesktop => vec![],
            )),
            click_audio(),
        ));

        parent.spawn((
            Name::new("pause_menu_exit_to_desktop_option"),
            InteractionGate::PauseMenuOnly,
            system_menu::SystemMenuOptionBundle::new(
                PAUSE_MENU_EXIT_TO_DESKTOP_TEXT,
                -85.0,
                menu_entity,
                3,
            ),
            Clickable::new(vec![PauseMenuActions::ExitToDesktop]),
            ActionPallet::<PauseMenuActions, PauseMenuSounds>(enum_map!(
                PauseMenuActions::Continue => vec![],
                PauseMenuActions::OpenOptions => vec![],
                PauseMenuActions::ExitToMenu => vec![],
                PauseMenuActions::ExitToDesktop => vec![
                    InputAction::PlaySound(PauseMenuSounds::Click),
                    InputAction::ExitApplication,
                ],
            )),
            click_audio(),
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

fn play_pause_menu_navigation_sound(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    menu_query: Query<
        (Entity, &SelectableMenu, &TransientAudioPallet<PauseMenuSounds>),
        With<PauseMenuOverlay>,
    >,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
) {
    system_menu::play_navigation_sound(
        &mut commands,
        keyboard_input.as_ref(),
        &menu_query,
        &mut audio_query,
        PauseMenuSounds::Switch,
        dilation.0,
    );
}
