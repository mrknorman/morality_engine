
use std::{fs::File, io::BufReader, path::PathBuf, time::Duration};
use bevy::{prelude::*, render::{render_resource::VertexAttribute, texture::TranscodeFormat}, sprite::Anchor, text::{BreakLineOn, Text2dBounds}};
use serde::{Deserialize, Serialize};
use rand::Rng;

use crate::{
	audio::{play_sound_once, BackgroundAudio}		, 
	train::{Train, TrainEntities, TrainText, TrainTrack}, 
	lever::{LeverState, Lever}
};

use crate::narration::Narration;

use crate::game_states::{SubState, MainState, GameState};
use crate::io_elements::spawn_text_button;

const PERSON : &str = "\t @ \n\t/|\\\n\t/ \\";
const PERSON_IN_DANGER : &str= "\t\\@/\n\t | \n\t/ \\";

const EXCLAIMATION : &str = "\t ! ";
const NEUTRAL : &str = "\t   ";


#[derive(Component)]
pub struct LeverTrackTransform{
	index : usize,
	initial : Vec3,
	left : Vec3,
	right : Vec3,
	random : Vec3
}

#[derive(Resource)]
pub struct DilemmaEntities{
	button_entity : Entity,
	train_entity : Option<TrainEntities>,
	dilemma_entity : Entity,
	narration_audio_entity : Entity
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Culpability {
    Uninvolved,
    Forced,
    Accidental,
    Negligent,
    Willing
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Gender {
    Male,
    Female,
    NonBinary,
    Other,
    None, 
    Unknown
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum EducationLevel {
    None,
    GSCE,
    ALevels,
    BachelorsDegree,
    MastersDegree,
    Doctorate,
    PostDoctorate, 
    Unknown
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Job {
    Unemployed,
    Student,
    Teacher,
    Actor,
    Banker,
    Baker,
    Cook,
    BarTender,
    SupermarketWorker,
    FireFighter,
    PoliceOfficer,
    Nurse,
    Doctor,
    Solider,
    Unknown
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DilemmaOptionLoader {
    name : String,
    description : String,
    humans : Option<Vec<Human>>,
    num_humans : Option<usize>
}

#[derive(Serialize, Deserialize)]
pub struct DilemmaLoader {
	index : String,
    name : String,
    description : String,
    countdown_duration_seconds : f64,
    options : Vec<DilemmaOptionLoader>,
    default_option : Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Human {
    fatality_probability : f64,
    culpability : Option<Culpability>,
    name : Option<String>,
    gender : Option<Gender>,
    age : Option<u64>,
    iq : Option<u64>, 
    highest_education :Option<EducationLevel>,
    occupation : Option<Job>
}

impl Human {

    pub fn display(&self) {

        print!("|");

        match &self.name {
            Some(_) => {print!(" Name : {:?} |", self.name.as_ref().unwrap())},
            None => {}
        }
        match &self.culpability {
            Some(_) => {print!(" Culpability : {:?} |", self.culpability.as_ref().unwrap())},
            None => {}
        }
        match &self.gender {
            Some(_) => {print!(" Gender : {:?} |", self.gender.as_ref().unwrap())},
            None => {}
        }
        match &self.age {
            Some(_) => {print!(" Age : {:?} |", self.age.as_ref().unwrap())},
            None => {}
        }
        match &self.occupation {
            Some(_) => {print!(" Occupation : {:?} |", self.occupation.as_ref().unwrap())},
            None => {}
        }
        match &self.highest_education {
            Some(_) => {print!(" Highest Education Level : {:?} |", self.highest_education.as_ref().unwrap())},
            None => {}
        }
        match &self.iq {
            Some(_) => {println!("IQ : {:?} |", self.iq.as_ref().unwrap())},
            None => {}
        }
        print!(" Fatality Probability: {}% |\n", self.fatality_probability*100.0);
    }

}

#[derive(Debug, Clone, Copy)]
pub struct DilemmaOptionConsequences {
    total_fatalities : usize
}

#[derive(Debug, Clone)]
pub struct DilemmaOption {
    name : String,
    description : String,
    humans : Vec<Human>,
    consequences : DilemmaOptionConsequences,
    num_humans : usize
}

impl DilemmaOption {

	fn from_loader(dilemma_option_loader : DilemmaOptionLoader) -> DilemmaOption {

		let humans = if dilemma_option_loader.humans.is_some() {
			dilemma_option_loader.humans.unwrap()} else
		{ vec![] };
			
		let num_humans = if dilemma_option_loader.num_humans.is_some() {
			dilemma_option_loader.num_humans.unwrap()
		} else { humans.len() };

		let consequences = DilemmaOptionConsequences {
			total_fatalities : num_humans
		};

		DilemmaOption {
			name : dilemma_option_loader.name,
			description : dilemma_option_loader.description,
			humans,
			consequences,
			num_humans
		}		

	}

}


#[derive(Component)]
pub struct DilemmaTimer {
	max_time_seconds : f32,
	timer : Timer
}

impl DilemmaTimer {

	pub fn spawn(
			commands : &mut Commands,
			max_time_seconds : f32
		) -> Entity {

		commands.spawn(
			(
			DilemmaTimer {
				max_time_seconds,
				timer : Timer::from_seconds(
					max_time_seconds,
					TimerMode::Once
				)
			},
			Text2dBundle {
				text : Text {
					sections : vec![
						TextSection::new(
							format!("{:.2}\n", max_time_seconds),
							TextStyle {
								font_size: 50.0,
								..default()
							}
						),
						TextSection::new(
							format!("seconds remaining.\n"),
							TextStyle {
								font_size: 12.0,
								..default()
							}
						)
					],
					justify : JustifyText::Center, 
					linebreak_behavior: BreakLineOn::WordBoundary
				},
				transform: Transform::from_xyz(0.0, -100.0, 1.0),
				text_anchor : Anchor::Center,
				..default()
			}
			)
		).id()
	}
}

#[derive(Resource)]
pub struct Dilemma {
	index : String,
    name : String,
    description : String,
    countdown_duration_seconds : f64,
    options : Vec<DilemmaOption>,
    default_option : Option<usize>,
}

impl Dilemma {
	pub fn load(file_path : PathBuf) -> Dilemma {
		let file = File::open(file_path).expect("Unable to open file");
        let reader = BufReader::new(file);
        
        let loaded_dilemma : DilemmaLoader = serde_json::from_reader(reader).unwrap();

		let options : Vec<DilemmaOption> = loaded_dilemma.options.iter().map(
			|option| DilemmaOption::from_loader(option.clone())
		).collect();

		Dilemma {
			index : loaded_dilemma.index,
			name : loaded_dilemma.name,
			description : loaded_dilemma.description,
			countdown_duration_seconds : loaded_dilemma.countdown_duration_seconds,
			options,
			default_option : loaded_dilemma.default_option
		}
	}
}

#[derive(Component)]
pub struct DilemmaInfoPanel;

impl DilemmaInfoPanel {
	pub fn spawn(
		commands : &mut Commands,
		dilemma : &Dilemma
	) -> Entity {

		let box_size = Vec2::new(500.0, 2000.0);

		commands.spawn(
			(
			DilemmaInfoPanel,
			Text2dBundle {
				text : Text {
					sections : vec![
						TextSection::new(
							format!("Dilemma {}: {}\n", dilemma.index, dilemma.name),
							TextStyle {
								font_size: 30.0,
								..default()
						}),
						TextSection::new(
							dilemma.description.clone(),
							TextStyle {
								font_size: 15.0,
								..default()
						}),

					],
					justify : JustifyText::Left, 
					linebreak_behavior: BreakLineOn::WordBoundary
				},
				text_2d_bounds : Text2dBounds {
					size: box_size,
				},
				transform: Transform::from_xyz(-600.0,300.0, 1.0),
				text_anchor : Anchor::TopLeft,
				..default()
			})
		).id()
	}
}

#[derive(Component)]
pub struct DilemmaOptionInfoPanel;

impl DilemmaOptionInfoPanel {
	pub fn spawn(
		commands : &mut Commands,
		option : &DilemmaOption,
		index : usize
	) -> Entity {

		let box_size = Vec2::new(500.0, 2000.0);

		let mut color = Color::WHITE;
		let mut x_transform : f32 = 0.0;

		if index == 0 {
			color = Color::VIOLET;
			x_transform = -600.0;
		} else if index == 1 {
			color = Color::TURQUOISE;
			x_transform = 150.0;
		}

		commands.spawn(
			(
			DilemmaOptionInfoPanel,
			Text2dBundle {
				text : Text {
					sections : vec![
						TextSection::new(
							format!(
								"Option {}: {} [Press {} to select]\n", 
								index, 
								option.name, 
								index
							),
							TextStyle {
								font_size: 20.0,
								color,
								..default()
						}),
						TextSection::new(
							option.description.clone(),
							TextStyle {
								font_size: 15.0,
								color,
								..default()
						}),

					],
					justify : JustifyText::Left, 
					linebreak_behavior: BreakLineOn::WordBoundary
				},
				text_2d_bounds : Text2dBounds {
					size: box_size,
				},
				transform: Transform::from_xyz(x_transform,-150.0, 1.0),
				text_anchor : Anchor::TopLeft,
				..default()
			})
		).id()
	}
}

#[derive(Component)]
pub struct BounceAnimation {
	playing : bool,
	initial_position : Vec3,
	initial_velocity_min : f32,
	initial_velocity_max : f32,
	current_velocity : f32
}

impl BounceAnimation {
	fn new(
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
	in_danger : bool,
	position : Vec3,
	animaton_interval_timer : Timer
}

impl PersonSprite {
	fn new(position: Vec3) -> PersonSprite {
		PersonSprite {
			in_danger : false,
			position,
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
	state : EmotionState,
	initial_size : f32,
	current_size : f32,
	translation : Vec3
}

impl EmoticonSprite {

	fn new() -> EmoticonSprite {
		EmoticonSprite{
			state : EmotionState::Neutral,
			initial_size : 12.0,
			current_size : 12.0,
			translation : Vec3{x: 0.0, y: 42.0, z:0.0}
		}
	}

	fn spawn_with_parent(self, parent: &mut ChildBuilder<'_>) -> Entity {
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

pub fn setup_dilemma(
		mut commands : Commands,
		asset_server: Res<AssetServer>
	) {

	let dilemma : Dilemma = Dilemma::load(
		PathBuf::from("./dilemmas/lab_1.json")
	);
	let dilemma_entity : Entity = DilemmaInfoPanel::spawn(
		&mut commands, &dilemma
	);

	let narration_audio_entity : Entity = commands.spawn((
        Narration {
            timer: Timer::from_seconds(1.0, TimerMode::Once)
        },
        AudioBundle {
			source: asset_server.load(
				PathBuf::from("sounds/dilemma_narration/lab_1.ogg")
			),
			settings : PlaybackSettings {
				paused : true,
				volume : bevy::audio::Volume::new(1.0),
				mode:  bevy::audio::PlaybackMode::Remove,
				..default()
			}
		})).id();

	let button_entity = spawn_text_button(
		"[Click here or Press Enter to Begin]",
		Some(MainState::InGame),
		Some(GameState::Dilemma),
		Some(SubState::Decision),
		2.0,
		&mut commands
	);

	commands.insert_resource(dilemma);
	commands.insert_resource(DilemmaEntities{
		button_entity,
		train_entity : None,
		dilemma_entity,
		narration_audio_entity
	});

}

pub fn setup_decision(
		mut commands : Commands,
		asset_server: Res<AssetServer>,
		dilemma: Res<Dilemma>,  // Add time resource to manage frame delta time
		entities : Res<DilemmaEntities>
	){

	commands.entity(entities.button_entity).despawn_recursive();
	commands.entity(entities.narration_audio_entity).despawn_recursive();

	DilemmaTimer::spawn(
		&mut commands, 
		dilemma.countdown_duration_seconds as f32
	);

	let train_audio = commands.spawn(AudioBundle {
		source: asset_server.load(
			PathBuf::from("./sounds/train_aproaching.ogg")
		),
		settings : PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(0.5),
			mode:  bevy::audio::PlaybackMode::Loop,
			..default()
	}}).id();

	let clock_audio: Entity = commands.spawn(AudioBundle {
		source: asset_server.load(PathBuf::from("./sounds/clock.ogg")),
		settings : PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(0.3),
			mode:  bevy::audio::PlaybackMode::Loop,
			..default()
	}}).id();

	let background_audio: Vec<Entity> = vec![train_audio, clock_audio];

    commands.insert_resource(BackgroundAudio{audio: background_audio});

	let train_text = TrainText::new(false, 0);
	let train: Train = Train::new(
		None,
		train_text.train_engine_text,
		train_text.carridge_text_vector,
		train_text.smoke_text_frames,
		Vec3::new(-350.0, -75.0, 1.0)
	);
	let train_entity : crate::train::TrainEntities = train.spawn(&mut commands);

	let start_state = match dilemma.default_option {
		None => LeverState::Random,
		Some(ref option) if *option == 1 => LeverState::Left,
		Some(_) => LeverState::Right,
	};

	Lever::spawn(
		Vec3::new(0.0, -200.0, 0.0), 
		start_state, 
		&mut commands
	);
	
	let track_1_translation = Vec3{x : -800.0, y : 0.0, z: 0.0};
	let track_1 = TrainTrack::new_from_length(
		300, 
		track_1_translation.clone()
	);

	let mut track_2_translation: Vec3 = Vec3{x : 1000.0, y : 100.0, z: 0.0};
	let track_2 = TrainTrack::new_from_length(
		300, 
		track_2_translation.clone()
	);
	track_2_translation.x += 110.0;
	track_2_translation.y -= 40.0;

	let mut track_3_translation = Vec3{x : 1000.0, y : 0.0, z: 0.0};
	let track_3 = TrainTrack::new_from_length(
		300, 
		track_3_translation.clone()
	);
	track_3_translation.x += 110.0;
	track_3_translation.y -= 40.0;

	track_1.spawn(&mut commands);
	let id_1: Entity = track_2.spawn(&mut commands);
	let id_2 : Entity = track_3.spawn(&mut commands);

	commands.entity(id_1).insert(LeverTrackTransform{
			index : 2,
			initial : track_2_translation.clone(),
			left : Vec3{x: 0.0, y: 0.0, z: 0.0},
			right : Vec3{x: 0.0, y: -100.0, z: 0.0},
			random : Vec3{x: 0.0, y: -50.0, z: 0.0}
		}
	);
	commands.entity(id_2).insert(LeverTrackTransform{
		index : 1, 
		initial : track_3_translation.clone(),
		left : Vec3{x: 0.0, y: 0.0, z: 0.0},
		right : Vec3{x: 0.0, y: -100.0, z: 0.0},
		random : Vec3{x: 0.0, y: -50.0, z: 0.0}
		}
	);

	let person = String::from(PERSON);


	for (index, option) in dilemma.options.clone().into_iter().enumerate() {
		DilemmaOptionInfoPanel::spawn(&mut commands, &option, index);
	}

	for _ in 0..dilemma.options[0].consequences.total_fatalities {
		commands.entity(id_2).with_children(|parent| {
				parent.spawn(
					Text2dBundle {
						text : Text {
							sections : vec![
								TextSection::new(
									person.clone(),
									TextStyle {
										font_size: 12.0,
										..default()
								})
							],
							justify : JustifyText::Left, 
							linebreak_behavior: BreakLineOn::WordBoundary
						},
						transform: Transform::from_xyz(-600.0,0.0, 0.0),
						text_anchor : Anchor::BottomCenter,
						..default()
					}
				);	
			}
		);
	}

	for _ in 0..dilemma.options[1].consequences.total_fatalities {

		let position: Vec3 = Vec3::new(-600.0, 0.0, 0.0);
		commands.entity(id_1).with_children(|parent| {
				parent.spawn(
					(Text2dBundle {
						text : Text {
							sections : vec![
								TextSection::new(
									person.clone(),
									TextStyle {
										font_size: 12.0,
										..default()
								})
							],
							justify : JustifyText::Left, 
							linebreak_behavior: BreakLineOn::WordBoundary
						},
						transform: Transform::from_translation(position),
						text_anchor : Anchor::BottomCenter,
						..default()
					},
					PersonSprite::new(position),
					BounceAnimation::new(40.0, 60.0)
					)
				).with_children(
					|parent| {
						EmoticonSprite::new().spawn_with_parent(parent);
					}
				);	
			}
		);
	}
}

pub fn lever_motion(
    mut movement_query: Query<(&mut LeverTrackTransform, &mut Transform)>,
    lever: Option<Res<Lever>>,
    time: Res<Time>,  // Add time resource to manage frame delta time
) {

	if lever.is_some() {

		let unwrapped_lever: Res<Lever> = lever.unwrap();

		for (lever_transform, mut transform) in movement_query.iter_mut() {
			let right_position: Vec3 = lever_transform.initial + lever_transform.right;
			let left_position: Vec3 = lever_transform.initial + lever_transform.left;

			let distance_threshold = 0.01; // Small threshold to determine when to snap to the final position
			let proportional_speed_factor = 0.1; // Factor to adjust the proportional speed
			let bounce_amplitude = 0.02; // Amplitude of the bounce effect
			let bounce_frequency = 10.0; // Frequency of the bounce effect

			if unwrapped_lever.state == LeverState::Right {
				let distance = (right_position - transform.translation).length();
				if distance > distance_threshold {
					let direction = (right_position - transform.translation).normalize();
					let movement_speed = distance * proportional_speed_factor;
					transform.translation += direction * movement_speed;
				} else {
					let bounce_offset = bounce_amplitude * (time.elapsed_seconds() * bounce_frequency).sin() as f32;
					transform.translation = right_position + Vec3::new(bounce_offset, 0.0, 0.0);
				}
			} else if unwrapped_lever.state == LeverState::Left {
				let distance = (left_position - transform.translation).length();
				if distance > distance_threshold {
					let direction = (left_position - transform.translation).normalize();
					let movement_speed = distance * proportional_speed_factor;
					transform.translation += direction * movement_speed;
				} else {
					let bounce_offset = bounce_amplitude * (time.elapsed_seconds() * bounce_frequency).sin() as f32;
					transform.translation = left_position + Vec3::new(bounce_offset, 0.0, 0.0);
				}
			}
		}
	} else {
		panic!("Lever motion check with non-existant lever!")
	}
}

pub fn person_check_danger(
    mut lever_query: Query<(&Children, &LeverTrackTransform)>,
    mut text_query: Query<&mut PersonSprite>,
    lever: Option<Res<Lever>>
) {

	if lever.is_some() {
		
		let unwrapped_lever: Res<Lever> = lever.unwrap();

		// Iterate through all lever queries
		for (children, lever_transform) in lever_query.iter_mut() {
			// Iterate through all children
			for &child in children.iter() {
				// Safely get the text and person associated with the child
				if let Ok(mut person) = text_query.get_mut(child) {

					let lever_state = &unwrapped_lever.state;

					if *lever_state != LeverState::Random {
						if lever_transform.index == unwrapped_lever.state.to_int() {
							person.in_danger = true;
						} else {
							person.in_danger = false;
						}
					}
				}
			}
		}
	} else {
		panic!("PersonSprite danger check with non-existant lever!")
	}
}

pub fn animate_person(
    time: Res<Time>,
    mut query: Query<(&mut Children, &mut Text, &mut Transform, &mut PersonSprite, &mut BounceAnimation)>,
	mut emoticon_query: Query<(&mut EmoticonSprite, &mut Transform, &mut Text), Without<PersonSprite>>
) {
    for (children, mut text, mut transform, mut person, mut animation) in query.iter_mut() {

		let emoticon_entity = children.iter().next();

        if person.in_danger {

			if animation.playing {

				if let Some(emoticon_entity) = emoticon_entity {
					if let Ok(mut query_result) = emoticon_query.get_mut(*emoticon_entity) {
						
						let duration_seconds = time.delta().as_millis() as f32 / 1000.0;

						query_result.0.current_size += 50.0*duration_seconds;
						let mut transform = query_result.1;
						let sprite = query_result.0;
						
						let mut text = query_result.2;

						transform.translation.y += 15.0*duration_seconds;

						text.sections[0] = TextSection::new(
							String::from(EXCLAIMATION),
							TextStyle {
								font_size: sprite.current_size,
								color : Color::RED,
								..default()
							}
						);
					}
				}
				
				text.sections[0] = TextSection::new(
					String::from(PERSON_IN_DANGER),
					TextStyle {
						font_size: 12.0,
						color : Color::RED,
						..default()
					}
				);

				if transform.translation.y >= animation.initial_position.y {
					let duration_seconds = time.delta().as_millis() as f32 / 1000.0;
					transform.translation.y += animation.current_velocity * duration_seconds;
					animation.current_velocity -= 9.81*20.0 * duration_seconds;
				} else {
					animation.playing = false;
					transform.translation.y = animation.initial_position.y;
					animation.current_velocity = 0.0;

					person.animaton_interval_timer.set_duration(
						Duration::from_secs_f32(rand::random::<f32>() / 2.0)
					);
					person.animaton_interval_timer.reset();

					if let Some(emoticon_entity) = emoticon_entity {
						if let Ok(mut query_result) = emoticon_query.get_mut(*emoticon_entity) {
	
							query_result.0.current_size = query_result.0.initial_size;
							let sprite = query_result.0;
							let mut transform = query_result.1;
							let mut text = query_result.2;

							transform.translation.y = sprite.translation.y;
	
							text.sections[0] = TextSection::new(
								String::from(NEUTRAL),
								TextStyle {
									font_size: sprite.initial_size,
									color : Color::WHITE,
									..default()
								}
							);
						}
					}
				}
			
			} else {

				if let Some(emoticon_entity) = emoticon_entity {
					if let Ok(mut query_result) = emoticon_query.get_mut(*emoticon_entity) {
						query_result.0.current_size = query_result.0.initial_size;
						let sprite = query_result.0;
						let mut transform: Mut<Transform> = query_result.1;
						let mut text = query_result.2;

						transform.translation.y = sprite.translation.y;

						text.sections[0] = TextSection::new(
							String::from(NEUTRAL),
							TextStyle {
								font_size: sprite.initial_size,
								color : Color::WHITE,
								..default()
							}
						);
					}
				}

				text.sections[0] = TextSection::new(
					String::from(PERSON),
					TextStyle {
						font_size: 12.0,
						..default()
					}
				);

				let mut rng = rand::thread_rng();

				person.animaton_interval_timer.tick(time.delta());
				if person.animaton_interval_timer.finished() {
					animation.playing = true;
					transform.translation.y = animation.initial_position.y;
					animation.current_velocity = rng.gen_range(animation.initial_velocity_min..animation.initial_velocity_max);
				}
			}
        }
		else {
			animation.playing = false;
			transform.translation.y = animation.initial_position.y;
			text.sections[0] = TextSection::new(
				String::from(PERSON),
				TextStyle {
					font_size: 12.0,
					..default()
				}
			);

			if let Some(emoticon_entity) = emoticon_entity {
				if let Ok(mut query_result) = emoticon_query.get_mut(*emoticon_entity) {

					query_result.0.current_size = query_result.0.initial_size;
					let sprite = query_result.0;
					let mut transform = query_result.1;
					let mut text = query_result.2;

					transform.translation.y = sprite.translation.y;

					text.sections[0] = TextSection::new(
						String::from(NEUTRAL),
						TextStyle {
							font_size: sprite.initial_size,
							color : Color::WHITE,
							..default()
						}
					);
				}
			}
		}
    }
}

pub fn update_timer(
		time: Res<Time>,
		mut timer_query: Query<(&mut DilemmaTimer, &mut Text)>
	) {
	
	for (mut timer, mut text) in timer_query.iter_mut() {

		timer.timer.tick(time.delta());

		let time_remaining = timer.timer.remaining_secs();

		text.sections[0] = TextSection::new(
			format!("{:.2}\n", time_remaining),
			TextStyle {
				font_size: 50.0,
				..default()
			}
		)
	}
}