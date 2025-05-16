use bevy::prelude::*;

use crate::{
    data::states::{
        DilemmaPhase, 
        GameState
    }, entities::train::Train, scenes::dilemma::{
        dilemma::Dilemma, 
        junction::Junction
    }, systems::{
        backgrounds::{
            Background, 
            BackgroundSystems
        }, 
        colors::{AlphaTranslation, ColorTranslation}, 
        inheritance::BequeathTextColor,
        motion::{
            Bounce, 
            PointToPointTranslation
        }, 
    }
};

pub struct DilemmaTransitionPlugin;
impl Plugin for DilemmaTransitionPlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(
			OnEnter(DilemmaPhase::IntroDecisionTransition), 
			setup_dilemma_transition
		)
		.add_systems(
			Update,
			end_dilemma_transition
			.run_if(in_state(GameState::Dilemma))
			.run_if(in_state(DilemmaPhase::IntroDecisionTransition)),
		);
    }
}

    fn setup_dilemma_transition(
            dilemma : Res<Dilemma>,
            mut commands : Commands,
            systems: Res<BackgroundSystems>,
            mut background_query : Query<(&mut Background, &mut AlphaTranslation)>,
            mut train_query : Query<&mut PointToPointTranslation, (With<Train>, Without<Junction>)>,
            mut junction_query: Query<&mut PointToPointTranslation, (With<Junction>, Without<Train>)>,
            mut title_query: Query<(Entity, &mut ColorTranslation), Without<Background>>
        ) {
        for mut train in train_query.iter_mut() {
            train.start();
        }
        for mut junction in junction_query.iter_mut() {
            junction.start()
        }
        for (entity, mut color) in title_query.iter_mut() {
            commands.entity(entity).insert(BequeathTextColor);
            commands.entity(entity).remove::<Bounce>();

            color.start()
        }
        for (mut background, mut color) in background_query.iter_mut() {
            color.start();
            background.speed = -dilemma.countdown_duration.as_secs_f32() / 5.0;
            commands.run_system(systems.0["update_background_speeds"]);
        }
}

    fn end_dilemma_transition(
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
                dilemma.countdown_duration,
                TimerMode::Once
            );
        }
    }
}