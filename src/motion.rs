use bevy::prelude::*;

use rand::Rng;
use std::time::Duration;

#[derive(Component)]
pub struct PointToPointTranslation {
    start: Vec3,
    end: Vec3,
    speed: f32,
    has_started: bool,
    has_finished: bool
}

impl PointToPointTranslation {
    pub fn new(start: Vec3, end: Vec3, duration_seconds: f32) -> PointToPointTranslation {
        let distance: f32 = (end - start).length();
        let speed: f32 = distance / duration_seconds;

        PointToPointTranslation {
            start,
            end,
            speed,
            has_started: false,
            has_finished: false,
        }
    }

    pub fn set_duration(
            &mut self, 
            new_duration_seconds : f32
        ) {

        let distance: f32 = (self.end - self.start).length();
        self.speed =  distance / new_duration_seconds;
    }

    pub fn start(&mut self) {
        self.has_started = true;
    }
    
    pub fn end(&mut self) {
        if self.has_started {
            self.has_finished = true;
        } else {
            panic!("Cannot end motion that has not started!");
        }
    }
}

pub fn point_to_point_translations(
        time: Res<Time>, 
        mut query: Query<(&mut PointToPointTranslation, &mut Transform)>
    ) {

    for (mut motion, mut transform) in query.iter_mut() {
        if motion.has_started && !motion.has_finished {
            let direction = (motion.end - motion.start).normalize();
            let distance_to_travel = motion.speed * time.delta_secs();
            let current_position = transform.translation;
            
            let distance_to_end = (motion.end - current_position).length();
            
            if distance_to_travel >= distance_to_end {
                transform.translation = motion.end;
                motion.has_finished = true;
            } else {
                transform.translation += direction * distance_to_travel;
            }
        }
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