use bevy::{
	prelude::*,
	sprite::Anchor,
};
use crate::{
	audio::{
		DilatableAudio, 
		TransientAudio, 
		TransientAudioPallet
	}, 
	motion::Bounce, 
	physics::{
		Gravity, 
		PhysicsPlugin
	}, 
	text::TextSprite, 
	time::Dilation
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
				PersonSprite::alert,
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

	pub fn alert(
		mut query: Query<(
			Entity,  
			&TransientAudioPallet, 
			&mut PersonSprite, 
			&mut Bounce
		), With<PersonSprite>>,
		dilation : Res<Dilation>,
		mut commands : Commands,
		mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>
	) {
		for (entity,  pallet, person, mut bounce) in query.iter_mut() {
			if person.in_danger && bounce.timer.just_finished() {
				TransientAudioPallet::play_transient_audio(
					entity,
					&mut commands,
					pallet,
					"exclamation".to_string(),
					dilation.0,
					&mut audio_query
				);
			}
			bounce.active = person.in_danger;
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
		dilation : Res<Dilation>,
		person_query: Query<&mut Bounce, With<PersonSprite>>,
		mut emoticon_query: Query<(&Parent, &mut Emoticon, &mut Transform, &mut Text2d)>,
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
