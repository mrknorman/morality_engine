use std::{
	fs::File, 
	io::BufReader, 
	path::PathBuf, 
	time::Duration
};
use serde::{
	Deserialize, 
	Serialize
};
use rand::Rng;

use bevy::{
	prelude::*, 
	sprite::Anchor, 
	text::{
		LineBreak, 
		TextBounds
	},
	color::palettes::css::RED
};

use crate::{
	audio::play_sound_once, 
	lever::{
		Lever, 
		LeverState, 
		OPTION_1_COLOR, 
		OPTION_2_COLOR
	}, 
	train::{
        Train,
        TrainBundle,
        STEAM_TRAIN
    },
	track::{
        Track,
        TrackBundle
    },
	person::{
		PERSON,
		PERSON_IN_DANGER,
		PersonSprite,
		BounceAnimation,
		EmoticonSprite,
		EXCLAIMATION,
		NEUTRAL
	},
	background::BackgroundSprite,
	motion::{
		PointToPointTranslation,
		Locomotion
	},
	game_states::DilemmaPhase
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

		let timer : Entity  = DilemmaTimer::spawn(
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
			Some(ref option) if *option == 1 => LeverState::Left,
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
							"seconds remaining.\n".to_string(),
							TextStyle {
								font_size: 12.0,
								..default()
							}
						)
					],
					justify : JustifyText::Center, 
					linebreak_behavior: LineBreak::WordBoundary
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

#[derive(Bundle)]
pub struct DilemmaInfoPanelBundle{
	pub marker : DilemmaInfoPanel,
	pub text : Text2dBundle
}

impl DilemmaInfoPanelBundle {
	pub fn new(
		dilemma : &Dilemma
	) -> Self {
		DilemmaInfoPanelBundle {
			marker : DilemmaInfoPanel,
			text : Text2dBundle {
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
					linebreak_behavior: LineBreak::WordBoundary
				},
				text_2d_bounds : TextBounds {
					size: Vec2::new(500.0, 2000.0),
				},
				transform: Transform::from_xyz(-600.0,300.0, 1.0),
				text_anchor : Anchor::TopLeft,
				..default()
			}
		}
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
			color =OPTION_1_COLOR;
			x_transform = -600.0;
		} else if index == 1 {
			color = OPTION_2_COLOR;
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
								index + 1, 
								option.name, 
								index + 1
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
					linebreak_behavior: LineBreak::WordBoundary
				},
				text_2d_bounds : TextBounds {
					size: box_size,
				},
				transform: Transform::from_xyz(x_transform,-150.0, 1.0),
				text_anchor : Anchor::TopLeft,
				..default()
			})
		).id()
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
		junction : ResMut<TrainJunction>,
		background_query : Query<Entity, With<BackgroundSprite>>
	){
	

	for entity in background_query.iter() {
		commands.entity(entity).despawn();
	}

	dashboard.despawn(&mut commands);
	junction.despawn(&mut commands);
}

pub fn lever_motion(
    mut movement_query: Query<(&mut LeverTrackTransform, &mut Transform)>,
	mut track_query: Query<&mut Text, (With<Track>, Without<LeverTrackTransform>)>,
    lever: Option<Res<Lever>>,
    time: Res<Time>,  // Add time resource to manage frame delta time
	
) {

	if lever.is_some() {

		let unwrapped_lever: Res<Lever> = lever.unwrap();

		for (
			lever_transform, 
			mut transform
		) in movement_query.iter_mut() {
			let right_position: Vec3 = lever_transform.initial + lever_transform.right;
			let left_position: Vec3 = lever_transform.initial + lever_transform.left;

			let distance_threshold = 0.01; // Small threshold to determine when to snap to the final position
			let proportional_speed_factor = 0.1; // Factor to adjust the proportional speed
			let bounce_amplitude = 0.02; // Amplitude of the bounce effect
			let bounce_frequency = 10.0; // Frequency of the bounce effect

			let main_track = track_query.get_single_mut(); 

			if unwrapped_lever.state == LeverState::Right {
				main_track.unwrap().sections[0].style.color = OPTION_2_COLOR;
				let distance = (right_position - transform.translation).length();
				if distance > distance_threshold {
					let direction = (right_position - transform.translation).normalize();
					let movement_speed = distance * proportional_speed_factor;
					transform.translation += direction * movement_speed;
				} else {
					let bounce_offset = bounce_amplitude * (time.elapsed_seconds() * bounce_frequency).sin();
					transform.translation = right_position + Vec3::new(bounce_offset, 0.0, 0.0);
				}
			} else if unwrapped_lever.state == LeverState::Left {
				main_track.unwrap().sections[0].style.color = OPTION_1_COLOR;
				let distance = (left_position - transform.translation).length();
				if distance > distance_threshold {
					let direction = (left_position - transform.translation).normalize();
					let movement_speed = distance * proportional_speed_factor;
					transform.translation += direction * movement_speed;
				} else {
					let bounce_offset = bounce_amplitude * (time.elapsed_seconds() * bounce_frequency).sin();
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

					let lever_state: &LeverState = &unwrapped_lever.state;

					if *lever_state != LeverState::Random {
						person.in_danger = lever_transform.index == unwrapped_lever.state.to_int();
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
	mut emoticon_query: Query<(&mut EmoticonSprite, &mut Transform, &mut Text), Without<PersonSprite>>,
	mut commands : Commands,
	asset_server: Res<AssetServer>
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
								color : Color::from(RED),
								..default()
							}
						);
					}
				}
				
				text.sections[0] = TextSection::new(
					String::from(PERSON_IN_DANGER),
					TextStyle {
						font_size: 12.0,
						color : Color::from(RED),
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
						Duration::from_secs_f32(rand::random::<f32>() * 2.0 + 1.0)
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

					play_sound_once("./sounds/male_scream.ogg", &mut commands, &asset_server);
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
		mut timer_query: Query<(&mut DilemmaTimer, &mut Text)>,
		mut next_game_state: ResMut<NextState<DilemmaPhase>>
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
		);

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
			AudioBundle {
				source: asset_server.load(PathBuf::from("./sounds/male_scream_long.ogg")),
				settings : PlaybackSettings {
					paused : false,
					mode:  bevy::audio::PlaybackMode::Despawn,
					volume :bevy::audio::Volume::new(0.3),
					speed : 0.1,
					..default()
				}
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
pub struct LeverTrackTransform{
	pub index : usize,
	pub initial : Vec3,
	pub left : Vec3,
	pub right : Vec3,
	pub random : Vec3
}

#[derive(Resource)]
pub struct TrainJunction{
	train : Entity,
	track : Vec<Entity>
}

impl TrainJunction{

	pub fn spawn(
			commands : &mut Commands,
			asset_server: &Res<AssetServer>,
			dilemma: &Dilemma
		) {

		let final_position = Vec3::new(
			90.0 * dilemma.countdown_duration_seconds,
			0.0, 
			0.0
		);

		let train: Entity = commands.spawn(
			TrainBundle::new(
				asset_server,
				STEAM_TRAIN,
				Vec3::new(50.0, -5.0, 1.0),
				0.0
			)
		).id();

		let color = match dilemma.default_option {
			None => Color::WHITE,
			Some(ref option) if *option == 1 => OPTION_1_COLOR,
			Some(_) =>  OPTION_2_COLOR,
		};

		let lower_track_y: f32 = -40.0;
		let upper_track_y: f32 = 60.0;

		let main_track_translation_end: Vec3 = Vec3::new(-1700.0, lower_track_y, 0.0);
		let main_track_translation_start: Vec3 = main_track_translation_end + final_position;
		let main_track: TrackBundle = TrackBundle::new(
			600, 
			main_track_translation_start
		);

		let track_1_translation_end: Vec3 = Vec3{x : 1000.0 , y : lower_track_y, z: 0.0};
		let track_1_translation_start: Vec3= track_1_translation_end + final_position;
		let track_1: TrackBundle = TrackBundle::new(
			300, 
			track_1_translation_start
		);

		let track_2_translation_end: Vec3 = Vec3{x : 1000.0 , y : upper_track_y, z: 0.0};
		let track_2_translation_start: Vec3 = track_2_translation_end + final_position;
		let track_2: TrackBundle = TrackBundle::new(
			300, 
			track_2_translation_start
		);
	
		let main_track : Entity = commands.spawn(main_track).id();
		let track_1 : Entity = commands.spawn(track_1).id();
		let track_2: Entity = commands.spawn(track_2).id();

		commands.entity(main_track).insert(
			PointToPointTranslation::new(
				main_track_translation_start, 
				main_track_translation_end,
				dilemma.countdown_duration_seconds
			)
		);
		commands.entity(track_1).insert((
			PointToPointTranslation::new(
				track_1_translation_start, 
				track_1_translation_end,
				dilemma.countdown_duration_seconds
			),
			LeverTrackTransform{
				index : 1, 
				initial : track_1_translation_end,
				left : Vec3{x: 0.0, y: 0.0, z: 0.0},
				right : Vec3{x: 0.0, y: -100.0, z: 0.0},
				random : Vec3{x: 0.0, y: -50.0, z: 0.0}
			})
		);
		commands.entity(track_2).insert((
			PointToPointTranslation::new(
				track_2_translation_start, 
				track_2_translation_end,
				dilemma.countdown_duration_seconds
			),
			LeverTrackTransform{
				index : 2,
				initial : track_2_translation_end,
				left : Vec3{x: 0.0, y: 0.0, z: 0.0},
				right : Vec3{x: 0.0, y: -100.0, z: 0.0},
				random : Vec3{x: 0.0, y: -50.0, z: 0.0}
			}
		));
		
		let person = String::from(PERSON);
		for _ in 0..dilemma.options[0].consequences.total_fatalities {
			let position: Vec3 = Vec3::new(-800.0, 0.0, 0.0);
			commands.entity(track_1).with_children(|parent| {
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
								linebreak_behavior: LineBreak::WordBoundary
							},
							transform: Transform::from_translation(
								position
							),
							text_anchor : Anchor::BottomCenter,
							..default()
						},
						PersonSprite::new(),
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
	
		for _ in 0..dilemma.options[1].consequences.total_fatalities {
			let position: Vec3 = Vec3::new(-800.0, 0.0, 0.0);
			commands.entity(track_2).with_children(|parent| {
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
								linebreak_behavior: LineBreak::WordBoundary
							},
							transform: Transform::from_translation(
								position
							),
							text_anchor : Anchor::BottomCenter,
							..default()
						},
						PersonSprite::new(),
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
		
		let track = vec![main_track, track_1, track_2];
		let junction: TrainJunction = TrainJunction{
			train,
			track
		};

		commands.insert_resource(junction);
	}

	pub fn despawn( 
		&self,
		commands: &mut Commands
	) {		
		for track in self.track.clone() {
			commands.entity(track).despawn();
		}
	}
}