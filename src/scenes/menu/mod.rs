use bevy::{audio::Volume, prelude::*};
use enum_map::Enum;
use once_cell::sync::Lazy;

use crate::{
    data::states::MainState,
    entities::{
        large_fonts::{AsciiString, TextEmotion},
        text::TextRaw,
        track::Track,
        train::{content::TrainTypes, Train},
    },
    systems::{
        audio::{
            continuous_audio, BackgroundAudio, ContinuousAudio, ContinuousAudioPallet, MusicAudio,
        },
        backgrounds::{content::BackgroundTypes, Background},
        colors::{CLICKED_BUTTON, DIM_BACKGROUND_COLOR, HOVERED_BUTTON, MENU_COLOR},
        interaction::InteractionVisualPalette,
        ui::menu::{
            main_menu_command_registry, schema, spawn_main_menu_option_list, MainMenuEntry,
        },
    },
};

use super::SceneQueue;

const SYSTEM_MUSIC_PATH: &str = "./audio/music/the_last_decision.ogg";

const MAIN_MENU_SCHEMA_JSON: &str = include_str!("./content/main_menu_ui.json");

static MAIN_MENU_OPTIONS: Lazy<Vec<MainMenuEntry>> = Lazy::new(|| {
    let resolved = schema::load_and_resolve_menu_schema_with_registry(
        MAIN_MENU_SCHEMA_JSON,
        main_menu_command_registry(),
    )
    .unwrap_or_else(|error| panic!("invalid main menu schema: {error}"));

    resolved
        .options
        .into_iter()
        .map(|option| MainMenuEntry {
            name: option.id,
            label: option.label,
            y: option.y,
            command: option.command,
        })
        .collect()
});

pub struct MenuScenePlugin;
impl Plugin for MenuScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MainState::Menu), MenuScene::setup);
    }
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuSounds {
    Static,
    Office,
}

#[derive(Component)]
#[require(Transform, Visibility)]
struct MenuScene;

impl MenuScene {
    const TITLE_TRANSLATION: Vec3 = Vec3::new(0.0, 225.0, 1.0);
    const TRAIN_TRANSLATION: Vec3 = Vec3::new(110.0, -35.0, 0.5);
    const TRACK_DISPLACEMENT: Vec3 = Vec3::new(-120.0, Train::track_alignment_offset_y(), 0.4);
    const SIGNATURE_TRANSLATION: Vec3 = Vec3::new(0.0, -100.0, 1.0);
    const OPTIONS_LIST_TRANSLATION: Vec3 = Vec3::new(0.0, -230.0, 1.0);

    fn setup(
        mut commands: Commands,
        mut queue: ResMut<SceneQueue>,
        asset_server: Res<AssetServer>,
        existing_scene_query: Query<Entity, With<MenuScene>>,
    ) {
        for existing_scene in &existing_scene_query {
            commands
                .entity(existing_scene)
                .despawn_related::<Children>();
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

        let menu_list_entity = spawn_main_menu_option_list(
            &mut commands,
            &asset_server,
            Self::OPTIONS_LIST_TRANSLATION,
            &MAIN_MENU_OPTIONS,
            InteractionVisualPalette::new(
                MENU_COLOR,
                HOVERED_BUTTON,
                CLICKED_BUTTON,
                HOVERED_BUTTON,
            ),
        );
        commands.entity(scene_entity).add_child(menu_list_entity);
    }
}
