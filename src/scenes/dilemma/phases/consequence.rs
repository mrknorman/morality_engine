use std::{path::PathBuf, time::Duration};

use bevy::prelude::*;
use enum_map::{enum_map, Enum};

use crate::{
    data::{
        states::{DilemmaPhase, GameState, MainState, StateVector},
        stats::GameStats,
    },
    entities::{person::PersonSprite, text::TextButton, train::Train},
    scenes::dilemma::{
        dilemma::{CurrentDilemmaStageIndex, Dilemma, DilemmaStage, DilemmaTimer},
        junction::Junction,
        DilemmaSounds,
    },
    style::common_ui::NextButton,
    systems::{
        audio::{OneShotAudio, OneShotAudioPallet, TransientAudio, TransientAudioPallet},
        interaction::{ActionPallet, InputAction},
        motion::PointToPointTranslation,
        physics::Velocity,
        scheduling::{TimerConfig, TimerPallet, TimerStartCondition},
        time::DilationTranslation,
    },
};

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaConsequenceEvents {
    SpeedUp,
    Button,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaConsequenceActions {
    ShowResults,
}

impl std::fmt::Display for DilemmaConsequenceActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct DilemmaConsequencePlugin;
impl Plugin for DilemmaConsequencePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PreConsequenceScreamState>()
        .add_systems(
            OnEnter(DilemmaPhase::Decision),
            DilemmaConsequenceScene::reset_pre_consequence_scream,
        )
        .add_systems(
            OnEnter(DilemmaPhase::Consequence),
            (DilemmaConsequenceScene::setup, GameStats::update_stats)
                .run_if(in_state(GameState::Dilemma)),
        )
        .add_systems(OnEnter(DilemmaPhase::Results), Junction::cleanup)
        .add_systems(
            Update,
            DilemmaConsequenceScene::trigger_pre_consequence_scream_from_decision
                .after(DilemmaTimer::update)
                .run_if(in_state(GameState::Dilemma))
                .run_if(in_state(DilemmaPhase::Decision)),
        )
        .add_systems(
            Update,
            DilemmaConsequenceScene::trigger_pre_consequence_scream_from_skip
                .run_if(in_state(GameState::Dilemma))
                .run_if(in_state(DilemmaPhase::Skip)),
        )
        .add_systems(
            Update,
            DilemmaConsequenceScene::spawn_delayed_children
                .run_if(in_state(GameState::Dilemma))
                .run_if(in_state(DilemmaPhase::Consequence)),
        )
        .add_systems(
            Update,
            DilemmaConsequenceScene::stop_scream_when_no_people_in_danger
                .run_if(in_state(GameState::Dilemma))
                .run_if(
                    in_state(DilemmaPhase::Decision)
                        .or(in_state(DilemmaPhase::Skip))
                        .or(in_state(DilemmaPhase::Consequence)),
                ),
        )
        .add_systems(
            OnExit(DilemmaPhase::Consequence),
            (
                DilemmaConsequenceScene::update_stage,
                DilemmaConsequenceScene::cleanup_pre_consequence_scream,
            ),
        );
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct DilemmaConsequenceScene;

#[derive(Component, Default)]
struct ConsequenceEventLatch {
    speedup_handled: bool,
    button_handled: bool,
}

#[derive(Component)]
pub struct ConsequenceScreamAudio;

#[derive(Resource, Default)]
struct PreConsequenceScreamState {
    played: bool,
}

impl DilemmaConsequenceScene {
    const PRE_CONSEQUENCE_SCREAM_SOUND: &str = "./audio/effects/male_scream_long.ogg";
    const PRE_CONSEQUENCE_SCREAM_LEAD_SECONDS: f32 = 1.0;

    fn reset_pre_consequence_scream(mut scream_state: ResMut<PreConsequenceScreamState>) {
        scream_state.played = false;
    }

    fn spawn_pre_consequence_scream(commands: &mut Commands, asset_server: &AssetServer) {
        commands.spawn((
            ConsequenceScreamAudio,
            OneShotAudioPallet::new(vec![OneShotAudio {
                source: asset_server.load(Self::PRE_CONSEQUENCE_SCREAM_SOUND),
                dilatable: true,
                ..default()
            }]),
        ));
    }

    fn trigger_pre_consequence_scream_from_decision(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        timer_query: Query<&DilemmaTimer>,
        person_query: Query<&PersonSprite>,
        scream_query: Query<Entity, With<ConsequenceScreamAudio>>,
        mut scream_state: ResMut<PreConsequenceScreamState>,
    ) {
        if scream_state.played || !scream_query.is_empty() {
            scream_state.played = true;
            return;
        }

        if !person_query.iter().any(|person| person.in_danger) {
            return;
        }

        let Some(timer) = timer_query.iter().next() else {
            return;
        };

        if timer.timer.remaining_secs() <= Self::PRE_CONSEQUENCE_SCREAM_LEAD_SECONDS {
            Self::spawn_pre_consequence_scream(&mut commands, &asset_server);
            scream_state.played = true;
        }
    }

    fn trigger_pre_consequence_scream_from_skip(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        translation_query: Query<&PointToPointTranslation, With<Train>>,
        person_query: Query<&PersonSprite>,
        scream_query: Query<Entity, With<ConsequenceScreamAudio>>,
        mut scream_state: ResMut<PreConsequenceScreamState>,
    ) {
        if scream_state.played || !scream_query.is_empty() {
            scream_state.played = true;
            return;
        }

        if !person_query.iter().any(|person| person.in_danger) {
            return;
        }

        let Some(translation) = translation_query.iter().next() else {
            return;
        };

        if translation.timer.remaining_secs() <= Self::PRE_CONSEQUENCE_SCREAM_LEAD_SECONDS {
            Self::spawn_pre_consequence_scream(&mut commands, &asset_server);
            scream_state.played = true;
        }
    }

    fn setup(
        stage: Res<DilemmaStage>,
        mut commands: Commands,
        mut velocity_query: Query<(Entity, &mut Velocity), With<Train>>,
        asset_server: Res<AssetServer>,
    ) {
        for (entity, mut velocity) in velocity_query.iter_mut() {
            commands.entity(entity).remove::<PointToPointTranslation>();
            velocity.0 = Vec3::new(stage.speed, 0.0, 0.0);
        }

        commands.spawn((
            Self,
            ConsequenceEventLatch::default(),
            DespawnOnExit(DilemmaPhase::Consequence),
            TimerPallet::new(vec![
                (
                    DilemmaConsequenceEvents::SpeedUp,
                    TimerConfig::new(TimerStartCondition::Immediate, 3.0, None),
                ),
                (
                    DilemmaConsequenceEvents::Button,
                    TimerConfig::new(TimerStartCondition::Immediate, 4.5, None),
                ),
            ]),
            children![
                OneShotAudioPallet::new(vec![OneShotAudio {
                    source: asset_server.load(PathBuf::from("./audio/effects/slowmo.ogg")),
                    ..default()
                }]),
                DilationTranslation::new(0.1, Duration::from_secs_f32(1.0))
            ],
        ));
    }

    fn spawn_delayed_children(
        mut commands: Commands,
        loading_query: Single<
            (
                Entity,
                &TimerPallet<DilemmaConsequenceEvents>,
                &mut ConsequenceEventLatch,
            ),
            With<DilemmaConsequenceScene>,
        >,
        dilemma: Res<Dilemma>,
        stage_index: Res<CurrentDilemmaStageIndex>,
        asset_server: Res<AssetServer>,
        mut next_main_state: ResMut<NextState<MainState>>,
        mut next_game_state: ResMut<NextState<GameState>>,
        mut next_sub_state: ResMut<NextState<DilemmaPhase>>,
    ) {
        const SPEEDUP_SOUND: &str = "./audio/effects/speedup.ogg";
        const SPEEDUP_DURATION_SECONDS: f32 = 1.057;

        let num_stages = dilemma.stages.len();

        let (entity, timers, mut latch) = loading_query.into_inner();

        if !latch.speedup_handled && timers.0[DilemmaConsequenceEvents::SpeedUp].times_finished() > 0
        {
            let speedup_audio = OneShotAudio {
                source: asset_server.load(SPEEDUP_SOUND),
                ..default()
            };

            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    DilationTranslation::new(
                        1.0,
                        Duration::from_secs_f32(SPEEDUP_DURATION_SECONDS),
                    ),
                    OneShotAudioPallet::new(vec![speedup_audio]),
                ));
            });

            latch.speedup_handled = true;
        }

        if !latch.button_handled && timers.0[DilemmaConsequenceEvents::Button].times_finished() > 0 {
            latch.button_handled = true;
            if num_stages - 1 == stage_index.0 {
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        NextButton,
                        TextButton::new(
                            vec![DilemmaConsequenceActions::ShowResults],
                            vec![KeyCode::Enter],
                            format!("[Behold the consequences!]"),
                        ),
                        ActionPallet::<DilemmaConsequenceActions, DilemmaSounds>(enum_map!(
                        DilemmaConsequenceActions::ShowResults => vec![
                            InputAction::PlaySound(DilemmaSounds::Click),
                            InputAction::ChangeState(
                                StateVector::new(
                                    None,
                                    None,
                                    Some(DilemmaPhase::Results),
                                )
                            ),
                            InputAction::Despawn(None)
                            ]
                        )),
                        TransientAudioPallet::new(vec![(
                            DilemmaSounds::Click,
                            vec![TransientAudio::new(
                                asset_server.load("./audio/effects/mech_click.ogg"),
                                0.1,
                                true,
                                1.0,
                                true,
                            )],
                        )]),
                    ));
                });
            } else {
                let next_state =
                    StateVector::new(None, None, Some(DilemmaPhase::DilemmaTransition));

                next_state.set_state(
                    &mut next_main_state,
                    &mut next_game_state,
                    &mut next_sub_state,
                );
            }
        }
    }

    fn update_stage(
        dilemma: Res<Dilemma>,
        mut stage_index: ResMut<CurrentDilemmaStageIndex>,
        mut stage: ResMut<DilemmaStage>,
    ) {
        stage_index.0 += 1;

        if let Some(next_stage) = dilemma.stages.get(stage_index.0) {
            *stage = next_stage.clone();
        }
    }

    fn cleanup_pre_consequence_scream(
        mut commands: Commands,
        scream_audio_query: Query<Entity, With<ConsequenceScreamAudio>>,
        mut scream_state: ResMut<PreConsequenceScreamState>,
    ) {
        for entity in scream_audio_query.iter() {
            commands.entity(entity).despawn();
        }
        scream_state.played = false;
    }

    fn stop_scream_when_no_people_in_danger(
        mut commands: Commands,
        person_query: Query<&PersonSprite>,
        scream_audio_query: Query<Entity, With<ConsequenceScreamAudio>>,
    ) {
        if person_query.iter().any(|person| person.in_danger) {
            return;
        }

        for entity in scream_audio_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}
