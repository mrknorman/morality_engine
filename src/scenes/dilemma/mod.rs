use crate::{
    data::{
        states::{DilemmaPhase, GameState, MainState},
        stats::{DilemmaRunStatsScope, DilemmaStats, GameStats},
    },
    entities::{
        large_fonts::{AsciiPlugin, AsciiString, TextEmotion},
        person::{BloodSprite, PersonPlugin},
        sprites::SpritePlugin,
        text::{scaled_font_size, TextPlugin, TextWindow},
        track::Track,
        train::{content::TrainTypes, Train, TrainPlugin},
    },
    scenes::dilemma::{
        dilemma::{CurrentDilemmaStageIndex, DilemmaStage},
        phases::transition::DilemmaTransitionPlugin,
    },
    style::ui::IOPlugin,
    systems::{
        audio::{continuous_audio, MusicAudio},
        backgrounds::{content::BackgroundTypes, Background, BackgroundPlugin},
        colors::{
            option_color, AlphaTranslation, Fade, BACKGROUND_COLOR, DIM_BACKGROUND_COLOR,
            PRIMARY_COLOR,
        },
        inheritance::BequeathTextAlpha,
        interaction::{Draggable, InteractionPlugin},
        motion::PointToPointTranslation,
        physics::ExplodedGlyph,
        scheduling::TimingPlugin,
        ui::window::UiWindowTitle,
    },
};
use bevy::{audio::Volume, prelude::*, sprite::Anchor, text::TextBounds};
use enum_map::Enum;
use phases::{
    consequence::DilemmaConsequencePlugin, decision::DilemmaDecisionPlugin,
    intro::DilemmaIntroPlugin, results::DilemmaResultsPlugin, skip::DilemmaSkipPlugin,
};
use std::{collections::HashSet, time::Duration};

pub mod phases;

pub mod dilemma;
use dilemma::{Dilemma, DilemmaPlugin};
pub mod content;
pub mod lever;
use content::DilemmaScene;
use lever::LeverPlugin;
mod junction;
use junction::JunctionPlugin;

use super::{Scene, SceneQueue};

pub struct DilemmaScenePlugin;
impl Plugin for DilemmaScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Dilemma), DilemmaScene::setup)
            .add_systems(
                OnExit(GameState::Dilemma),
                DilemmaScene::cleanup_detached_viscera,
            )
            .add_systems(
                OnEnter(MainState::Menu),
                DilemmaScene::cleanup_detached_viscera,
            )
            .add_plugins(DilemmaIntroPlugin)
            .add_plugins(DilemmaDecisionPlugin)
            .add_plugins(DilemmaTransitionPlugin)
            .add_plugins(DilemmaConsequencePlugin)
            .add_plugins(DilemmaResultsPlugin)
            .add_plugins(DilemmaSkipPlugin);

        if !app.is_plugin_added::<SpritePlugin>() {
            app.add_plugins(SpritePlugin);
        }
        if !app.is_plugin_added::<TextPlugin>() {
            app.add_plugins(TextPlugin);
        }
        if !app.is_plugin_added::<TrainPlugin>() {
            app.add_plugins(TrainPlugin);
        }
        if !app.is_plugin_added::<AsciiPlugin>() {
            app.add_plugins(AsciiPlugin);
        }
        if !app.is_plugin_added::<LeverPlugin>() {
            app.add_plugins(LeverPlugin);
        }
        if !app.is_plugin_added::<PersonPlugin>() {
            app.add_plugins(PersonPlugin);
        }
        if !app.is_plugin_added::<InteractionPlugin>() {
            app.add_plugins(InteractionPlugin);
        }
        if !app.is_plugin_added::<TimingPlugin>() {
            app.add_plugins(TimingPlugin);
        }
        if !app.is_plugin_added::<BackgroundPlugin>() {
            app.add_plugins(BackgroundPlugin);
        }
        if !app.is_plugin_added::<JunctionPlugin>() {
            app.add_plugins(JunctionPlugin);
        }
        if !app.is_plugin_added::<DilemmaPlugin>() {
            app.add_plugins(DilemmaPlugin);
        }
        if !app.is_plugin_added::<IOPlugin>() {
            app.add_plugins(IOPlugin);
        }
    }
}

impl DilemmaScene {
    const TRAIN_INITIAL_POSITION: Vec3 = Vec3::new(120.0, -10.0, 1.0);
    const MAIN_TRACK_TRANSLATION_END: Vec3 = Vec3::new(
        0.0,
        Self::TRAIN_INITIAL_POSITION.y + Train::track_alignment_offset_y(),
        0.0,
    );
    pub fn track_color_for_option(option_index: usize) -> Color {
        option_color(option_index)
    }

    fn setup(
        mut commands: Commands,
        queue: Res<SceneQueue>,
        asset_server: Res<AssetServer>,
        stats: Res<GameStats>,
    ) {
        let scene = queue.current;

        let dilemma = match scene {
            Scene::Dilemma(content) => Dilemma::new(&content),
            _ => panic!("Scene is not dilemma!"),
        };

        let total_dilemma_time: Duration =
            dilemma.stages.iter().map(|s| s.countdown_duration).sum();

        commands.insert_resource(DilemmaStats::new(total_dilemma_time));
        commands.insert_resource(DilemmaRunStatsScope::new(
            stats.dilemma_stats.len(),
            dilemma.stages.len(),
        ));

        commands.insert_resource(CurrentDilemmaStageIndex(0));

        let stage: &dilemma::DilemmaStage = dilemma.stages.first().expect("Dilemma has no stages!");

        let (transition_duration, train_x_displacement, _, _) =
            Self::generate_common_parameters(stage);

        commands.spawn((
            scene,
            DespawnOnExit(GameState::Dilemma),
            children![
                (
                    MusicAudio,
                    AudioPlayer::<AudioSource>(asset_server.load(dilemma.music_path.clone())),
                    PlaybackSettings {
                        paused: false,
                        volume: Volume::Linear(0.3),
                        ..continuous_audio()
                    }
                ),
                (
                    TextColor(PRIMARY_COLOR),
                    TextEmotion::Happy,
                    AsciiString(format!("DILEMMA {}", dilemma.index)),
                    Fade {
                        duration: transition_duration,
                        paused: true
                    },
                    Transform::from_xyz(0.0, 300.0, 1.0)
                ),
                (
                    TextWindow {
                        title: Some(UiWindowTitle {
                            text: format!("Description: {}", dilemma.name.clone()),
                            ..default()
                        }),
                        ..default()
                    },
                    TextBounds {
                        width: Some(400.0),
                        height: None
                    },
                    Draggable::default(),
                    TextColor(PRIMARY_COLOR),
                    Text2d::new(&dilemma.description),
                    TextFont {
                        font_size: scaled_font_size(12.0),
                        ..default()
                    },
                    Anchor::TOP_LEFT,
                    Transform::from_xyz(-600.0, 200.0, 2.0)
                ),
                (
                    TextColor(BACKGROUND_COLOR),
                    Background::new(
                        BackgroundTypes::Desert,
                        0.00002,
                        -0.5 * (dilemma.stages.first().expect("Dilemma has no stages").speed
                            / 70.0)
                    ),
                    BequeathTextAlpha,
                    AlphaTranslation::new(DIM_BACKGROUND_COLOR.alpha(), transition_duration, true)
                ),
                (
                    Train(TrainTypes::SteamTrain),
                    PointToPointTranslation::new(
                        Self::TRAIN_INITIAL_POSITION,
                        Self::TRAIN_INITIAL_POSITION + train_x_displacement,
                        transition_duration,
                        true
                    )
                )
            ],
        ));

        commands.spawn((
            DespawnOnExit(DilemmaPhase::Intro),
            TextColor(BACKGROUND_COLOR),
            Track::new(2000),
            Transform::from_translation(Self::MAIN_TRACK_TRANSLATION_END),
        ));

        commands.insert_resource(
            dilemma
                .stages
                .first()
                .expect("Dilemma has no stages!")
                .clone(),
        );
        commands.insert_resource(dilemma);
    }

    fn cleanup_detached_viscera(
        mut commands: Commands,
        exploded_query: Query<Entity, With<ExplodedGlyph>>,
        blood_query: Query<Entity, With<BloodSprite>>,
    ) {
        let mut to_despawn = HashSet::new();
        to_despawn.extend(exploded_query.iter());
        to_despawn.extend(blood_query.iter());

        for entity in to_despawn {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.despawn();
            }
        }
    }

    fn generate_common_parameters(stage: &DilemmaStage) -> (Duration, Vec3, Vec3, Color) {
        let decision_position = -stage.speed * stage.countdown_duration.as_secs_f32();
        let transition_duration =
            Duration::from_secs_f32(stage.countdown_duration.as_secs_f32() / 15.0);
        let train_x_displacement = Vec3::new(decision_position, 0.0, 0.0);
        let final_position = Vec3::new(150.0 * stage.countdown_duration.as_secs_f32(), 0.0, 0.0);
        let main_track_translation_start: Vec3 = Self::MAIN_TRACK_TRANSLATION_END + final_position;
        let initial_color: Color = match stage.default_option {
            None => Color::WHITE,
            Some(ref option) => Self::track_color_for_option(*option),
        };

        (
            transition_duration,
            train_x_displacement,
            main_track_translation_start,
            initial_color,
        )
    }
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaSounds {
    TrainApproaching,
    Clock,
    Click,
    Lever,
}
