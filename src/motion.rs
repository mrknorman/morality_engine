use bevy::{image::TranscodeFormat, prelude::*};

use crate::physics::{
    Velocity,
    Gravity,
    PhysicsPlugin
};

use rand::Rng;
use std::time::Duration;

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MotionSystemsActive {
    #[default]
	False,
    True
}

pub struct MotionPlugin;

impl Plugin for MotionPlugin {
    fn build(&self, app: &mut App) {	
		app
		.init_state::<MotionSystemsActive>()
		.add_systems(
			Update,
			activate_systems
		).add_systems(
            Update,
            (
				Wobble::wobble,
				Bouncy::bounce,
				Locomotion::locomote,
                PointToPointTranslation::translate
            )
            .run_if(in_state(MotionSystemsActive::True))
        );

		if !app.is_plugin_added::<PhysicsPlugin>() {
			app.add_plugins(PhysicsPlugin);
		}
    }
}

fn activate_systems(
	mut state: ResMut<NextState<MotionSystemsActive>>,
	query: Query<(Option<&Bouncy>, Option<&Wobble>, Option<&Locomotion>, Option<&PointToPointTranslation>)>
) {
	if !query.is_empty() {
		state.set(MotionSystemsActive::True)
	} else {
		state.set(MotionSystemsActive::False)
	}
}

#[derive(Component)]
pub struct PointToPointTranslation {
    initial_position: Vec3,
    final_position: Vec3,
	timer : Timer
}

impl PointToPointTranslation {
    pub fn new(initial_position: Vec3, final_position: Vec3, duration: Duration) -> PointToPointTranslation {
		let mut translation = PointToPointTranslation {
			initial_position,
			final_position,
			timer : Timer::new(
				duration,
				TimerMode::Once
			)
		};

		translation.timer.pause();
		translation
	}

	pub fn translate(
		time: Res<Time>, 
		mut query: Query<(&mut PointToPointTranslation, &mut Transform)>
	) {
		for (mut motion, mut transform) in query.iter_mut() {

			motion.timer.tick(time.delta());

			if !motion.timer.paused() {

				let fraction_complete = motion.timer.fraction();
				let difference = motion.final_position - motion.initial_position;

				transform.translation = motion.initial_position + difference*fraction_complete;
			}
		}
	}

	pub fn start(&mut self) {
		self.timer.unpause();
	}
}



#[derive(Component, Clone)]
pub struct TranslationAnchor{
	pub translation : Vec3
}

impl Default for TranslationAnchor {
    fn default() -> Self {
        TranslationAnchor {
            translation : Vec3::default()
        }
    }
}

impl TranslationAnchor {

	pub fn new(translation : Vec3) -> TranslationAnchor {
		TranslationAnchor {
			translation
		}
	}
}

#[derive(Component, Clone)]
pub struct Wobble{
	pub timer: Timer
}

impl Default for Wobble {
    fn default() -> Self {
        Wobble::new()
    }
}

impl Wobble {
	pub fn new() -> Wobble {
		Wobble{
			timer : Timer::new(
				Duration::from_millis(100), 
				TimerMode::Repeating
			)
		}
	}

	pub fn wobble(
		time: Res<Time>, // Inject the Time resource to access the game time
		mut wobble_query: Query<(&mut Transform, &mut Wobble, &TranslationAnchor)>
	) {

		let mut rng = rand::thread_rng(); // Random number generator

		for (mut transform, mut wobble, translation_anchor) in wobble_query.iter_mut() {
			if wobble.timer.tick(time.delta()).finished() {
				// Calculate offset using sine and cosine functions for smooth oscillation
				let dx = rng.gen_range(-1.0..=1.0);
				let dy = rng.gen_range(-1.0..=1.0);  

				// Apply the calculated offsets to the child's position
				transform.translation.x = translation_anchor.translation.x + dx as f32;
				transform.translation.y = translation_anchor.translation.y + dy as f32;
			}
		}
	}
}

#[derive(Component)]
pub struct Bouncy {
	pub bouncing : bool,
	pub is_mid_bounce : bool,
	pub initial_velocity_min : f32,
	pub initial_velocity_max : f32,
	pub interval_min : f32,
	pub interval_max : f32,
	pub timer : Timer
}

impl Bouncy {
	pub fn new(
		initial_velocity_min : f32,
		initial_velocity_max : f32,
		interval_min : f32,
		interval_max : f32
	) -> Bouncy {

		Bouncy {
			bouncing : false,
			is_mid_bounce : false,
			initial_velocity_min,
			initial_velocity_max,
			interval_min,
			interval_max,
			timer : Timer::from_seconds(
				rand::random::<f32>() + 0.5,
				TimerMode::Repeating
			) 
		}
	}

    pub fn bounce(
		time: Res<Time>,
		mut query: Query<(
			&mut Bouncy,
			&mut Velocity,
			&Gravity
		)>
	) {
		let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
		for (mut bounce, mut velocity, gravity) in query.iter_mut() {

			if bounce.is_mid_bounce || !bounce.bouncing {
				bounce.timer.pause();
			} else {
				bounce.timer.unpause();
			}

			bounce.timer.tick(time.delta());

			if bounce.is_mid_bounce {
				let interval_max = bounce.interval_max;
				let interval_min  = bounce.interval_min;

				if !gravity.is_falling {
					bounce.is_mid_bounce = false;
					bounce.timer.set_duration(
						Duration::from_secs_f32(rand::random::<f32>()
						* (interval_max - interval_min) 
						+ interval_min)
					);
					bounce.timer.reset();
				}
			} else {
				if bounce.timer.just_finished() && !gravity.is_falling && bounce.bouncing {
					bounce.is_mid_bounce = true;
					velocity.0 = Vec3::new(0.0, rng.gen_range(bounce.initial_velocity_min..bounce.initial_velocity_max), 0.0);
				}
			}
		}
	}
}

#[derive(Component, Clone)] 
pub struct Locomotion{
	pub speed: f32
} 

impl Locomotion{

	pub fn new(speed: f32) -> Locomotion  {
		Locomotion{
			speed
		}
	}

	pub fn locomote(
		time: Res<Time>,
		mut locomotion_query: Query<(&Locomotion, &mut Transform)>
	) {
		let time_seconds: f32 = time.delta().as_secs_f32(); // Current time in seconds

		for (locomotion, mut transform) in locomotion_query.iter_mut() {
			let dx = locomotion.speed*time_seconds;
			transform.translation.x += dx;
		}
	}
}