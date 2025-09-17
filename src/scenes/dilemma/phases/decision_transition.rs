use std::time::Duration;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    data::states::{
        DilemmaPhase, 
        GameState
    }, entities::{person::BloodSprite, train::Train}, scenes::dilemma::{
        content::DilemmaScene, dilemma::{CurrentDilemmaStage, Dilemma}, junction::Junction, lever::{Lever, LeverState}
    }, systems::{
        backgrounds::{
            Background, 
            BackgroundSystems
        }, 
        colors::{AlphaTranslation, BACKGROUND_COLOR, ColorTranslation, Fade}, 
        inheritance::BequeathTextColor,
        motion::{
            Bounce, 
            PointToPointTranslation
        }, physics::{DespawnOffscreen, ExplodedGlyph, Velocity}, time::Dilation, 
    }
};

pub struct DilemmaDecisionTransitionPlugin;
impl Plugin for DilemmaDecisionTransitionPlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(
			OnEnter(DilemmaPhase::DecisionDecisionTransition), 
			setup_dilemma_decision_transition
		)
		.add_systems(
			Update,
			end_dilemma_decision_transition
			.run_if(in_state(GameState::Dilemma))
			.run_if(in_state(DilemmaPhase::DecisionDecisionTransition)),
		).add_systems(
			Update,
			update_viscera_speeds
			.run_if(in_state(DilemmaPhase::DecisionDecisionTransition)),
		);
    }
}

fn setup_dilemma_decision_transition(
    dilemma : Res<Dilemma>,
    mut lever : ResMut<Lever>,
    mut current_dilemma_stage : ResMut<CurrentDilemmaStage>,
    mut commands : Commands,
    systems: Res<BackgroundSystems>,
    mut background_query : Query<(Entity, &mut Background)>,
    mut train_query : Query<(Entity, &mut Velocity, &Transform), (With<Train>, Without<Junction>)>,
    mut junction_query: Query<(Entity, &Transform), (With<Junction>, Without<Train>)>,
) { 

        current_dilemma_stage.0 += 1;

        let current_stage = dilemma.stages[current_dilemma_stage.0].clone();

        lever.0 = match current_stage.default_option {
			None => LeverState::Random,
			Some(ref option) if *option == 0 => LeverState::Left,
			Some(_) => LeverState::Right
		};

    	let decision_position = -70.0 * current_stage.countdown_duration.as_secs_f32();
		let transition_duration = Duration::from_secs_f32(decision_position/ DilemmaScene::TRAIN_SPEED);
		let train_x_displacement = Vec3::new(decision_position, 0.0, 0.0);
		let final_position = Vec3::new(
			150.0 * current_stage.countdown_duration.as_secs_f32(),
			0.0, 
			0.0
		);
		let main_track_translation_start: Vec3 = DilemmaScene::MAIN_TRACK_TRANSLATION_END + final_position;
		let initial_color = match current_stage.default_option {
			None => Color::WHITE,
			Some(ref option) => DilemmaScene::TRACK_COLORS[*option]
		};
        
    commands.spawn(
(
            StateScoped(GameState::Dilemma),
            DespawnOffscreen{
                margin : 10.0
            },
            Junction{
                dilemma : dilemma.clone()
            },
            TextColor(BACKGROUND_COLOR),
            ColorTranslation::new(
                initial_color,
                transition_duration,
                false
            ),
            PointToPointTranslation::new(
                main_track_translation_start,
                DilemmaScene::MAIN_TRACK_TRANSLATION_END,
                transition_duration,
                false
            )
        )
    );

    let mut train_transform = Transform::default();
    let mut train_velocity = Vec3::ZERO;
    for (entity, mut velocity, transform) in train_query.iter_mut() {
        train_velocity = velocity.0;
        train_transform = transform.clone();
        velocity.0 = Vec3::ZERO;
        commands.entity(entity).insert(PointToPointTranslation::new(
            transform.translation,
            DilemmaScene::TRAIN_INITIAL_POSITION + train_x_displacement,
            transition_duration,
            false
        ));
    }
    for (entity, transform) in junction_query.iter_mut() {

        let displacement = transform.translation - train_transform.translation;

        commands.entity(entity).insert(
            PointToPointTranslation::new(
                transform.translation,
                DilemmaScene::TRAIN_INITIAL_POSITION + train_x_displacement + displacement - Vec3::ZERO.with_x(train_velocity.x * transition_duration.as_secs_f32()),
                transition_duration,
                false
            )
        );
    }
    
    for (entity, mut background) in background_query.iter_mut() {
        commands.entity(entity).insert(
            AlphaTranslation {
                initial_alpha : 0.0,
                final_alpha : 1.0,
                timer : Timer::new(
                    transition_duration,
                    TimerMode::Once
                )
            }
        );
        background.speed = -current_stage.countdown_duration.as_secs_f32() / 5.0;
        commands.run_system(systems.0["update_background_speeds"]);
    }
}

fn end_dilemma_decision_transition(
    dilemma: Res<Dilemma>,
    current_dilemma_stage : Res<CurrentDilemmaStage>,
    mut commands : Commands,
    systems: Res<BackgroundSystems>,
    mut next_sub_state: ResMut<NextState<DilemmaPhase>>,
    mut train_query : Query<&mut PointToPointTranslation, (With<Train>, Without<Junction>)>,
    mut junction_query: Query<&mut PointToPointTranslation, (With<Junction>, Without<Train>)>,
    mut background_query : Query<&mut Background>
) {

    let current_stage = dilemma.stages[current_dilemma_stage.0].clone();

    let mut all_translations_finished = true;
    for translation in train_query.iter_mut() {
        all_translations_finished &= translation.timer.finished();
    }
    for translation in junction_query.iter_mut() {
        all_translations_finished &= translation.timer.finished();
    }

    if all_translations_finished {

        for mut background in background_query.iter_mut() {
            background.speed = 0.0;
            commands.run_system(systems.0["update_background_speeds"]);
        }
        
        next_sub_state.set(
            DilemmaPhase::Decision
        );

        for mut translation in train_query.iter_mut() {
            translation.initial_position = translation.final_position;
            translation.final_position = DilemmaScene::TRAIN_INITIAL_POSITION + Vec3::new(-50.0, 0.0, 0.0);
            translation.timer = Timer::new(
                current_stage.countdown_duration,
                TimerMode::Once
            );
        }
    }
}

   pub fn update_viscera_speeds(
        dilemma: Res<Dilemma>,
        current_dilemma_stage : Res<CurrentDilemmaStage>,
        dilation : Res<Dilation>,
        time: Res<Time>,
        window: Single<&Window, With<PrimaryWindow>>,
        mut sprites: Query<&mut Transform, Or<(With<BloodSprite>, With<ExplodedGlyph>)>>,
    ) {
        let screen_height = window.height();
        let duration_seconds = time.delta_secs()*dilation.0;
        let current_stage = dilemma.stages[current_dilemma_stage.0].clone();
    
        for mut transform in &mut sprites {            
            let distance_from_bottom = (screen_height - transform.translation.y).max(0.0);
            let x_speed = distance_from_bottom * -current_stage.countdown_duration.as_secs_f32() / 5.0;
            transform.translation.x += (x_speed/2.5)*duration_seconds;
        }
    }