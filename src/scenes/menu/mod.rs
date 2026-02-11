use bevy::{audio::Volume, prelude::*};
use enum_map::{
    Enum, 
    enum_map
};

use crate::{
    data::states::MainState, entities::{
        large_fonts::{
            AsciiPlugin, 
            AsciiString, TextEmotion
        }, text::{
            TextButton, 
            TextRaw
        }, track::Track, train::{
            Train, 
            TrainPlugin, 
            content::TrainTypes
        } 
    }, style::{
        ui::IOPlugin
    }, systems::{
        audio::{
            BackgroundAudio, ContinuousAudio, ContinuousAudioPallet, DilatableAudio, MusicAudio, TransientAudio, TransientAudioPallet, continuous_audio
        }, 
        backgrounds::{
            Background, 
            BackgroundPlugin,
            content::BackgroundTypes
        }, 
        colors::{
            CLICKED_BUTTON,
            DIM_BACKGROUND_COLOR, 
            HOVERED_BUTTON,
            MENU_COLOR,
            ColorAnchor,
        },
        interaction::{ 
            ActionPallet, 
            Clickable,
            InputAction, 
            InteractionPlugin,
            InteractionVisualPalette,
            InteractionVisualState,
            Selectable,
            SelectableMenu,
        },
        time::Dilation,
    }
};

use super::SceneQueue;

const SYSTEM_MUSIC_PATH: &str = "./audio/music/the_last_decision.ogg";

pub struct MenuScenePlugin;
impl Plugin for MenuScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MainState::Menu), 
            MenuScene::setup
        );
        app.add_systems(
            Update,
            play_menu_navigation_sound
                .run_if(in_state(MainState::Menu))
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

impl MenuScene {

    const TITLE_TRANSLATION : Vec3 = Vec3::new(-380.0, 225.0, 1.0);
    const TRAIN_TRANSLATION: Vec3 = Vec3::new(110.0, -35.0, 0.5);
    const TRACK_DISPLACEMENT: Vec3 = Vec3::new(-120.0, -30.0, 0.4);
    const SIGNATURE_TRANSLATION : Vec3 = Vec3::new(0.0, -100.0, 1.0);
    const OPTIONS_LIST_TRANSLATION : Vec3 = Vec3::new(0.0, -230.0, 1.0);

    fn setup(
        mut commands: Commands, 
        mut queue : ResMut<SceneQueue>,
        asset_server: Res<AssetServer>
    ) { 
        *queue = SceneQueue::default();

        let scene_entity = commands
            .spawn(
            (
                queue.current,
                MenuScene,
                DespawnOnExit(MainState::Menu),
                children![
                    (
                        BackgroundAudio,
                        ContinuousAudioPallet::new(
                            vec![
                                ContinuousAudio{
                                    key : MenuSounds::Static,
                                    source : AudioPlayer::<AudioSource>(
                                        asset_server.load(
                                            "./audio/effects/static.ogg"
                                        )
                                    ), 
                                    settings : PlaybackSettings{
                                        volume : Volume::Linear(0.1),
                                        ..continuous_audio()
                                    },
                                    dilatable : true
                                },
                                ContinuousAudio{
                                    key : MenuSounds::Office,
                                    source : AudioPlayer::<AudioSource>(
                                        asset_server.load(
                                            "./audio/effects/office.ogg"
                                        )
                                    ), 
                                    settings : PlaybackSettings{
                                        volume : Volume::Linear(0.5),
                                        ..continuous_audio()
                                    },
                                    dilatable : true
                                }
                            ]
                        )
                    ),
                    (         
                        Background::new(
                            BackgroundTypes::Desert,	
                            0.00002,
                            -0.5
                        ),
                        TextColor(DIM_BACKGROUND_COLOR)
                    ),
                    (
                        Track::new(600),
                        TextColor(DIM_BACKGROUND_COLOR),
                        Transform::from_translation(Self::TRAIN_TRANSLATION + Self::TRACK_DISPLACEMENT)
                    ),
                    (
                        MusicAudio,
                        AudioPlayer::<AudioSource>(asset_server.load(
                            SYSTEM_MUSIC_PATH,
                        )),
                        PlaybackSettings{
                            volume : Volume::Linear(0.3),
                            ..continuous_audio()
                        }
                    ),
                    (
                        TextEmotion::Happy,
                        AsciiString(
                            "THE TROLLEY\n  ALGORITHM".to_string()
                        ),
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
                        Text2d::new(
                            "A game by Michael Norman"
                        ),
                        Transform::from_translation(Self::SIGNATURE_TRANSLATION)
                    )
                ]
            ),
        )
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
                SelectableMenu::new(
                    0,
                    vec![KeyCode::ArrowUp],
                    vec![KeyCode::ArrowDown],
                    vec![KeyCode::Enter],
                    true,
                ),
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
            parent.spawn((
                Name::new("menu_start_option"),
                TextButton,
                Text2d::new("Start Game"),
                TextLayout {
                    justify: Justify::Center,
                    ..default()
                },
                TextColor(MENU_COLOR),
                ColorAnchor(MENU_COLOR),
                Clickable::new(vec![MenuActions::EnterGame]),
                InteractionVisualState::default(),
                InteractionVisualPalette::new(
                    MENU_COLOR,
                    HOVERED_BUTTON,
                    CLICKED_BUTTON,
                    HOVERED_BUTTON,
                ),
                Transform::from_xyz(0.0, 46.0, 1.0),
                Selectable::new(menu_list_entity, 0),
                ActionPallet::<MenuActions, MenuSounds>(enum_map!(
                    MenuActions::EnterGame => vec![
                        InputAction::PlaySound(MenuSounds::Click),
                        InputAction::NextScene
                    ],
                    MenuActions::OpenOptions => vec![],
                    MenuActions::ExitDesktop => vec![],
                )),
                option_click_audio(),
            ));

            parent.spawn((
                Name::new("menu_options_option"),
                TextButton,
                Text2d::new("Options"),
                TextLayout {
                    justify: Justify::Center,
                    ..default()
                },
                TextColor(MENU_COLOR),
                ColorAnchor(MENU_COLOR),
                Clickable::new(vec![MenuActions::OpenOptions]),
                InteractionVisualState::default(),
                InteractionVisualPalette::new(
                    MENU_COLOR,
                    HOVERED_BUTTON,
                    CLICKED_BUTTON,
                    HOVERED_BUTTON,
                ),
                Transform::from_xyz(0.0, 0.0, 1.0),
                Selectable::new(menu_list_entity, 1),
                ActionPallet::<MenuActions, MenuSounds>(enum_map!(
                    MenuActions::EnterGame => vec![],
                    MenuActions::OpenOptions => vec![],
                    MenuActions::ExitDesktop => vec![],
                )),
                option_click_audio(),
            ));

            parent.spawn((
                Name::new("menu_exit_option"),
                TextButton,
                Text2d::new("Exit to Desktop"),
                TextLayout {
                    justify: Justify::Center,
                    ..default()
                },
                TextColor(MENU_COLOR),
                ColorAnchor(MENU_COLOR),
                Clickable::new(vec![MenuActions::ExitDesktop]),
                InteractionVisualState::default(),
                InteractionVisualPalette::new(
                    MENU_COLOR,
                    HOVERED_BUTTON,
                    CLICKED_BUTTON,
                    HOVERED_BUTTON,
                ),
                Transform::from_xyz(0.0, -46.0, 1.0),
                Selectable::new(menu_list_entity, 2),
                ActionPallet::<MenuActions, MenuSounds>(enum_map!(
                    MenuActions::EnterGame => vec![],
                    MenuActions::OpenOptions => vec![],
                    MenuActions::ExitDesktop => vec![
                        InputAction::PlaySound(MenuSounds::Click),
                        InputAction::ExitApplication,
                    ],
                )),
                option_click_audio(),
            ));
        });
    }
}

fn play_menu_navigation_sound(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    menu_query: Query<
        (Entity, &SelectableMenu, &TransientAudioPallet<MenuSounds>),
        With<MenuSelectableList>,
    >,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
) {
    for (menu_entity, menu, pallet) in &menu_query {
        let up_pressed = menu
            .up_keys
            .iter()
            .any(|&key| keyboard_input.just_pressed(key));
        let down_pressed = menu
            .down_keys
            .iter()
            .any(|&key| keyboard_input.just_pressed(key));

        if (up_pressed && !down_pressed) || (down_pressed && !up_pressed) {
            TransientAudioPallet::play_transient_audio(
                menu_entity,
                &mut commands,
                pallet,
                MenuSounds::Switch,
                dilation.0,
                &mut audio_query,
            );
        }
    }
}
