use std::time::Duration;

use bevy::prelude::*;

use crate::{
    data::states::{
        DilemmaPhase, 
        GameState
    }, entities::train::Train, scenes::dilemma::{
        content::DilemmaScene, dilemma::Dilemma, junction::Junction
    }, systems::{
        backgrounds::{
            Background, 
            BackgroundSystems
        }, 
        colors::{AlphaTranslation, BACKGROUND_COLOR, ColorTranslation}, 
        inheritance::BequeathTextColor,
        motion::{
            Bounce, 
            PointToPointTranslation
        }, physics::Velocity, 
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
		);
    }
}

fn setup_dilemma_decision_transition(
    dilemma : Res<Dilemma>,
    mut commands : Commands,
    systems: Res<BackgroundSystems>,
    mut background_query : Query<(&mut Background, &mut AlphaTranslation)>,
    mut train_query : Query<(Entity, &mut Velocity, &Transform), (With<Train>, Without<Junction>)>,
    mut junction_query: Query<(Entity, &Transform), (With<Junction>, Without<Train>)>,
    mut title_query: Query<(Entity, &mut ColorTranslation), Without<Background>>
) {

    	let decision_position = -70.0 * dilemma.stages[0].countdown_duration.as_secs_f32();
		let transition_duration = Duration::from_secs_f32(decision_position/ DilemmaScene::TRAIN_SPEED);
		let train_x_displacement = Vec3::new(decision_position, 0.0, 0.0);
		let final_position = Vec3::new(
			150.0 * dilemma.stages[0].countdown_duration.as_secs_f32(),
			0.0, 
			0.0
		);
		let main_track_translation_start: Vec3 = DilemmaScene::MAIN_TRACK_TRANSLATION_END + final_position;
		let initial_color = match dilemma.stages[0].default_option {
			None => Color::WHITE,
			Some(ref option) => DilemmaScene::TRACK_COLORS[*option]
		};
    
    commands.spawn(
(
            StateScoped(GameState::Dilemma),
            Junction{
                dilemma : dilemma.clone()
            },
            TextColor(BACKGROUND_COLOR),
            ColorTranslation::new(
                initial_color,
                transition_duration,
                true
            ),
            Transform::from_translation(main_track_translation_start),
            PointToPointTranslation::new(
                DilemmaScene::MAIN_TRACK_TRANSLATION_END,
                transition_duration,
                false
            )
        )
    );

    let mut train_velocity = Vec3::ZERO;
    let mut train_transform = Transform::default();
    for (entity, mut velocity, transform) in train_query.iter_mut() {
        train_velocity = velocity.0;
        train_transform = transform.clone();
        velocity.0 = Vec3::ZERO;
        commands.entity(entity).insert(PointToPointTranslation::new(
            DilemmaScene::TRAIN_INITIAL_POSITION + train_x_displacement,
            transition_duration,
            false
        ));
    }
    for (entity, transform) in junction_query.iter_mut() {

        let displacement = transform.translation - train_transform.translation;

        commands.entity(entity).insert(
            PointToPointTranslation::new(
                DilemmaScene::TRAIN_INITIAL_POSITION + train_x_displacement + displacement + Vec3::ZERO.with_x(-train_velocity.x * transition_duration.as_secs_f32()),
                transition_duration,
                false
            )
        );
    }
    for (entity, mut color) in title_query.iter_mut() {
        commands.entity(entity).insert(BequeathTextColor);
        commands.entity(entity).remove::<Bounce>();

        color.start()
    }
    for (mut background, mut color) in background_query.iter_mut() {
        color.start();
        background.speed = -dilemma.stages[0].countdown_duration.as_secs_f32() / 5.0;
        commands.run_system(systems.0["update_background_speeds"]);
    }
}

fn end_dilemma_decision_transition(
    dilemma: Res<Dilemma>,
    mut commands : Commands,
    systems: Res<BackgroundSystems>,
    mut next_sub_state: ResMut<NextState<DilemmaPhase>>,
    mut train_query : Query<&mut PointToPointTranslation, (With<Train>, Without<Junction>)>,
    mut junction_query: Query<&mut PointToPointTranslation, (With<Junction>, Without<Train>)>,
    mut background_query : Query<&mut Background>
) {

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
            let initial_position = translation.initial_position;
            translation.initial_position = translation.final_position;
            translation.final_position = initial_position + Vec3::new(-50.0, 0.0, 0.0);
            translation.timer = Timer::new(
                dilemma.stages[0].countdown_duration,
                TimerMode::Once
            );
        }
    }
}