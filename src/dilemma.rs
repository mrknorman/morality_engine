
use std::{fs::File, io::BufReader, path::PathBuf};
use bevy::{prelude::*, sprite::Anchor, text::{BreakLineOn, Text2dBounds}, transform};
use serde::{Deserialize, Serialize};

use crate::{audio::play_sound_once, train::{Train, TrainEntities, TrainText, TrainTrack}};
use crate::narration::Narration;

#[derive(PartialEq)]
enum LeverState{
	Left,
	Right,
	Random
}
#[derive(Resource)]
pub struct Lever{
	state : LeverState,
	current_audio : Option<Entity>
}

#[derive(Component)]
pub struct LeverTransform{
	initial : Vec3,
	left : Vec3,
	right : Vec3,
	random : Vec3
}

#[derive(Resource)]
struct DilemmaEntities{
	train_entity : TrainEntities,
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

pub struct DilemmaOptionConsequences {
    total_fatalities : usize
}

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
				transform: Transform::from_xyz(-500.0,250.0, 1.0),
				text_anchor : Anchor::TopLeft,
				..default()
			})
		).id()
	}
}

pub fn setup_dilemma(
		mut commands : Commands,
		asset_server: Res<AssetServer>
	) {
		
	let train_text = TrainText::new(false, 0);
	let train: Train = Train::new(
		None,
		train_text.train_engine_text,
		train_text.carridge_text_vector,
		train_text.smoke_text_frames,
		Vec3::new(-500.0, -75.0, 1.0)
	);
	let train_entity : crate::train::TrainEntities = train.spawn(&mut commands);

	let dilemma : Dilemma = Dilemma::load(
		PathBuf::from("./dilemmas/lab_1.json")
	);
	let dilemma_entity : Entity = DilemmaInfoPanel::spawn(&mut commands, &dilemma);

	let narration_audio_entity : Entity = commands.spawn((
        Narration {
            timer: Timer::from_seconds(2.0, TimerMode::Once)
        },
        AudioBundle {
			source: asset_server.load(PathBuf::from("sounds/dilemma_narration/lab_1.ogg")),
			settings : PlaybackSettings {
				paused : true,
				volume : bevy::audio::Volume::new(1.0),
				mode:  bevy::audio::PlaybackMode::Remove,
				..default()
			}
		})).id();

	commands.insert_resource(DilemmaEntities{
		train_entity,
		dilemma_entity,
		narration_audio_entity
	});

	let start_state = match dilemma.default_option {
		None => LeverState::Random,
		Some(ref option) if *option == 1 => LeverState::Left,
		Some(_) => LeverState::Right,
	};

	commands.insert_resource(Lever{
		state : start_state,
		current_audio : None
	});
	
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

	commands.entity(id_1).insert(LeverTransform{
			initial : track_2_translation.clone(),
			left : Vec3{x: 0.0, y: 0.0, z: 0.0},
			right : Vec3{x: 0.0, y: -100.0, z: 0.0},
			random : Vec3{x: 0.0, y: -50.0, z: 0.0}
		}
	);
	commands.entity(id_2).insert(LeverTransform{
		initial : track_3_translation.clone(),
		left : Vec3{x: 0.0, y: 0.0, z: 0.0},
		right : Vec3{x: 0.0, y: -100.0, z: 0.0},
		random : Vec3{x: 0.0, y: -50.0, z: 0.0}
	}
);

}

pub fn check_level_pull(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    lever: Res<Lever>,
) {
    // Variables to hold new state and audio entity
    let mut new_state = None;
    let mut new_audio = None;

    // Determine the new state and despawn the current audio if necessary
    if keyboard_input.just_pressed(KeyCode::Digit1) && (lever.state != LeverState::Left) {
        new_state = Some(LeverState::Left);
        new_audio = Some(play_sound_once("sounds/switch.ogg", &mut commands, &asset_server));
    } else if keyboard_input.just_pressed(KeyCode::Digit2) && (lever.state != LeverState::Right) {
        new_state = Some(LeverState::Right);
        new_audio = Some(play_sound_once("sounds/switch.ogg", &mut commands, &asset_server));
    }
	
    // If there is a new state, update the Lever resource
    if let Some(state) = new_state {
        commands.insert_resource(Lever {
            state,
            current_audio: new_audio,
        });
    }
}

pub fn lever_motion(
    mut movement_query: Query<(&mut LeverTransform, &mut Transform)>,
    lever: Res<Lever>,
    time: Res<Time>,  // Add time resource to manage frame delta time
) {
    for (lever_transform, mut transform) in movement_query.iter_mut() {
        let right_position: Vec3 = lever_transform.initial + lever_transform.right;
        let left_position: Vec3 = lever_transform.initial + lever_transform.left;

        let distance_threshold = 0.01; // Small threshold to determine when to snap to the final position
        let proportional_speed_factor = 0.1; // Factor to adjust the proportional speed
        let bounce_amplitude = 0.02; // Amplitude of the bounce effect
        let bounce_frequency = 10.0; // Frequency of the bounce effect

        if lever.state == LeverState::Right {
            let distance = (right_position - transform.translation).length();
            if distance > distance_threshold {
                let direction = (right_position - transform.translation).normalize();
                let movement_speed = distance * proportional_speed_factor;
                transform.translation += direction * movement_speed;
            } else {
                let bounce_offset = bounce_amplitude * (time.elapsed_seconds() * bounce_frequency).sin() as f32;
                transform.translation = right_position + Vec3::new(bounce_offset, 0.0, 0.0);
            }
        } else if lever.state == LeverState::Left {
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
}