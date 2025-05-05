use bevy::{
	prelude::*, render::primitives::Aabb, sprite::Anchor
};
use enum_map::Enum;
use crate::{
	systems::{
		audio::{
			DilatableAudio, 
			TransientAudio, 
			TransientAudioPallet,
		},
		motion::Bounce,
		physics::{
			Gravity, 
			PhysicsPlugin
		},
		time::Dilation
	},		
	entities::text::TextSprite,
	data::states::DilemmaPhase,  
};

use super::{text::TextTitle, train::{Train, TrainCarriage}}; 

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
				PersonSprite::scream,
				PersonSprite::alert,
				PersonSprite::explode,
				Emoticon::animate
            )
            .run_if(in_state(PersonSystemsActive::True))
			.run_if(
				in_state(DilemmaPhase::Decision).or(in_state(DilemmaPhase::Consequence))
			)
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


#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmotionSounds {
	Exclaim
}

const PERSON : &str = " @ \n/|\\\n/ \\";
const PERSON_IN_DANGER : &str= "\\@/\n | \n/ \\";

const EXCLAMATION : &str = "!";
const NEUTRAL : &str = "    ";

fn default_person_anchor() -> Anchor {
	Anchor::BottomCenter
}

fn default_person() -> Text2d {
	Text2d::new(PERSON)
}

#[derive(Component)]
#[require(Anchor = default_person_anchor(), Gravity, TextSprite, Text2d = default_person())]
pub struct PersonSprite{
	pub in_danger : bool,
}

impl Default for PersonSprite {
	fn default() -> Self {
		PersonSprite {
			in_danger : false,
		}
	}
}

impl PersonSprite {
	pub fn animate(
		mut query: Query<(
			&mut Text2d, 
			&mut Bounce,
		), With<PersonSprite>>,
	) {
		for (mut text, bounce) in query.iter_mut() {

			if bounce.enacting {
				text.0 = String::from(PERSON_IN_DANGER);
			} else {
				text.0 = String::from(PERSON);
			}
		}
	}

	pub fn scream(
		mut query: Query<(
			Entity,  
			&TransientAudioPallet<EmotionSounds>, 
			&mut PersonSprite, 
			&mut Bounce
		), With<PersonSprite>>,
		dilation : Res<Dilation>,
		mut commands : Commands,
		mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>
	) {
		for (entity,  pallet, person, bounce) in query.iter_mut() {
			if person.in_danger && bounce.timer.just_finished() {
				TransientAudioPallet::<EmotionSounds>::play_transient_audio(
					entity,
					&mut commands,
					pallet,
					EmotionSounds::Exclaim,
					dilation.0,
					&mut audio_query
				);
			}
		}
	}

	pub fn alert(
		mut query: Query<(
			&mut PersonSprite, 
			&mut Bounce
		)>
	) {
		for (person, mut bounce) in query.iter_mut() {
			bounce.active = person.in_danger;
		}
	}

	pub fn explode(
		person_query: Query<(
			Entity,
			&Aabb,
			&GlobalTransform
		), (With<PersonSprite>, Without<Train>)>,
		train_query: Query<(
			&Aabb,
			&GlobalTransform
		), (With<TrainCarriage>, Without<PersonSprite>)>,
		mut commands: Commands,
	) {
		// Iterate through all person sprites
		for (person_entity, person_aabb, person_transform) in person_query.iter() {
			// Get person AABB in world space
			let person_world_min = person_transform.transform_point(Vec3::from(person_aabb.center - person_aabb.half_extents));
			let person_world_max = person_transform.transform_point(Vec3::from(person_aabb.center + person_aabb.half_extents));
			
			// For each person, check collision with all trains
			for (train_aabb, train_transform) in train_query.iter() {
				// Get train AABB in world space
				let train_world_min = train_transform.transform_point(Vec3::from(train_aabb.center - train_aabb.half_extents));
				let train_world_max = train_transform.transform_point(Vec3::from(train_aabb.center + train_aabb.half_extents));
				
				// Simple AABB overlap check in world space
				if person_world_min.x <= train_world_max.x && 
				   person_world_max.x >= train_world_min.x && 
				   person_world_min.y <= train_world_max.y && 
				   person_world_max.y >= train_world_min.y {
					// If there's a collision, despawn the person entity
					commands.entity(person_entity).despawn();
					break;
				}
			}
		}
	}
}

fn default_emoticon() -> Text2d {
	Text2d::new(NEUTRAL)
}

fn default_emoticon_transform() -> Transform {
	Transform::from_xyz(0.0, 50.0, 0.0)
}

#[derive(Component)]
#[require(TextSprite,  Text2d = default_emoticon(), Transform = default_emoticon_transform())]
pub struct Emoticon{
	pub initial_size : f32,
	pub current_size : f32,
	pub translation : Vec3
}

impl Default for Emoticon {
	fn default() -> Self {
		Self{
			initial_size : 1.0,
			current_size : 1.0,
			translation : Vec3{x: 0.0, y: 50.0, z:0.0}
		}	
	}
}

impl Emoticon {
	pub fn animate(
		time: Res<Time>,
		dilation : Res<Dilation>,
		person_query: Query<&mut Bounce, With<PersonSprite>>,
		mut emoticon_query: Query<(&ChildOf, &mut Emoticon, &mut Transform, &mut Text2d)>,
	) {

		let duration_seconds = time.delta_secs()*dilation.0;
		for (parent, mut sprite, mut transform, mut text) in emoticon_query.iter_mut() {
			if let Ok(animation) = person_query.get(parent.get()) {
				if animation.enacting {
					sprite.current_size += duration_seconds*2.0;
					transform.translation.y += duration_seconds*15.0;
					transform.scale = Vec3::new(
						sprite.current_size, 
						sprite.current_size, 
						1.0
					);
					text.0 = String::from(EXCLAMATION);
				} else {
					sprite.current_size = sprite.initial_size;
					transform.translation.y = sprite.translation.y;
					transform.scale = Vec3::ONE;
					text.0 = String::from(NEUTRAL);
				}
			}
		}
	}
}
