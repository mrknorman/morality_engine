use std::{iter::zip, time::Duration};

use bevy::{audio::Volume, prelude::*, sprite::Anchor, text::TextBounds};
use enum_map::{enum_map, Enum};

use crate::{
    data::{
        states::{DilemmaPhase, GameState, StateVector},
        stats::DilemmaStats,
    },
    entities::{
        sprites::window::WindowTitle,
        text::{scaled_font_size, TextTitle, TextWindow},
        track::Track,
    },
    scenes::dilemma::{
        dilemma::{CurrentDilemmaStageIndex, DilemmaStage, DilemmaTimer},
        lever::{Lever, LeverState, LEVER_LEFT, LEVER_MIDDLE, LEVER_RIGHT},
        DilemmaSounds,
    },
    style::common_ui::{CenterLever, DilemmaTimerPosition},
    systems::{
        audio::{
            continuous_audio, ContinuousAudio, ContinuousAudioPallet, TransientAudio,
            TransientAudioPallet,
        },
        backgrounds::Background,
        colors::{
            ColorAnchor, ColorChangeEvent, ColorChangeOn, ColorTranslation, DANGER_COLOR,
            OPTION_1_COLOR, OPTION_2_COLOR, PRIMARY_COLOR,
        },
        interaction::{ActionPallet, ClickablePong, Draggable, InputAction, KeyMapping, Pressable},
    },
};

pub struct DilemmaDecisionPlugin;
impl Plugin for DilemmaDecisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(DilemmaPhase::Decision),
            DecisionScene::setup.run_if(in_state(GameState::Dilemma)),
        )
        .add_systems(
            Update,
            DecisionScene::update_stats.run_if(resource_changed::<Lever>),
        )
        .add_systems(
            OnExit(DilemmaPhase::Decision),
            (DecisionScene::cleanup, DecisionScene::finalize_stats),
        );
    }
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionActions {
    LockDecision,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeverActions {
    LeftPull,
    RightPull,
}

#[derive(Component)]
#[require(Transform, Visibility)]
struct DecisionScene;

impl DecisionScene {
    const TITLE_TRANSLATION: Vec3 = Vec3::new(0.0, -100.0, 1.0);
    const OPTION_WINDOW_TRANSLATIONS: [Vec3; 2] = [
        Vec3::new(-600.0, -200.0, 2.0),
        Vec3::new(200.0, -200.0, 2.0),
    ];

    fn setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        stage: Res<DilemmaStage>,
        index: Res<CurrentDilemmaStageIndex>,
        lever: Res<Lever>,
    ) {
        let lever_state = if index.0 == 0 {
            stage.default_option
        } else {
            lever.0.to_int()
        };

        let (start_text, state, color) = match lever_state {
            None => (LEVER_MIDDLE, LeverState::Random, Color::WHITE),
            Some(ref option) if *option == 0 => (LEVER_LEFT, LeverState::Left, OPTION_1_COLOR),
            Some(_) => (LEVER_RIGHT, LeverState::Right, OPTION_2_COLOR),
        };

        commands
            .spawn((
                DespawnOnExit(DilemmaPhase::Decision),
                DecisionScene,
                children![
                    ContinuousAudioPallet::new(vec![
                        ContinuousAudio {
                            key: DilemmaSounds::TrainApproaching,
                            source: AudioPlayer::<AudioSource>(
                                asset_server.load("./audio/effects/train/approaching.ogg")
                            ),
                            settings: PlaybackSettings {
                                volume: Volume::Linear(1.0),
                                ..continuous_audio()
                            },
                            dilatable: true
                        },
                        ContinuousAudio {
                            key: DilemmaSounds::Clock,
                            source: AudioPlayer::<AudioSource>(
                                asset_server.load("./audio/effects/clock.ogg")
                            ),
                            settings: PlaybackSettings {
                                volume: Volume::Linear(0.3),
                                ..continuous_audio()
                            },
                            dilatable: true
                        }
                    ]),
                    (
                        Pressable::new(vec![KeyMapping {
                            keys: vec![KeyCode::Enter],
                            actions: vec![DecisionActions::LockDecision],
                            allow_repeated_activation: false
                        }]),
                        ActionPallet(enum_map!(
                            DecisionActions::LockDecision => vec![
                                InputAction::ChangeState(
                                    StateVector::new(
                                        None, None, Some(DilemmaPhase::Skip)
                                    ),
                                ),
                                InputAction::PlaySound(DilemmaSounds::Lever)
                            ]
                        ))
                    ),
                    (
                        TextTitle,
                        DilemmaTimerPosition,
                        DilemmaTimer::new(
                            stage.countdown_duration,
                            Duration::from_secs_f32(5.0),
                            Duration::from_secs_f32(2.0)
                        ),
                        ColorAnchor::default(),
                        ColorChangeOn::new(vec![ColorChangeEvent::Pulse(vec![DANGER_COLOR])]),
                        Transform::from_translation(Self::TITLE_TRANSLATION)
                    ),
                    (
                        Lever(state),
                        ClickablePong::new(
                            vec![vec![LeverActions::RightPull], vec![LeverActions::LeftPull]],
                            lever_state.unwrap_or(0)
                        ),
                        Pressable::new(vec![
                            KeyMapping {
                                keys: vec![KeyCode::Digit2],
                                actions: vec![LeverActions::RightPull],
                                allow_repeated_activation: false
                            },
                            KeyMapping {
                                keys: vec![KeyCode::Digit1],
                                actions: vec![LeverActions::LeftPull],
                                allow_repeated_activation: false
                            }
                        ]),
                        ActionPallet(enum_map!(
                            LeverActions::LeftPull => vec![
                                InputAction::ChangeLeverState(LeverState::Left),
                                InputAction::PlaySound(DilemmaSounds::Lever),
                            ],
                            LeverActions::RightPull => vec![
                                InputAction::ChangeLeverState(LeverState::Right),
                                InputAction::PlaySound(DilemmaSounds::Lever),
                            ]
                        )),
                        CenterLever,
                        Text2d::new(start_text),
                        TextFont {
                            font_size: scaled_font_size(25.0),
                            ..default()
                        },
                        TextColor(color),
                        TextLayout {
                            justify: Justify::Center,
                            ..default()
                        },
                        TransientAudioPallet::new(vec![(
                            DilemmaSounds::Lever,
                            vec![TransientAudio::new(
                                asset_server.load("./audio/effects/switch.ogg"),
                                0.1,
                                true,
                                1.0,
                                true
                            )]
                        )]),
                    )
                ],
            ))
            .with_children(|parent| {
                for (option, transform) in zip(
                    stage.options.clone(),
                    Self::OPTION_WINDOW_TRANSLATIONS.iter(),
                ) {
                    parent.spawn((
                        TextWindow {
                            title: Some(WindowTitle {
                                text: format!(
                                    "Option {}: {} [Press {} to select]\n",
                                    option.index + 1,
                                    option.name,
                                    option.index + 1
                                ),
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
                        Text2d::new(&option.description),
                        TextFont {
                            font_size: scaled_font_size(12.0),
                            ..default()
                        },
                        Anchor::TOP_LEFT,
                        Transform::from_translation(*transform),
                    ));
                }
            });
    }

    fn cleanup(
        mut commands: Commands,
        background_query: Query<Entity, With<Background>>,
        track_query: Query<Entity, With<Track>>,
    ) {
        for entity in background_query.iter() {
            commands.entity(entity).insert(ColorTranslation::new(
                Color::NONE,
                Duration::from_secs_f32(0.4),
                false,
            ));
        }
        for entity in track_query.iter() {
            commands.entity(entity).insert(ColorTranslation::new(
                Color::NONE,
                Duration::from_secs_f32(0.4),
                false,
            ));
        }
    }

    fn update_stats(
        mut stats: ResMut<DilemmaStats>,
        lever: Res<Lever>,
        mut timer: Query<&mut DilemmaTimer>,
    ) {
        for timer in timer.iter_mut() {
            stats.update(&lever.0, &timer.timer);
        }
    }

    fn finalize_stats(
        mut stats: ResMut<DilemmaStats>,
        lever: Res<Lever>,
        stage: Res<DilemmaStage>,
        mut timer: Query<&mut DilemmaTimer>,
    ) {
        let consequence = stage.options[lever.0 as usize].consequences;

        for timer in timer.iter_mut() {
            stats.finalize(&consequence, &lever.0, &timer.timer);
        }
    }
}
