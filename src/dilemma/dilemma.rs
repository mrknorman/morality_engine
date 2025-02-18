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
use bevy::{
	ecs::{
		component::StorageType, 
		component::StorageType::Table
	}, 
	prelude::*, 
	sprite::Anchor,
	text::{
		LineBreak, 
		TextBounds
	}
};

use crate::{
	background::BackgroundSprite, colors::{		
		OPTION_1_COLOR, 
		OPTION_2_COLOR
	}, 
	game_states::DilemmaPhase,
	inheritance::BequeathTextColor, 
	motion::Pulse, 
	text::TextRaw, 
	time::Dilation, 
	track::Track
};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DilemmaSystemsActive {
    #[default]
	False,
    True
}

pub struct DilemmaPlugin;
impl Plugin for DilemmaPlugin {
    fn build(&self, app: &mut App) {	
		app
		.init_state::<DilemmaSystemsActive>()
		.add_systems(
			Update,
			activate_systems
		).add_systems(
            Update,
            (
				DilemmaTimer::update,
				DilemmaTimer::start_pulse
			)
            .run_if(in_state(DilemmaSystemsActive::True))
        )
		.register_required_components::<DilemmaTimer, TextRaw>()
		.register_required_components::<DilemmaTimer, Text2d>()
		.register_required_components::<DilemmaTimer, BequeathTextColor>()
		.register_required_components::<DilemmaTimer, Pulse>()
		;
    }
}

fn activate_systems(
        mut state: ResMut<NextState<DilemmaSystemsActive>>,
        query: Query<&DilemmaTimer>
    ) {
        
	if !query.is_empty() {
		state.set(DilemmaSystemsActive::True)
	} else {
		state.set(DilemmaSystemsActive::False)
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
    pub total_fatalities : usize
}

#[derive(Debug, Clone)]
pub struct DilemmaOption {
    name : String,
    description : String,
    humans : Vec<Human>,
    pub consequences : DilemmaOptionConsequences,
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

		Self {
			name : dilemma_option_loader.name,
			description : dilemma_option_loader.description,
			humans,
			consequences,
			num_humans
		}
	}
}

pub struct DilemmaTimer{
	pub timer : Timer,
	pulse_start_time : Duration,
	pulse_speedup_time : Duration,
	sped_up : bool
}

impl DilemmaTimer {

	pub fn new(
		duration : Duration, 
		pulse_start_time : Duration, 
		pulse_speedup_time : Duration
	) -> Self { 

		Self{
			timer : Timer::new(duration, TimerMode::Once),
			pulse_start_time,
			pulse_speedup_time,
			sped_up : false
		}
	}

	pub fn update(
		time: Res<Time>,
		dilation : Res<Dilation>,
		mut timer_query: Query<(&mut DilemmaTimer, &mut Text2d)>,
		mut next_game_state: ResMut<NextState<DilemmaPhase>>
	) {
	
		for (mut timer, mut text) in timer_query.iter_mut() {

			timer.timer.tick(time.delta().mul_f32(dilation.0));
			text.0 = format!("{:.2}\n", timer.timer.remaining_secs());
			if timer.timer.finished() {
				next_game_state.set(
					DilemmaPhase::ConsequenceAnimation
				)
			}
		}
	}

	pub fn start_pulse(
		mut timer_query: Query<(&mut DilemmaTimer, &mut Pulse)>,
	) {
		for (mut timer, mut pulse) in timer_query.iter_mut() {
			pulse.active = timer.timer.remaining() <= timer.pulse_start_time + pulse.interval_timer.duration() + pulse.pulse_timer.duration();
		
			if (timer.timer.remaining() <= timer.pulse_speedup_time + Duration::from_secs_f32(0.5) + pulse.pulse_timer.duration()) && !timer.sped_up {
				pulse.interval_timer = Timer::new(
					Duration::from_secs_f32(0.5) - pulse.pulse_timer.duration().mul_f32(2.0),
					TimerMode::Once
				);
				
				timer.sped_up = true;
			}
		}
	}
}

impl Component for DilemmaTimer{

	const STORAGE_TYPE: StorageType = Table;

	fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {

				let timer_duration_seconds = world.entity(entity).get::<DilemmaTimer>().unwrap().timer.duration().as_secs_f32();

				let mut commands = world.commands();
				commands.entity(entity).insert(
					Text2d::new(format!("{:.2}\n", timer_duration_seconds ))
				);

				commands.entity(entity).insert(
					TextFont{
						font_size : 50.0,
						..default()
					}
				);

				commands.entity(entity).with_children( | parent | {
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
				});
			});
		} 
}

#[derive(Resource, Clone)]
pub struct Dilemma {
	pub index : String,
    pub name : String,
    pub description : String,
    pub countdown_duration : Duration,
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
			countdown_duration : Duration::from_secs_f32(loaded_dilemma.countdown_duration_seconds),
			options,
			default_option : loaded_dilemma.default_option
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

pub fn cleanup_decision(
		mut commands : Commands,
		background_query : Query<Entity, With<BackgroundSprite>>,
		track_query : Query<Entity, With<Track>>
	){
	
	for entity in background_query.iter() {
		commands.entity(entity).despawn();
	}
	for entity in track_query.iter() {
		commands.entity(entity).despawn();
	}
}