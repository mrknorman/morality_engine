use bevy::{
    prelude::*, 
    window::PrimaryWindow
};

use crate::{
    data::states::{
        DilemmaPhase, 
        GameState
    }, 
    entities::{
        person::BloodSprite, 
        train::Train
    }, 
    scenes::dilemma::{
        content::DilemmaScene, 
        dilemma::DilemmaStage, 
        junction::Junction, 
        lever::{
            Lever, 
            LeverState
        }
    }, systems::{
        backgrounds::{
            Background, 
            BackgroundSystems
        }, 
        colors::{
            AlphaTranslation, 
            ColorTranslation
        }, 
        motion:: PointToPointTranslation, 
        physics::{
            CameraVelocity, 
            ExplodedGlyph, 
            Velocity
        }
    }
};

pub struct DilemmaTransitionPlugin;
impl Plugin for DilemmaTransitionPlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(
			OnEnter(DilemmaPhase::DilemmaTransition), 
			(setup, update_viscera_speeds, update_lever).chain()
		)
		.add_systems(
			Update,
			trigger_exit
			.run_if(in_state(GameState::Dilemma))
			.run_if(in_state(DilemmaPhase::DilemmaTransition)),
		)
        .add_systems(
            OnExit(DilemmaPhase::DilemmaTransition),
            stop_viscera
        );
    }
}

fn update_lever(
        stage : Res<DilemmaStage>,
        mut lever : ResMut<Lever>
    ) {
        
    lever.0 = match stage.default_option {
        None => LeverState::Random,
        Some(ref option) if *option == 0 => LeverState::Left,
        Some(_) => LeverState::Right
    };
}

fn setup(
    stage : Res<DilemmaStage>,
    systems: Res<BackgroundSystems>,
    mut commands : Commands,
    mut background_query : Query<(Entity, &mut Background)>,
    mut train_query : Query<(Entity, &mut Velocity, &Transform), (With<Train>, Without<Junction>)>,
    mut junction_query: Query<(Entity, &Transform), (With<Junction>, Without<Train>)>,
) {         
    

    let (transition_duration, train_x_displacement, main_track_translation_start, initial_color) = DilemmaScene::generate_common_parameters(&stage);

    commands.spawn(
(
            StateScoped(GameState::Dilemma),
            Junction{
                stage : stage.clone()
            },
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
        background.speed = -stage.speed * stage.countdown_duration.as_secs_f32() / 350.0;
        commands.run_system(systems.0["update_background_speeds"]);
    }
}

fn trigger_exit(
    stage : Res<DilemmaStage>,
    systems: Res<BackgroundSystems>,
    mut commands : Commands,
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
            translation.initial_position = translation.final_position;
            translation.final_position = DilemmaScene::TRAIN_INITIAL_POSITION + Vec3::new(-50.0, 0.0, 0.0);
            translation.timer = Timer::new(
                stage.countdown_duration,
                TimerMode::Once
            );
        }
    }
}

pub fn update_viscera_speeds(
    stage : Res<DilemmaStage>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut sprites: Query<(&mut CameraVelocity, &Transform), Or<(With<BloodSprite>, With<ExplodedGlyph>)>>,
) {
    let screen_height = window.height();

    for (mut velocity, transform) in &mut sprites {            
        let distance_from_bottom = (screen_height - transform.translation.y).max(0.0);
        let x_speed = distance_from_bottom * -stage.speed * stage.countdown_duration.as_secs_f32() / 350.0;
        velocity.0.x = x_speed;
    }
}

pub fn stop_viscera(
    mut sprites: Query<&mut CameraVelocity, Or<(With<BloodSprite>, With<ExplodedGlyph>)>>,
) {
    for mut velocity in &mut sprites {            
        velocity.0.x = 0.0;
    }
}