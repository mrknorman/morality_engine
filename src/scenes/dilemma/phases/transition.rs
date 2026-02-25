use bevy::prelude::*;

use crate::{
    data::states::{DilemmaPhase, GameState},
    entities::{person::BloodSprite, train::Train},
    scenes::dilemma::{
        content::DilemmaScene,
        dilemma::{CurrentDilemmaStageIndex, DilemmaStage},
        junction::Junction,
        lever::{Lever, LeverState},
        visuals::AmbientBackgroundElement,
    },
    systems::{
        backgrounds::{Background, BackgroundSystems},
        colors::{AlphaTranslation, ColorTranslation},
        motion::PointToPointTranslation,
        physics::{CameraVelocity, ExplodedGlyph, Velocity},
    },
};

pub struct DilemmaTransitionPlugin;
impl Plugin for DilemmaTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(DilemmaPhase::DilemmaTransition),
            (setup, update_viscera_speeds, update_lever).chain(),
        )
        .add_systems(
            Update,
            trigger_exit
                .run_if(in_state(GameState::Dilemma))
                .run_if(in_state(DilemmaPhase::DilemmaTransition)),
        )
        .add_systems(OnExit(DilemmaPhase::DilemmaTransition), stop_viscera);
    }
}

fn update_lever(
    stage: Res<DilemmaStage>,
    index: Res<CurrentDilemmaStageIndex>,
    mut lever: ResMut<Lever>,
) {
    let selected_option = if index.0 == 0 {
        stage.default_option
    } else {
        lever.selected_index().or(stage.default_option)
    };
    let next_state = LeverState::from_option_index(selected_option);
    lever.set_state_and_options(next_state, stage.options.len());
}

fn setup(
    stage: Res<DilemmaStage>,
    index: Res<CurrentDilemmaStageIndex>,
    lever: Res<Lever>,
    systems: Res<BackgroundSystems>,
    mut commands: Commands,
    mut background_query: Query<(Entity, &mut Background)>,
    mut train_query: Query<(Entity, &mut Velocity, &Transform), (With<Train>, Without<Junction>)>,
    mut junction_query: Query<(Entity, &Transform), (With<Junction>, Without<Train>)>,
) {
    let (transition_duration, train_x_displacement, main_track_translation_start, _) =
        DilemmaScene::generate_common_parameters(&stage);

    let option_value = if index.0 == 0 {
        stage.default_option
    } else {
        lever.selected_index().or(stage.default_option)
    };

    let initial_color = option_value.map_or(Color::WHITE, DilemmaScene::track_color_for_option);

    commands.spawn((
        DespawnOnExit(GameState::Dilemma),
        Junction {
            stage: stage.clone(),
        },
        ColorTranslation::new(initial_color, transition_duration, false),
        PointToPointTranslation::new(
            main_track_translation_start,
            DilemmaScene::MAIN_TRACK_TRANSLATION_END,
            transition_duration,
            false,
        ),
    ));

    let mut train_transform = Transform::default();
    let mut train_velocity = Vec3::ZERO;
    for (entity, mut velocity, transform) in train_query.iter_mut() {
        train_velocity = velocity.0;
        train_transform = *transform;
        velocity.0 = Vec3::ZERO;
        commands.entity(entity).insert(PointToPointTranslation::new(
            transform.translation,
            DilemmaScene::TRAIN_INITIAL_POSITION + train_x_displacement,
            transition_duration,
            false,
        ));
    }
    for (entity, transform) in junction_query.iter_mut() {
        let displacement = transform.translation - train_transform.translation;
        commands.entity(entity).insert(PointToPointTranslation::new(
            transform.translation,
            DilemmaScene::TRAIN_INITIAL_POSITION + train_x_displacement + displacement
                - Vec3::ZERO.with_x(train_velocity.x * transition_duration.as_secs_f32()),
            transition_duration,
            false,
        ));
    }

    for (entity, mut background) in background_query.iter_mut() {
        commands.entity(entity).insert(AlphaTranslation {
            initial_alpha: 0.0,
            final_alpha: 1.0,
            timer: Timer::new(transition_duration, TimerMode::Once),
        });
        background.speed = -stage.speed * stage.countdown_duration.as_secs_f32() / 350.0;
        commands.run_system(systems.update_speeds_system());
    }
}

fn trigger_exit(
    stage: Res<DilemmaStage>,
    systems: Res<BackgroundSystems>,
    mut commands: Commands,
    mut next_sub_state: ResMut<NextState<DilemmaPhase>>,
    mut train_query: Query<&mut PointToPointTranslation, (With<Train>, Without<Junction>)>,
    junction_query: Query<&PointToPointTranslation, (With<Junction>, Without<Train>)>,
    mut background_query: Query<&mut Background>,
) {
    // Train translation is the canonical phase clock. Junction translation
    // liveness can vary as old junction entities are spawned/despawned and
    // must not block progression.
    let mut has_train_translation = false;
    let mut all_train_translations_finished = true;
    for translation in train_query.iter_mut() {
        has_train_translation = true;
        all_train_translations_finished &= translation.timer.is_finished();
    }

    // Keep this read to make sure transition setup actually attached junction
    // motion; this is observability-only and intentionally non-blocking.
    let _junction_translation_count = junction_query.iter().count();

    if has_train_translation && all_train_translations_finished {
        for mut background in background_query.iter_mut() {
            background.speed = 0.0;
            commands.run_system(systems.update_speeds_system());
        }

        *next_sub_state = NextState::PendingIfNeq(DilemmaPhase::Decision);

        for mut translation in train_query.iter_mut() {
            translation.initial_position = translation.final_position;
            translation.final_position =
                DilemmaScene::TRAIN_INITIAL_POSITION + Vec3::new(-50.0, 0.0, 0.0);
            translation.timer = Timer::new(stage.countdown_duration, TimerMode::Once);
        }
    }
}

pub fn update_viscera_speeds(
    stage: Res<DilemmaStage>,
    mut sprites: Query<
        (&mut CameraVelocity, &Transform),
        (
            Or<(With<BloodSprite>, With<ExplodedGlyph>)>,
            Without<AmbientBackgroundElement>,
        ),
    >,
) {
    for (mut velocity, transform) in &mut sprites {
        let x_speed = transform.translation.z.powf(0.3)
            * -stage.speed
            * stage.countdown_duration.as_secs_f32()
            * 3.0;
        velocity.0.x = x_speed;
    }
}

pub fn stop_viscera(
    mut sprites: Query<
        &mut CameraVelocity,
        (
            Or<(With<BloodSprite>, With<ExplodedGlyph>)>,
            Without<AmbientBackgroundElement>,
        ),
    >,
) {
    for mut velocity in &mut sprites {
        velocity.0.x = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenes::dilemma::dilemma::DilemmaStage;
    use std::time::Duration;

    #[test]
    fn update_viscera_speeds_skips_ambient_background_elements() {
        let mut app = App::new();
        app.insert_resource(DilemmaStage {
            countdown_duration: Duration::from_secs_f32(10.0),
            options: vec![],
            default_option: None,
            speed: 70.0,
        });

        let ambient = app
            .world_mut()
            .spawn((
                BloodSprite(1),
                AmbientBackgroundElement,
                CameraVelocity(Vec3::new(12.0, 0.0, 0.0)),
                Transform::from_xyz(0.0, 0.0, 1.0),
            ))
            .id();

        let gameplay = app
            .world_mut()
            .spawn((
                BloodSprite(1),
                CameraVelocity(Vec3::new(4.0, 0.0, 0.0)),
                Transform::from_xyz(0.0, 0.0, 8.0),
            ))
            .id();

        app.add_systems(Update, update_viscera_speeds);
        app.update();

        let ambient_velocity = app
            .world()
            .get::<CameraVelocity>(ambient)
            .expect("ambient entity should still exist")
            .0
            .x;
        let gameplay_velocity = app
            .world()
            .get::<CameraVelocity>(gameplay)
            .expect("gameplay entity should still exist")
            .0
            .x;

        assert_eq!(ambient_velocity, 12.0);
        assert_ne!(gameplay_velocity, 4.0);
    }

    #[test]
    fn stop_viscera_skips_ambient_background_elements() {
        let mut app = App::new();

        let ambient = app
            .world_mut()
            .spawn((
                ExplodedGlyph,
                AmbientBackgroundElement,
                CameraVelocity(Vec3::new(9.0, 0.0, 0.0)),
            ))
            .id();

        let gameplay = app
            .world_mut()
            .spawn((ExplodedGlyph, CameraVelocity(Vec3::new(9.0, 0.0, 0.0))))
            .id();

        app.add_systems(Update, stop_viscera);
        app.update();

        let ambient_velocity = app
            .world()
            .get::<CameraVelocity>(ambient)
            .expect("ambient entity should still exist")
            .0
            .x;
        let gameplay_velocity = app
            .world()
            .get::<CameraVelocity>(gameplay)
            .expect("gameplay entity should still exist")
            .0
            .x;

        assert_eq!(ambient_velocity, 9.0);
        assert_eq!(gameplay_velocity, 0.0);
    }
}
