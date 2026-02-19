use bevy::{audio::Volume, prelude::*};
use enum_map::{enum_map, Enum};
use once_cell::sync::Lazy;

use crate::{
    data::states::{MainState, PauseState},
    entities::{
        large_fonts::{AsciiPlugin, AsciiString, TextEmotion},
        text::TextRaw,
        track::Track,
        train::{content::TrainTypes, Train, TrainPlugin},
    },
    startup::{
        render::{MainCamera, OffscreenCamera},
        system_menu,
    },
    style::ui::IOPlugin,
    systems::{
        audio::{
            continuous_audio, BackgroundAudio, ContinuousAudio, ContinuousAudioPallet,
            DilatableAudio, MusicAudio, TransientAudio, TransientAudioPallet,
        },
        backgrounds::{content::BackgroundTypes, Background, BackgroundPlugin},
        colors::{ColorAnchor, CLICKED_BUTTON, DIM_BACKGROUND_COLOR, HOVERED_BUTTON, MENU_COLOR},
        interaction::{
            ActionPallet, Clickable, InputAction,
            InteractionCapture, InteractionCaptureOwner, InteractionGate, InteractionPlugin,
            InteractionSystem,
            InteractionVisualPalette, SelectableClickActivation,
            SelectableMenu,
        },
        time::Dilation,
        ui::menu::{self, schema, MenuHost, MenuPage},
    },
};

use super::SceneQueue;

const SYSTEM_MUSIC_PATH: &str = "./audio/music/the_last_decision.ogg";
const SYSTEM_MENU_DIM_ALPHA: f32 = 0.8;
const SYSTEM_MENU_DIM_SIZE: f32 = 6000.0;
const SYSTEM_MENU_DIM_Z: f32 = -5.0;

struct MainMenuOptionSpec {
    name: String,
    label: String,
    y: f32,
    action: MenuActions,
}

const MAIN_MENU_SCHEMA_JSON: &str = include_str!("./content/main_menu_ui.json");

static MAIN_MENU_OPTIONS: Lazy<Vec<MainMenuOptionSpec>> = Lazy::new(|| {
    let resolved =
        schema::load_and_resolve_menu_schema(MAIN_MENU_SCHEMA_JSON, resolve_main_menu_action)
            .unwrap_or_else(|error| panic!("invalid main menu schema: {error}"));

    resolved
        .options
        .into_iter()
        .map(|option| MainMenuOptionSpec {
            name: option.id,
            label: option.label,
            y: option.y,
            action: option.command,
        })
        .collect()
});

pub struct MenuScenePlugin;
impl Plugin for MenuScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MainState::Menu), MenuScene::setup);
        app.add_systems(
            Update,
            (
                update_main_menu_options_overlay_position,
                play_menu_navigation_sound,
            )
                .chain()
                .run_if(in_state(MainState::Menu)),
        );
        app.add_systems(
            Update,
            open_main_menu_options_overlay
                .after(InteractionSystem::Audio)
                .run_if(in_state(MainState::Menu)),
        );
        if !app.is_plugin_added::<TrainPlugin>() {
            app.add_plugins(TrainPlugin);
        }
        if !app.is_plugin_added::<InteractionPlugin>() {
            app.add_plugins(InteractionPlugin);
        }
        if !app.is_plugin_added::<IOPlugin>() {
            app.add_plugins(IOPlugin);
        }
        if !app.is_plugin_added::<AsciiPlugin>() {
            app.add_plugins(AsciiPlugin);
        }
        if !app.is_plugin_added::<BackgroundPlugin>() {
            app.add_plugins(BackgroundPlugin);
        }
    }
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuSounds {
    Static,
    Office,
    Click,
    Switch,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuActions {
    EnterGame,
    OpenOptions,
    ExitDesktop,
}

fn resolve_main_menu_action(command_id: &str) -> Result<MenuActions, String> {
    match command_id {
        "enter_game" => Ok(MenuActions::EnterGame),
        "open_options" => Ok(MenuActions::OpenOptions),
        "exit_desktop" => Ok(MenuActions::ExitDesktop),
        _ => Err(format!("unknown main menu command `{command_id}`")),
    }
}

impl std::fmt::Display for MenuActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
struct MenuScene;

#[derive(Component)]
struct MenuSelectableList;

#[derive(Component)]
struct MainMenuInteractive;

#[derive(Component)]
struct MenuOpenOptionsButton;

#[derive(Component)]
struct MainMenuOptionsOverlay;

impl MenuScene {
    const TITLE_TRANSLATION: Vec3 = Vec3::new(0.0, 225.0, 1.0);
    const TRAIN_TRANSLATION: Vec3 = Vec3::new(110.0, -35.0, 0.5);
    const TRACK_DISPLACEMENT: Vec3 =
        Vec3::new(-120.0, Train::track_alignment_offset_y(), 0.4);
    const SIGNATURE_TRANSLATION: Vec3 = Vec3::new(0.0, -100.0, 1.0);
    const OPTIONS_LIST_TRANSLATION: Vec3 = Vec3::new(0.0, -230.0, 1.0);

    fn actions_for_option(action: MenuActions) -> ActionPallet<MenuActions, MenuSounds> {
        match action {
            MenuActions::EnterGame => ActionPallet(enum_map!(
                MenuActions::EnterGame => vec![
                    InputAction::PlaySound(MenuSounds::Click),
                    InputAction::NextScene
                ],
                MenuActions::OpenOptions => vec![],
                MenuActions::ExitDesktop => vec![],
            )),
            MenuActions::OpenOptions => ActionPallet(enum_map!(
                MenuActions::EnterGame => vec![],
                MenuActions::OpenOptions => vec![InputAction::PlaySound(MenuSounds::Click)],
                MenuActions::ExitDesktop => vec![],
            )),
            MenuActions::ExitDesktop => ActionPallet(enum_map!(
                MenuActions::EnterGame => vec![],
                MenuActions::OpenOptions => vec![],
                MenuActions::ExitDesktop => vec![
                    InputAction::PlaySound(MenuSounds::Click),
                    InputAction::ExitApplication,
                ],
            )),
        }
    }

    fn setup(
        mut commands: Commands,
        mut queue: ResMut<SceneQueue>,
        asset_server: Res<AssetServer>,
        existing_scene_query: Query<Entity, With<MenuScene>>,
    ) {
        for existing_scene in &existing_scene_query {
            commands.entity(existing_scene).despawn_related::<Children>();
            commands.entity(existing_scene).despawn();
        }

        *queue = SceneQueue::default();

        let scene_entity = commands
            .spawn((
                queue.current,
                MenuScene,
                DespawnOnExit(MainState::Menu),
                children![
                    (
                        BackgroundAudio,
                        ContinuousAudioPallet::new(vec![
                            ContinuousAudio {
                                key: MenuSounds::Static,
                                source: AudioPlayer::<AudioSource>(
                                    asset_server.load("./audio/effects/static.ogg")
                                ),
                                settings: PlaybackSettings {
                                    volume: Volume::Linear(0.1),
                                    ..continuous_audio()
                                },
                                dilatable: true
                            },
                            ContinuousAudio {
                                key: MenuSounds::Office,
                                source: AudioPlayer::<AudioSource>(
                                    asset_server.load("./audio/effects/office.ogg")
                                ),
                                settings: PlaybackSettings {
                                    volume: Volume::Linear(0.5),
                                    ..continuous_audio()
                                },
                                dilatable: true
                            }
                        ])
                    ),
                    (
                        Background::new(BackgroundTypes::Desert, 0.00002, -0.5),
                        TextColor(DIM_BACKGROUND_COLOR)
                    ),
                    (
                        Track::new(600),
                        TextColor(DIM_BACKGROUND_COLOR),
                        Transform::from_translation(
                            Self::TRAIN_TRANSLATION + Self::TRACK_DISPLACEMENT
                        )
                    ),
                    (
                        MusicAudio,
                        AudioPlayer::<AudioSource>(asset_server.load(SYSTEM_MUSIC_PATH,)),
                        PlaybackSettings {
                            volume: Volume::Linear(0.3),
                            ..continuous_audio()
                        }
                    ),
                    (
                        TextEmotion::Happy,
                        AsciiString("THE TROLLEY\n  ALGORITHM".to_string()),
                        TextColor(MENU_COLOR),
                        Transform::from_translation(Self::TITLE_TRANSLATION)
                    ),
                    (
                        Train(TrainTypes::SteamTrain),
                        Transform::from_translation(Self::TRAIN_TRANSLATION),
                        TextColor(MENU_COLOR),
                    ),
                    (
                        TextRaw,
                        Text2d::new("A game by Michael Norman"),
                        Transform::from_translation(Self::SIGNATURE_TRANSLATION)
                    )
                ],
            ))
            .id();

        let option_click_audio = || {
            TransientAudioPallet::new(vec![(
                MenuSounds::Click,
                vec![TransientAudio::new(
                    asset_server.load("./audio/effects/mech_click.ogg"),
                    0.1,
                    true,
                    1.0,
                    true,
                )],
            )])
        };

        let menu_list_entity = commands
            .spawn((
                Name::new("menu_selectable_list"),
                MenuSelectableList,
                MainMenuInteractive,
                SelectableMenu::new(
                    0,
                    vec![KeyCode::ArrowUp],
                    vec![KeyCode::ArrowDown],
                    vec![KeyCode::Enter, KeyCode::ArrowRight],
                    true,
                )
                .with_click_activation(SelectableClickActivation::HoveredOnly),
                TransientAudioPallet::new(vec![(
                    MenuSounds::Switch,
                    vec![TransientAudio::new(
                        asset_server.load("./audio/effects/switch.ogg"),
                        0.03,
                        true,
                        0.2,
                        false,
                    )],
                )]),
                Transform::from_translation(Self::OPTIONS_LIST_TRANSLATION),
                Visibility::Visible,
            ))
            .id();

        commands.entity(scene_entity).add_child(menu_list_entity);

        commands.entity(menu_list_entity).with_children(|parent| {
            for (index, option) in MAIN_MENU_OPTIONS.iter().enumerate() {
                let mut entity_commands = parent.spawn((
                    Name::new(option.name.clone()),
                    MainMenuInteractive,
                    system_menu::SystemMenuOptionBundle::new_at(
                        option.label.clone(),
                        0.0,
                        option.y,
                        menu_list_entity,
                        index,
                    )
                    .with_visual_style(
                        system_menu::SystemMenuOptionVisualStyle::default()
                            .without_selection_indicators(),
                    ),
                    Clickable::new(vec![option.action]),
                    MenuScene::actions_for_option(option.action),
                    option_click_audio(),
                ));
                if option.action == MenuActions::OpenOptions {
                    entity_commands.insert(MenuOpenOptionsButton);
                }
                entity_commands.insert((
                    TextColor(MENU_COLOR),
                    ColorAnchor(MENU_COLOR),
                    InteractionVisualPalette::new(
                        MENU_COLOR,
                        HOVERED_BUTTON,
                        CLICKED_BUTTON,
                        HOVERED_BUTTON,
                    ),
                ));
            }
        });
    }
}

fn get_menu_camera_center(
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

fn open_main_menu_options_overlay(
    mut commands: Commands,
    mut options_query: Query<&mut Clickable<MenuActions>, With<MenuOpenOptionsButton>>,
    existing_overlay: Query<(), With<MainMenuOptionsOverlay>>,
    offscreen_camera_query: Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_query: Query<&GlobalTransform, With<MainCamera>>,
    asset_server: Res<AssetServer>,
) {
    if !existing_overlay.is_empty() {
        return;
    }

    let mut should_open = false;
    for mut clickable in &mut options_query {
        if clickable.triggered {
            clickable.triggered = false;
            should_open = true;
            break;
        }
    }

    if !should_open {
        return;
    }

    let Some(camera_translation) =
        get_menu_camera_center(&offscreen_camera_query, &main_camera_query)
    else {
        return;
    };

    let menu_entity = menu::spawn_menu_root(
        &mut commands,
        &asset_server,
        MenuHost::Main,
        "main_menu_options_overlay",
        Vec3::new(
            camera_translation.x,
            camera_translation.y,
            system_menu::MENU_Z,
        ),
        MenuPage::Options,
        InteractionGate::PauseMenuOnly,
        (MainMenuOptionsOverlay, DespawnOnExit(MainState::Menu)),
    );

    commands.entity(menu_entity).with_children(|parent| {
        parent.spawn((
            Name::new("main_menu_options_dimmer"),
            InteractionCapture,
            InteractionCaptureOwner::new(menu_entity),
            Sprite::from_color(
                Color::srgba(0.0, 0.0, 0.0, SYSTEM_MENU_DIM_ALPHA),
                Vec2::splat(SYSTEM_MENU_DIM_SIZE),
            ),
            Transform::from_xyz(0.0, 0.0, SYSTEM_MENU_DIM_Z),
        ));
    });
}

fn update_main_menu_options_overlay_position(
    offscreen_camera_query: Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut overlay_query: Query<&mut Transform, With<MainMenuOptionsOverlay>>,
) {
    let Some(camera_translation) =
        get_menu_camera_center(&offscreen_camera_query, &main_camera_query)
    else {
        return;
    };

    for mut overlay_transform in &mut overlay_query {
        overlay_transform.translation.x = camera_translation.x;
        overlay_transform.translation.y = camera_translation.y;
    }
}

fn play_menu_navigation_sound(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    menu_query: Query<
        (
            Entity,
            &SelectableMenu,
            &TransientAudioPallet<MenuSounds>,
            Option<&InteractionGate>,
        ),
        With<MenuSelectableList>,
    >,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
) {
    system_menu::play_navigation_sound_owner_scoped(
        &mut commands,
        &keyboard_input,
        pause_state.as_ref(),
        &capture_query,
        &menu_query,
        &mut audio_query,
        MenuSounds::Switch,
        dilation.0,
    );
}
