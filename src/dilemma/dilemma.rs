use std::{
	fs::File, 
	io::BufReader, 
	path::PathBuf, 
	iter::zip
};
use serde::{
	Deserialize, 
	Serialize
};


use bevy::{
	prelude::*, 
	sprite::Anchor, 
	ecs::component::StorageType,
	text::{
		LineBreak, 
		TextBounds
	}
};

use crate::{
	background::BackgroundSprite, game_states::DilemmaPhase, lever::{
		Lever, 
		LeverState, 
		OPTION_1_COLOR, 
		OPTION_2_COLOR
	}, motion::Locomotion, person::{
		BounceAnimation, Emoticon, PersonSprite,
	},  track::Track, train::Train, 
	audio::{
		TransientAudio, 
		TransientAudioPallet
	}
};

#[derive(Resource)]
pub struct DilemmaDashboard{
	timer : Entity,
	info  : Vec<Entity>,
	lever : Entity
}

impl DilemmaDashboard {
	pub fn spawn(
			commands : &mut Commands,
			dilemma: &Res<Dilemma>
		) {

		let timer : Entity = DilemmaTimer::spawn(
			commands, 
			dilemma.countdown_duration_seconds
		);

		let mut info : Vec<Entity> = vec![];
		for (index, option) in dilemma.options.clone().into_iter().enumerate() {
			info.push(
				DilemmaOptionInfoPanel::spawn(commands, &option, index)
			);
		}

		let start_state = match dilemma.default_option {
			None => LeverState::Random,
			Some(ref option) if *option == 0 => LeverState::Left,
			Some(_) => LeverState::Right,
		};

		let lever = Lever::spawn(
			Vec3::new(0.0, -200.0, 0.0), 
			start_state, 
			commands
		);

		let dashboard: DilemmaDashboard = DilemmaDashboard{
			timer,
			info,
			lever
		};
		commands.insert_resource(dashboard);
	}

	fn despawn(
		&self, 			
		commands : &mut Commands
	) {		
		commands.entity(self.timer).despawn_recursive();

		for entity in self.info.clone() {
			commands.entity(entity).despawn_recursive();
		}

		commands.entity(self.lever).despawn_recursive();
	}

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
    Gcse,
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
    countdown_duration_seconds : f32,
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
        println!(" Fatality Probability: {}% |", self.fatality_probability*100.0);
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
			dilemma_option_loader.humans.unwrap()
		} else { vec![] };
			
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
				timer : Timer::from_seconds(
					max_time_seconds,
					TimerMode::Once
				)
			},
			Text2d::new(format!("{:.2}\n", max_time_seconds)),
			TextFont {
				font_size: 50.0,
				..default()
			},
			TextLayout{
				justify : JustifyText::Center, 
				linebreak : LineBreak::WordBoundary
			},
			Transform::from_xyz(0.0, -100.0, 1.0),
			Anchor::Center
		)).with_children( | parent | {
			parent.spawn(
				(
					TextSpan::new(
						"seconds remaining.\n".to_string()
					), 
					TextFont {
						font_size: 12.0,
						..default()
					}
				)
			);
		}).id()
	}
}

#[derive(Resource, Clone)]
pub struct Dilemma {
	pub index : String,
    pub name : String,
    pub description : String,
    pub countdown_duration_seconds : f32,
    pub options : Vec<DilemmaOption>,
    pub default_option : Option<usize>,
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

#[derive(Component)]
pub struct DilemmaOptionInfoPanel;

impl DilemmaOptionInfoPanel {
	pub fn spawn(
		commands : &mut Commands,
		option : &DilemmaOption,
		index : usize
	) -> Entity {

		let mut color = Color::WHITE;
		let mut x_transform : f32 = 0.0;

		if index == 0 {
			color =OPTION_1_COLOR;
			x_transform = -600.0;
		} else if index == 1 {
			color = OPTION_2_COLOR;
			x_transform = 150.0;
		}

		commands.spawn(
			(
			DilemmaOptionInfoPanel,
			Text2d::new(
				format!(
					"Option {}: {} [Press {} to select]\n", 
					index + 1, 
					option.name, 
					index + 1
				)
			),
			TextColor(color),
			TextFont{
				font_size : 15.0,
				..default()
			},
			TextLayout{
				justify : JustifyText::Left,
				linebreak : LineBreak::WordBoundary
			},
			TextBounds{
				width : Some(500.0),
				height : Some(2000.0)
			}, 
			Transform::from_xyz(x_transform,-150.0, 1.0),
			Anchor::TopLeft
		)).with_children( | parent | {
			parent.spawn(
				(
					TextSpan::new(option.description.clone()),
					TextColor(color),
					TextFont{
						font_size : 15.0,
						..default()
					}
				)
			); 
		} ).id()
	}
}

#[derive(Resource)]
pub struct TransitionCounter{
	pub timer : Timer
}

pub fn end_transition(
	time : Res<Time>,
	mut counter: ResMut<TransitionCounter>,
	mut next_sub_state: ResMut<NextState<DilemmaPhase>>,
	locomotion_query : Query<&mut Locomotion, With<Train>>,
	background_query : Query<&mut BackgroundSprite>
) {

	if counter.timer.tick(time.delta()).just_finished() {
		next_sub_state.set(
			DilemmaPhase::Decision
		);

		Train::update_speed(
			locomotion_query, 
			50.0
		);
		BackgroundSprite::update_speed(background_query,0.0);
	}
}

pub fn cleanup_decision(
		mut commands : Commands,
		dashboard : ResMut<DilemmaDashboard>,
		background_query : Query<Entity, With<BackgroundSprite>>
	){

	for entity in background_query.iter() {
		commands.entity(entity).despawn();
	}
	dashboard.despawn(&mut commands);
}

#[derive(Component)]
pub struct LeverTrackTransform{
	pub track_y_positions : Vec<f32>
}

pub fn switch_junction(
		mut movement_query: Query<(&mut LeverTrackTransform, &mut Transform)>,
		mut track_query: Query<&mut TextColor, With<TrunkTrack>>,
		lever: Option<Res<Lever>>,
		time: Res<Time>,  // Add time resource to manage frame delta time
	) {

	if lever.is_some() {

		let unwrapped_lever: Res<Lever> = lever.unwrap();

		for (
			lever_transform, 
			mut transform
		) in movement_query.iter_mut() {
			let left_position: Vec3 = Vec3::new(transform.translation.x, -lever_transform.track_y_positions[0], 1.0);
			let right_position: Vec3 = Vec3::new(transform.translation.x, -lever_transform.track_y_positions[1], 1.0);

			let distance_threshold = 0.01; // Small threshold to determine when to snap to the final position
			let proportional_speed_factor = 0.1; // Factor to adjust the proportional speed
			let bounce_amplitude = 0.02; // Amplitude of the bounce effect
			let bounce_frequency = 10.0; // Frequency of the bounce effect

			let main_track = track_query.get_single_mut(); 

			if unwrapped_lever.state == LeverState::Right {
				main_track.unwrap().0 = OPTION_2_COLOR;
				let distance = (right_position - transform.translation).length();
				if distance > distance_threshold {
					let direction = (right_position - transform.translation).normalize();
					let movement_speed = distance * proportional_speed_factor;
					transform.translation += direction * movement_speed;
				} else {
					let bounce_offset = bounce_amplitude * (time.elapsed_secs() * bounce_frequency).sin();
					transform.translation = right_position + Vec3::new(bounce_offset, 0.0, 0.0);
				}
			} else if unwrapped_lever.state == LeverState::Left {
				main_track.unwrap().0 = OPTION_1_COLOR;
				let distance = (left_position - transform.translation).length();
				if distance > distance_threshold {
					let direction = (left_position - transform.translation).normalize();
					let movement_speed = distance * proportional_speed_factor;
					transform.translation += direction * movement_speed;
				} else {
					let bounce_offset = bounce_amplitude * (time.elapsed_secs() * bounce_frequency).sin();
					transform.translation = left_position + Vec3::new(bounce_offset, 0.0, 0.0);
				}
			}
		}
	} else {
		panic!("Lever motion check with non-existant lever!")
	}
}

pub fn check_if_person_in_path_of_train(
	mut lever_query: Query<(&Children, &BranchTrack)>,
	mut text_query: Query<&mut PersonSprite>,
	lever: Option<Res<Lever>>
) {
	if let Some(lever) = lever {		
		for (children, track) in lever_query.iter_mut(){
			for &child in children.iter() {
				if let Ok(mut person) = text_query.get_mut(child) {
					person.in_danger = (Some(track.index) == lever.state.to_int()) && !(lever.state == LeverState::Random);
				}
			}
		}
	}
}

pub fn update_timer(
		time: Res<Time>,
		mut timer_query: Query<(&mut DilemmaTimer, &mut Text2d)>,
		mut next_game_state: ResMut<NextState<DilemmaPhase>>
	) {
	
	for (mut timer, mut text) in timer_query.iter_mut() {

		timer.timer.tick(time.delta());

		let time_remaining = timer.timer.remaining_secs();

		text.0 = format!("{:.2}\n", time_remaining);

		if timer.timer.just_finished() {
			next_game_state.set(
				DilemmaPhase::ConsequenceAnimation
			)
		}
	}
}

#[derive(Resource)]
pub struct DramaticPauseTimer{
	pub speed_up_timer: Timer,
	pub scream_timer: Timer
}

#[derive(Component)]
pub struct LongScream;

pub fn consequence_animation_tick_down(
		time: Res<Time>,
		mut commands : Commands,
		asset_server: Res<AssetServer>,
		mut timer: ResMut<DramaticPauseTimer>,
		locomotion_query : Query<&mut Locomotion, With<Train>>,
		lever: Option<Res<Lever>>,
	) {

	timer.scream_timer.tick(time.delta());
	timer.speed_up_timer.tick(time.delta());

	if timer.scream_timer.just_finished() {
		if lever.is_some() {

		}
		commands.spawn((
			LongScream,
			AudioPlayer::<AudioSource>(asset_server.load(PathBuf::from("./sounds/male_scream_long.ogg"))),
            PlaybackSettings {
				paused : false,
				mode:  bevy::audio::PlaybackMode::Despawn,
				volume :bevy::audio::Volume::new(0.3),
				speed : 0.1,
				..default()
            }
		));
	} else if ! timer.scream_timer.finished() {

		let total_time: f32 = timer.scream_timer.duration().as_secs_f32();
		let elapsed_time: f32 = timer.scream_timer.elapsed_secs();

		let fraction: f32 = elapsed_time/total_time;

		let initial_speed: f32 = 50.0;
		let final_speed: f32 = 5.0;
		let speed_reduction: f32  = initial_speed - final_speed;

		let current_speed: f32 = initial_speed - fraction*speed_reduction;

		Train::update_speed(
			locomotion_query, 
			current_speed
		);
	}
}

pub fn consequence_animation_tick_up(
	time: Res<Time>,
	mut timer: ResMut<DramaticPauseTimer>,
	locomotion_query : Query<&mut Locomotion, With<Train>>,
	mut audio : Query<&mut AudioSink, With<LongScream>>
) {
	timer.scream_timer.tick(time.delta());
	timer.speed_up_timer.tick(time.delta());

	if timer.scream_timer.finished() {
		let total_time: f32 = timer.speed_up_timer.duration().as_secs_f32() - timer.scream_timer.duration().as_secs_f32();
		let elapsed_time: f32 = timer.speed_up_timer.elapsed_secs() - timer.scream_timer.duration().as_secs_f32();

		let fraction: f32 = elapsed_time/total_time;

		let initial_speed: f32 = 5.0;
		let final_speed: f32 = 500.0;
		let speed_reduction: f32  = initial_speed - final_speed;
		let current_speed: f32 = initial_speed - fraction*speed_reduction;

		Train::update_speed(
			locomotion_query,
			current_speed
		);

		let initial_speed: f32 = 0.1;
		let final_speed: f32 = 1.0;
		let speed_reduction: f32 = initial_speed - final_speed;
		let current_speed: f32 = initial_speed - fraction*speed_reduction;

		for sink in audio.iter_mut() {
			sink.set_speed(current_speed);
		}
	}
}

#[derive(Component)]
pub struct TrunkTrack;

#[derive(Component)]
pub struct BranchTrack{
	index : usize
}

#[derive(Component)]
#[require(Visibility, Transform)]
pub struct Turnout;

#[derive(Clone)]
pub struct Junction{
	pub dilemma : Dilemma
}

impl Component for Junction {

	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {
				let junction: Option<Junction> = {
                    let entity_mut = world.entity(entity);
                    entity_mut.get::<Junction>()
                        .map(|train: &Junction| train.clone())
                };

				let asset_server = world.get_resource::<AssetServer>().unwrap();
				let audio_vector = vec![											
					TransientAudio::new(
						asset_server.load("sounds/male_scream_1.ogg"),
						1.0,
						false,
						1.0
					),
					TransientAudio::new(
						asset_server.load("sounds/male_scream_2.ogg"),
						1.0,
						false,
						1.0
					),
					TransientAudio::new(
						asset_server.load("sounds/male_scream_3.ogg"),
						1.0,
						false,
						1.0
					)
				];

				let mut commands = world.commands();
				if let Some(junction) = junction {
					let dilemma = junction.dilemma; 

					let track_y_positions = vec![0.0, 100.0];				
					let (color, initial_y_position) = match dilemma.default_option {
						None => (Color::WHITE, 50.0),
						Some(ref option) if *option == 0 => (OPTION_1_COLOR, track_y_positions[*option]),
						Some(ref option) => (OPTION_2_COLOR, track_y_positions[*option]),
					};
			
					let mut track_entities = vec![];
					commands.entity(entity).with_children(
						|junction| {
							track_entities.push(
								junction.spawn((
									TrunkTrack,
									Track::new(600),
									TextColor(color),
									Transform::from_xyz(-2000.0, initial_y_position, 1.0)
							)).id());

							let turnout_entity = junction.spawn((
								Turnout,
								Transform::from_xyz( 1240.0, 0.0, 0.0),
								LeverTrackTransform{
									track_y_positions : track_y_positions.clone()
								}
							)).with_children( |turnout| {
								for (branch_index, (option, y_position)) in zip(dilemma.options, track_y_positions.clone()).enumerate() {
									track_entities.push(turnout.spawn((
										BranchTrack{index : branch_index},
										Track::new(300),
										Transform::from_xyz(0.0, y_position, 0.0)		
									)).with_children(|track: &mut ChildBuilder<'_>| {
											for fatality_index in 0..option.consequences.total_fatalities {
												track.spawn(
													(
														PersonSprite::default(),
														Transform::from_xyz(
															-1060.0 + fatality_index as f32 * 10.0,
															0.0,
															0.0 
														),
														TransientAudioPallet::new(
															vec![
																("exclamation".to_string(),
																audio_vector.clone()
															)]
														),
														BounceAnimation::new(40.0, 60.0)
													)
												).with_children(
													|parent| {
														parent.spawn(Emoticon::default());
													}
												);	
											}
										}
									).id());
								}
							}).id();

							track_entities.push(turnout_entity);
						}
					);
				}
			}
        );
    }
}