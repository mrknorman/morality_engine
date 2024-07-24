use bevy::prelude::*;

pub const PERSON : &str = "\t @ \n\t/|\\\n\t/ \\";
pub const PERSON_IN_DANGER : &str= "\t\\@/\n\t | \n\t/ \\";

pub const EXCLAIMATION : &str = "\t ! ";
pub const NEUTRAL : &str = "\t   ";

#[derive(Component)]
pub struct BounceAnimation {
	pub playing : bool,
	pub initial_position : Vec3,
	pub initial_velocity_min : f32,
	pub initial_velocity_max : f32,
	pub current_velocity : f32
}

impl BounceAnimation {
	pub fn new(
		initial_velocity_min : f32,
		initial_velocity_max : f32
	) -> BounceAnimation {

		BounceAnimation {
			playing : false,
			initial_position : Vec3::new(0.0, 0.0, 0.0),
			initial_velocity_min,
			initial_velocity_max,
			current_velocity : 0.0 
		}
	}
}

#[derive(Component)]
pub struct PersonSprite{
	pub in_danger : bool,
	pub animaton_interval_timer : Timer
}

impl PersonSprite {
	pub fn new() -> PersonSprite {
		PersonSprite {
			in_danger : false,
			animaton_interval_timer: Timer::from_seconds(
				rand::random::<f32>() + 0.5,
				TimerMode::Repeating
			) 
		}
	}
}

pub enum EmotionState{
	Neutral,
	Afraid
}

#[derive(Component)]
pub struct EmoticonSprite{
	pub state : EmotionState,
	pub initial_size : f32,
	pub current_size : f32,
	pub translation : Vec3
}

impl EmoticonSprite {

	pub fn new() -> EmoticonSprite {
		EmoticonSprite{
			state : EmotionState::Neutral,
			initial_size : 12.0,
			current_size : 12.0,
			translation : Vec3{x: 0.0, y: 42.0, z:0.0}
		}
	}

	pub fn spawn_with_parent(self, parent: &mut ChildBuilder<'_>) -> Entity {
		let text: &str = NEUTRAL;

		let translation = self.translation;

		parent.spawn((
			self,
			Text2dBundle {
				text : Text {
					sections : vec!(
						TextSection::new(text, TextStyle {
							font_size : 12.0,
							..default()
						})
					),
					justify : JustifyText::Center, 
					..default()
				},
				transform: Transform::from_translation(translation),
				..default()
			})).id()
	}
}