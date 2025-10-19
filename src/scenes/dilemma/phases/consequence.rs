use std::{
    path::PathBuf, 
    time::Duration
};

use bevy::prelude::*;
use enum_map::{
    enum_map,
    Enum
};

use crate::{
    data::{
        states::{
            DilemmaPhase, GameState, MainState, StateVector
        },
        stats::GameStats
    }, entities::{
        text::TextButton, train::Train
    }, scenes::dilemma::{
        DilemmaSounds, dilemma::{CurrentDilemmaStageIndex,  Dilemma, DilemmaStage}, junction::Junction, lever::Lever
    }, style::common_ui::NextButton, systems::{
        audio::{
            OneShotAudio, 
            OneShotAudioPallet, 
            TransientAudio, 
            TransientAudioPallet 
        }, interaction::{
            ActionPallet, 
            InputAction, 
        }, motion::PointToPointTranslation, physics::Velocity, scheduling::{
            TimerConfig, 
            TimerPallet, 
            TimerStartCondition
        }, time::DilationTranslation
    }
};


#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaConsequenceEvents {
    SpeedUp,
	Scream,
	Button
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaConsequenceActions {
    ShowResults
}

impl std::fmt::Display for DilemmaConsequenceActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


pub struct DilemmaConsequencePlugin;
impl Plugin for DilemmaConsequencePlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(
			OnEnter(DilemmaPhase::Consequence),
			(
                DilemmaConsequenceScene::setup, 
                GameStats::update_stats
            )
			.run_if(in_state(GameState::Dilemma)),
		)
        .add_systems(
			OnEnter(DilemmaPhase::Results), 
			Junction::cleanup
		)
		.add_systems(
			Update,
			DilemmaConsequenceScene::spawn_delayed_children
			.run_if(in_state(GameState::Dilemma))
			.run_if(in_state(DilemmaPhase::Consequence)),
		).add_systems(
            OnExit(DilemmaPhase::Consequence),
            DilemmaConsequenceScene::update_stage
        );
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct DilemmaConsequenceScene;

impl DilemmaConsequenceScene{
    fn setup(
        stage : Res<DilemmaStage>,
        mut commands : Commands,
        mut velocity_query : Query<(Entity, &mut Velocity), With<Train>>,
        asset_server: Res<AssetServer>
    ) {
        for (entity, mut velocity) in velocity_query.iter_mut() {
            commands.entity(entity).remove::<PointToPointTranslation>();
            velocity.0 = Vec3::new(stage.speed, 0.0, 0.0);
        }
        
        commands.spawn((
            Self,
            DespawnOnExit(DilemmaPhase::Consequence),
            TimerPallet::new(
                vec![
                    (
                        DilemmaConsequenceEvents::SpeedUp,
                        TimerConfig::new(
                            TimerStartCondition::Immediate, 
                            3.0,
                            None
                        )
                    ),
                    (
                        DilemmaConsequenceEvents::Scream,
                        TimerConfig::new(
                            TimerStartCondition::Immediate, 
                            1.0,
                            None
                        )
                    ),
                    (
                        DilemmaConsequenceEvents::Button,
                        TimerConfig::new(
                            TimerStartCondition::Immediate, 
                            4.5,
                            None
                        )
                    )
                ]
            ), children![
                OneShotAudioPallet::new(
                    vec![
                        OneShotAudio {
                            source : asset_server.load(
                                PathBuf::from("./audio/effects/slowmo.ogg")
                            ),
                            ..default()
                        }
                    ]
                ),
                DilationTranslation::new(
                    0.1, 
                    Duration::from_secs_f32(1.0)
                )
            ]
        ));
    }
    
    fn spawn_delayed_children(
        mut commands: Commands,
        loading_query: Single<(Entity, &TimerPallet<DilemmaConsequenceEvents>), With<DilemmaConsequenceScene>>,
        dilemma: Res<Dilemma>,
        stage_index : Res<CurrentDilemmaStageIndex>,
        lever: Res<Lever>,
        asset_server: Res<AssetServer>,
        mut next_main_state: ResMut<NextState<MainState>>,
        mut next_game_state: ResMut<NextState<GameState>>,
        mut next_sub_state: ResMut<NextState<DilemmaPhase>>
    ) {
        const SCREAM_SOUND: &str = "./audio/effects/male_scream_long.ogg";
        const SPEEDUP_SOUND: &str = "./audio/effects/speedup.ogg";
        const SPEEDUP_DURATION_SECONDS: f32 = 1.057;
    
        // Determine if there are fatalities based on the current dilemma option.
        let stage = dilemma.stages[stage_index.0].clone();
        let are_fatalities = stage.options[lever.0 as usize].num_humans > 0;
        let num_stages = dilemma.stages.len();
    
        let (entity, timers) = loading_query.into_inner();
        
        // Spawn scream audio if the Scream event just finished and there are fatalities.
        if timers.0[DilemmaConsequenceEvents::Scream].just_finished() && are_fatalities {
            let scream_audio = OneShotAudio {
                source: asset_server.load(SCREAM_SOUND),
                dilatable : true,
                ..default()
            };

            commands.entity(entity).with_children(|parent| {
                parent.spawn(OneShotAudioPallet::new(vec![scream_audio]));
            });
        }

        // Spawn speedup audio with associated dilation translation if the SpeedUp event just finished.
        if timers.0[DilemmaConsequenceEvents::SpeedUp].just_finished() {
            let speedup_audio = OneShotAudio {
                source: asset_server.load(SPEEDUP_SOUND),
                ..default()
            };

            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    DilationTranslation::new(1.0, Duration::from_secs_f32(SPEEDUP_DURATION_SECONDS)),
                    OneShotAudioPallet::new(vec![speedup_audio]),
                ));
            });
        }

        if timers.0[DilemmaConsequenceEvents::Button].just_finished() { 

            if num_stages - 1 == stage_index.0 {
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        NextButton,
                        TextButton::new(
                            vec![DilemmaConsequenceActions::ShowResults],
                            vec![KeyCode::Enter],
                            format!("[Behold the consequences!]"),
                        ),
                        ActionPallet::<DilemmaConsequenceActions, DilemmaSounds>(
                            enum_map!(
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
                                )
                        ),
                        TransientAudioPallet::new(
                            vec![(
                                DilemmaSounds::Click,
                                vec![
                                    TransientAudio::new(
                                        asset_server.load("./audio/effects/mech_click.ogg"), 
                                        0.1, 
                                        true,
                                        1.0,
                                        true
                                    )
                                ]
                            )]
                        ),
                    ));
                });
            } else {
                let next_state = StateVector::new(None, None, Some(DilemmaPhase::DilemmaTransition));
                
                next_state.set_state(                        
                    &mut next_main_state,
                    &mut next_game_state,
                    &mut next_sub_state
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
}