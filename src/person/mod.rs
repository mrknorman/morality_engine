use std::time::Duration;
use rand::Rng;

use bevy::{
	prelude::*,
	sprite::Anchor, transform,
};
use crate::{
	text::TextSprite,
	colors::{PRIMARY_COLOR, DANGER_COLOR},
	audio::{TransientAudioPallet, TransientAudio},
	physics::{Gravity, Velocity, PhysicsPlugin}
};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PersonSystemsActive {
    #[default]
	False,
    True
}

pub struct PersonPlugin;

impl Plugin for PersonPlugin {
    fn build(&self, app: &mut App) {	
		app
		.init_state::<PersonSystemsActive>()
		.add_systems(
			Update,
			activate_systems
		).add_systems(
            Update,
            (
				PersonSprite::animate,
				Emoticon::animate
            )
            .run_if(in_state(PersonSystemsActive::True))
        );

		if !app.is_plugin_added::<PhysicsPlugin>() {
			app.add_plugins(PhysicsPlugin);
		}
    }
}

fn activate_systems(
	mut person_state: ResMut<NextState<PersonSystemsActive>>,
	person_query: Query<&PersonSprite>
) {
	if !person_query.is_empty() {
		person_state.set(PersonSystemsActive::True)
	} else {
		person_state.set(PersonSystemsActive::False)
	}
}

const PERSON : &str = " @ \n/|\\\n/ \\";
const PERSON_IN_DANGER : &str= "\\@/\n | \n/ \\";

const EXCLAMATION : &str = "!";
const NEUTRAL : &str = "    ";

pub const ACCELERATION_DUE_TO_GRAVITY : f32 = -200.0;
const ANIMATION_COOLDOWN_MIN_SECONDS : f32 = 1.0;
const ANIMATION_COOLDOWN_MAX_SECONDS : f32 = 2.0;

#[derive(Component)]
pub struct Bounce {
	pub bouncing : bool,
	pub is_mid_bounce : bool,
	pub initial_velocity_min : f32,
	pub initial_velocity_max : f32,
	pub interval_min : f32,
	pub interval_max : f32,
	timer : Timer
}

impl Bounce {
	pub fn new(
		initial_velocity_min : f32,
		initial_velocity_max : f32,
		interval_min : f32,
		interval_max : f32
	) -> Bounce {

		Bounce {
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
}

fn default_person_anchor() -> Anchor {
	Anchor::BottomCenter
}

fn default_person() -> Text2d {
	Text2d::new(PERSON)
}

#[derive(Component)]
#[require(Anchor(default_person_anchor), Gravity, TextSprite, Text2d(default_person))]
pub struct PersonSprite{
	pub in_danger : bool,
	pub animaton_interval_timer : Timer
}

impl Default for PersonSprite {
	fn default() -> Self {
		PersonSprite {
			in_danger : false,
			animaton_interval_timer: Timer::from_seconds(
				rand::random::<f32>(),
				TimerMode::Repeating
			) 
		}
	}
}

impl PersonSprite {

	pub fn bounce(
		time: Res<Time>,
		mut query: Query<(
			&mut Bounce,
			&mut Velocity,
			&Gravity
		)>
	) {
		let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
		for (mut bounce, mut velocity, gravity) in query.iter_mut() {

			if !bounce.is_mid_bounce {
				bounce.timer.tick(time.delta());
			}

			if bounce.is_mid_bounce {

				let interval_max = bounce.interval_max;
				let interval_min  = bounce.interval_max;

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
				if bounce.timer.finished() && !gravity.is_falling && bounce.bouncing {
					bounce.is_mid_bounce = true;
					velocity.0 = Vec3::new(0.0, rng.gen_range(bounce.initial_velocity_min..bounce.initial_velocity_max), 0.0);
				}
			}
		}
	}

	pub fn animate(
		time: Res<Time>,
		mut query: Query<(
			Entity,  
			&mut Text2d, 
			&TransientAudioPallet, 
			&mut TextColor,
			&mut Transform, 
			&mut PersonSprite, 
			&mut Bounce,
			&mut Velocity,
			&Gravity
		)>,
		mut commands : Commands,
		mut audio_query: Query<&mut TransientAudio>,
	) {
		let mut rng: rand::prelude::ThreadRng = rand::thread_rng();

		for (entity, mut text, pallet, mut color,  mut transform, mut person, mut animation, mut velocity, gravity) in query.iter_mut() {
		
			if person.in_danger && !animation.is_mid_bounce {
				person.animaton_interval_timer.tick(time.delta());
			}

			if animation.is_mid_bounce {
				text.0 = String::from(PERSON_IN_DANGER);
				transform.scale = Vec3::ONE;
				color.0 = DANGER_COLOR;

				if !gravity.is_falling {
					animation.is_mid_bounce = false;
					person.animaton_interval_timer.set_duration(
						Duration::from_secs_f32(rand::random::<f32>()
						* (ANIMATION_COOLDOWN_MAX_SECONDS - ANIMATION_COOLDOWN_MIN_SECONDS) 
						+ ANIMATION_COOLDOWN_MIN_SECONDS)
					);
					person.animaton_interval_timer.reset();
				}
			} else {
				text.0 = String::from(PERSON);
				color.0 = PRIMARY_COLOR;
				transform.scale = Vec3::ONE;
				if person.animaton_interval_timer.finished() && !gravity.is_falling && person.in_danger {
					animation.is_mid_bounce = true;
					velocity.0 = Vec3::new(0.0, rng.gen_range(animation.initial_velocity_min..animation.initial_velocity_max), 0.0);
					TransientAudioPallet::play_transient_audio(
                        entity,
                        &mut commands,
                        pallet,
                        "exclamation".to_string(),
                        &mut audio_query
                    );
				}
			}
		}
	}
}
pub enum EmotionState{
	Neutral,
	Afraid
}

fn default_emoticon() -> Text2d {
	Text2d::new(NEUTRAL)
}

fn default_emoticon_transform() -> Transform {
	Transform::from_xyz(0.0, 50.0, 0.0)
}

#[derive(Component)]
#[require(TextSprite,  Text2d(default_emoticon), Transform(default_emoticon_transform))]
pub struct Emoticon{
	pub state : EmotionState,
	pub initial_size : f32,
	pub current_size : f32,
	pub translation : Vec3
}

impl Default for Emoticon {
	fn default() -> Self {
		Self{
			state : EmotionState::Neutral,
			initial_size : 1.0,
			current_size : 1.0,
			translation : Vec3{x: 0.0, y: 50.0, z:0.0}
		}	
	}
}

impl Emoticon {
	pub fn animate(
		time: Res<Time>,
		person_query: Query<&mut Bounce>,
		mut emoticon_query: Query<(&Parent, &mut Emoticon, &mut Transform, &mut Text2d, &mut TextColor)>,
	) {

		let duration_seconds = time.delta().as_secs_f32();
		for (parent, mut sprite, mut transform, mut text, mut color) in emoticon_query.iter_mut() {
			if let Ok(animation) = person_query.get(parent.get()) {
				if animation.is_mid_bounce {
					sprite.current_size += duration_seconds*2.0;
					transform.translation.y += duration_seconds*15.0;
					transform.scale = Vec3::new(
						sprite.current_size, 
						sprite.current_size, 
						1.0
					);
					text.0 = String::from(EXCLAMATION);
					color.0 = DANGER_COLOR;
				} else {
					sprite.current_size = sprite.initial_size;
					transform.translation.y = sprite.translation.y;
					transform.scale = Vec3::ONE;
					color.0 = PRIMARY_COLOR;
					text.0 = String::from(NEUTRAL);
				}
			}
		}
	}
}
