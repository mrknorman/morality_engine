use crate::{
    data::{
        rng::GlobalRng,
        states::{DilemmaPhase, GameState, MainState},
        stats::{DilemmaRunStatsScope, DilemmaStats, GameStats},
    },
    entities::{
        large_fonts::{AsciiString, TextEmotion},
        person::{BloodSprite, PersonPlugin},
        text::{scaled_font_size, TextFrames, TextSprite, TextWindow},
        track::Track,
        train::Train,
    },
    scenes::dilemma::{
        dilemma::{CurrentDilemmaStageIndex, DilemmaStage},
        phases::transition::DilemmaTransitionPlugin,
    },
    scenes::runtime::SceneNavigator,
    systems::{
        audio::{continuous_audio, MusicAudio},
        backgrounds::{parallax_speed, Background, BackgroundSprite},
        colors::{
            option_color, AlphaTranslation, Fade, BACKGROUND_COLOR, DIM_BACKGROUND_COLOR,
            PRIMARY_COLOR,
        },
        inheritance::BequeathTextAlpha,
        interaction::Draggable,
        motion::PointToPointTranslation,
        physics::{CameraVelocity, ExplodedGlyph},
        ui::window::UiWindowTitle,
    },
};
use bevy::{audio::Volume, prelude::*, sprite::Anchor, text::TextBounds, window::PrimaryWindow};
use enum_map::Enum;
use phases::{
    consequence::DilemmaConsequencePlugin, decision::DilemmaDecisionPlugin,
    intro::DilemmaIntroPlugin, results::DilemmaResultsPlugin, skip::DilemmaSkipPlugin,
};
use rand::Rng;
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
pub mod visuals;
use visuals::{
    resolve_visuals, smoke_frames, AmbientBackgroundElement, AmbientSmokeAnimation,
    AmbientSmokePlume, AMBIENT_BLOOD_GLYPHS, AMBIENT_BODY_PART_GLYPHS,
};

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
                Update,
                AmbientSmokeAnimation::animate.run_if(in_state(GameState::Dilemma)),
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

        if !app.is_plugin_added::<LeverPlugin>() {
            app.add_plugins(LeverPlugin);
        }
        if !app.is_plugin_added::<PersonPlugin>() {
            app.add_plugins(PersonPlugin);
        }
        if !app.is_plugin_added::<JunctionPlugin>() {
            app.add_plugins(JunctionPlugin);
        }
        if !app.is_plugin_added::<DilemmaPlugin>() {
            app.add_plugins(DilemmaPlugin);
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
        mut rng: ResMut<GlobalRng>,
        window: Single<&Window, With<PrimaryWindow>>,
        stats: Res<GameStats>,
        mut next_main_state: ResMut<NextState<MainState>>,
        mut next_game_state: ResMut<NextState<GameState>>,
        mut next_sub_state: ResMut<NextState<DilemmaPhase>>,
    ) {
        let scene = queue.current_scene();

        let dilemma = match scene {
            Scene::Dilemma(content) => match Dilemma::try_new(&content) {
                Ok(dilemma) => dilemma,
                Err(error) => {
                    warn!("failed to load dilemma content: {error}; falling back to menu");
                    SceneNavigator::fallback_state_vector().set_state(
                        &mut next_main_state,
                        &mut next_game_state,
                        &mut next_sub_state,
                    );
                    return;
                }
            },
            _ => {
                warn!("expected dilemma scene but found non-dilemma route; falling back to menu");
                SceneNavigator::fallback_state_vector().set_state(
                    &mut next_main_state,
                    &mut next_game_state,
                    &mut next_sub_state,
                );
                return;
            }
        };

        let stage = match dilemma.stages.first().cloned() {
            Some(stage) => stage,
            None => {
                warn!("dilemma content has no stages; falling back to menu");
                SceneNavigator::fallback_state_vector().set_state(
                    &mut next_main_state,
                    &mut next_game_state,
                    &mut next_sub_state,
                );
                return;
            }
        };

        let total_dilemma_time: Duration =
            dilemma.stages.iter().map(|s| s.countdown_duration).sum();

        commands.insert_resource(DilemmaStats::new(total_dilemma_time));
        commands.insert_resource(DilemmaRunStatsScope::new(
            stats.dilemma_stats.len(),
            dilemma.stages.len(),
        ));

        commands.insert_resource(CurrentDilemmaStageIndex(0));

        let (transition_duration, train_x_displacement, _, _) =
            Self::generate_common_parameters(&stage);

        let resolved_visuals = resolve_visuals(&dilemma.visuals, stage.speed);
        let smoke_frames = smoke_frames();
        let screen_width = window.width() / 2.0 + 100.0;
        let screen_height = window.height();
        let perspective_scale = |y: f32| -> f32 {
            let t = ((y + screen_height / 2.0) / screen_height).clamp(0.0, 1.0);
            1.0 - 0.5 * t
        };
        let mut background_entities: Vec<(Entity, f32)> = Vec::new();
        const BACKGROUND_SPAWN_VARIANCE: f32 = 100.0;

        let scene_root = commands
            .spawn((scene, DespawnOnExit(GameState::Dilemma)))
            .id();
        commands.entity(scene_root).with_children(|parent| {
            parent.spawn((
                MusicAudio,
                AudioPlayer::<AudioSource>(asset_server.load(dilemma.music_path.clone())),
                PlaybackSettings {
                    paused: false,
                    volume: Volume::Linear(0.3),
                    ..continuous_audio()
                },
            ));

            parent.spawn((
                TextColor(PRIMARY_COLOR),
                TextEmotion::Happy,
                AsciiString(format!("DILEMMA {}", dilemma.index)),
                Fade {
                    duration: transition_duration,
                    paused: true,
                },
                Transform::from_xyz(0.0, 300.0, 1.0),
            ));

            parent.spawn((
                TextWindow {
                    title: Some(UiWindowTitle {
                        text: format!("Description: {}", dilemma.name.clone()),
                        ..default()
                    }),
                    ..default()
                },
                TextBounds {
                    width: Some(400.0),
                    height: None,
                },
                Draggable::default(),
                TextColor(PRIMARY_COLOR),
                Text2d::new(&dilemma.description),
                TextFont {
                    font_size: scaled_font_size(12.0),
                    ..default()
                },
                Anchor::TOP_LEFT,
                Transform::from_xyz(-600.0, 200.0, 2.0),
            ));

            for layer in &resolved_visuals.background_layers {
                let target_alpha =
                    (DIM_BACKGROUND_COLOR.alpha() * layer.alpha_multiplier).clamp(0.0, 1.0);
                let background_entity = parent
                    .spawn((
                        TextColor(BACKGROUND_COLOR),
                        Background::new(layer.background_type, layer.density, layer.speed),
                        BequeathTextAlpha,
                        AlphaTranslation::new(target_alpha, transition_duration, true),
                    ))
                    .id();
                background_entities.push((background_entity, layer.speed));
            }

            parent.spawn((
                Train(dilemma.train),
                PointToPointTranslation::new(
                    Self::TRAIN_INITIAL_POSITION,
                    Self::TRAIN_INITIAL_POSITION + train_x_displacement,
                    transition_duration,
                    true,
                ),
            ));
        });

        let primary_background = background_entities.first().copied();
        if let Some((primary_background_entity, background_speed)) = primary_background {
            commands
                .entity(primary_background_entity)
                .with_children(|background_parent| {
                    if let Some(ambient_smoke) = &resolved_visuals.ambient_smoke {
                        if !smoke_frames.is_empty() {
                            let smoke_text_frames = TextFrames::new(smoke_frames.clone());
                            for plume_index in 0..ambient_smoke.count {
                                let x = rng
                                    .uniform
                                    .random_range(ambient_smoke.min_x..=ambient_smoke.max_x);
                                let y = rng
                                    .uniform
                                    .random_range(ambient_smoke.min_y..=ambient_smoke.max_y);
                                let frame_index = plume_index % smoke_frames.len();
                                let scale_factor = perspective_scale(y);
                                let random_offset = rng.uniform.random_range(
                                    screen_width..screen_width + BACKGROUND_SPAWN_VARIANCE,
                                );
                                let speed = parallax_speed(
                                    screen_height,
                                    y,
                                    background_speed,
                                    scale_factor,
                                );

                                background_parent.spawn((
                                    AmbientBackgroundElement,
                                    AmbientSmokePlume,
                                    AmbientSmokeAnimation::new(
                                        ambient_smoke.frame_seconds,
                                        frame_index,
                                    ),
                                    BackgroundSprite::new(screen_width, random_offset),
                                    CameraVelocity(Vec3::new(speed, 0.0, 0.0)),
                                    TextSprite,
                                    Text2d::new(smoke_frames[frame_index].clone()),
                                    smoke_text_frames.clone(),
                                    TextColor(BACKGROUND_COLOR),
                                    Transform {
                                        translation: Vec3::new(x, y, 0.0),
                                        scale: Vec3::splat(scale_factor),
                                        ..default()
                                    },
                                    BequeathTextAlpha,
                                    AlphaTranslation::new(
                                        (DIM_BACKGROUND_COLOR.alpha() * 0.9).clamp(0.0, 1.0),
                                        transition_duration,
                                        true,
                                    ),
                                ));
                            }
                        }
                    }

                    if let Some(ambient_viscera) = &resolved_visuals.ambient_viscera {
                        for _ in 0..ambient_viscera.body_parts_count {
                            let x = rng
                                .uniform
                                .random_range(ambient_viscera.min_x..=ambient_viscera.max_x);
                            let y = rng
                                .uniform
                                .random_range(ambient_viscera.min_y..=ambient_viscera.max_y);
                            let glyph_index =
                                rng.uniform.random_range(0..AMBIENT_BODY_PART_GLYPHS.len());
                            let glyph = AMBIENT_BODY_PART_GLYPHS[glyph_index];
                            let tint = rng.uniform.random_range(0.0..=1.8);
                            let scale_factor = perspective_scale(y);
                            let random_offset = rng.uniform.random_range(
                                screen_width..screen_width + BACKGROUND_SPAWN_VARIANCE,
                            );
                            let speed =
                                parallax_speed(screen_height, y, background_speed, scale_factor);

                            background_parent.spawn((
                                AmbientBackgroundElement,
                                ExplodedGlyph,
                                BackgroundSprite::new(screen_width, random_offset),
                                CameraVelocity(Vec3::new(speed, 0.0, 0.0)),
                                TextSprite,
                                Text2d::new(glyph),
                                TextColor(Color::srgba(2.5, tint, tint, 1.0)),
                                Transform {
                                    translation: Vec3::new(x, y, 0.0),
                                    scale: Vec3::splat(scale_factor),
                                    ..default()
                                },
                                BequeathTextAlpha,
                                AlphaTranslation::new(
                                    (DIM_BACKGROUND_COLOR.alpha() * 0.95).clamp(0.0, 1.0),
                                    transition_duration,
                                    true,
                                ),
                            ));
                        }

                        for _ in 0..ambient_viscera.blood_count {
                            let x = rng
                                .uniform
                                .random_range(ambient_viscera.min_x..=ambient_viscera.max_x);
                            let y = rng
                                .uniform
                                .random_range(ambient_viscera.min_y..=ambient_viscera.max_y);
                            let glyph_index =
                                rng.uniform.random_range(0..AMBIENT_BLOOD_GLYPHS.len());
                            let glyph = AMBIENT_BLOOD_GLYPHS[glyph_index];
                            let scale_factor = perspective_scale(y);
                            let random_offset = rng.uniform.random_range(
                                screen_width..screen_width + BACKGROUND_SPAWN_VARIANCE,
                            );
                            let speed =
                                parallax_speed(screen_height, y, background_speed, scale_factor);

                            background_parent.spawn((
                                AmbientBackgroundElement,
                                BloodSprite(rng.uniform.random_range(1..=3)),
                                BackgroundSprite::new(screen_width, random_offset),
                                CameraVelocity(Vec3::new(speed, 0.0, 0.0)),
                                TextSprite,
                                Text2d::new(glyph),
                                TextColor(Color::srgba(2.0, 0.0, 0.0, 1.0)),
                                Transform {
                                    translation: Vec3::new(x, y, 0.0),
                                    scale: Vec3::splat(scale_factor),
                                    ..default()
                                },
                                BequeathTextAlpha,
                                AlphaTranslation::new(
                                    (DIM_BACKGROUND_COLOR.alpha() * 0.95).clamp(0.0, 1.0),
                                    transition_duration,
                                    true,
                                ),
                            ));
                        }
                    }
                });
        }

        commands.spawn((
            DespawnOnExit(DilemmaPhase::Intro),
            TextColor(BACKGROUND_COLOR),
            Track::new(2000),
            Transform::from_translation(Self::MAIN_TRACK_TRANSLATION_END),
        ));

        commands.insert_resource(stage);
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
