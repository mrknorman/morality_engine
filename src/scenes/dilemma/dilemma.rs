use std::time::Duration;
use serde::{
	Deserialize, 
	Serialize
};
use bevy::{
	ecs::component::{Mutable, StorageType::{self, Table}}, 
	prelude::*, 
};

use crate::{
	data::{
		states::DilemmaPhase, 
		stats::{
			DilemmaStats, 
			GameStats
		} 
	}, 
	entities::text::TextRaw, 
	scenes::{
		dialogue::content::*,
		ending::content::*, 
		Scene, 
		SceneQueue
	}, 
	systems::{
		inheritance::BequeathTextColor, 
		motion::Pulse,
		time::Dilation 
	}
};

use super::content::*;

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
		.add_systems(
			OnExit(DilemmaPhase::Consequence), 
			Dilemma::update_queue
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
	index : usize,
    name : String,
    description : String,
    humans : Option<Vec<Human>>,
    num_humans : Option<usize>
}

#[derive(Serialize, Deserialize)]
pub struct DilemmaLoader {
	index : String,
    name : String,
	narration_path : String,
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
	pub index : usize,
    pub name : String,
    pub description : String,
    humans : Vec<Human>,
    pub consequences : DilemmaOptionConsequences,
    pub num_humans : usize
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
			index : dilemma_option_loader.index,
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
					DilemmaPhase::Consequence
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
	type Mutability = Mutable;

	fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, context| {

				let timer_duration_seconds = world.entity(context.entity).get::<DilemmaTimer>().unwrap().timer.duration().as_secs_f32();

				let mut commands = world.commands();
				commands.entity(context.entity).insert(
					Text2d::new(format!("{:.2}\n", timer_duration_seconds ))
				);

				commands.entity(context.entity).insert(
					TextFont{
						font_size : 50.0,
						..default()
					}
				);

				commands.entity(context.entity).with_children( | parent | {
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
	pub narration_path : String,
    pub description : String,
    pub countdown_duration : Duration,
    pub options : Vec<DilemmaOption>,
    pub default_option : Option<usize>,
}

impl Dilemma {
	pub fn new(content : &DilemmaScene) -> Dilemma {
        let loaded_dilemma : DilemmaLoader = serde_json::from_str(content.content()).expect("Failed to parse embedded JSON");

		let options : Vec<DilemmaOption> = loaded_dilemma.options.iter().map(
			|option| DilemmaOption::from_loader(option.clone())
		).collect();

		Dilemma {
			index : loaded_dilemma.index,
			name : loaded_dilemma.name,
			narration_path : loaded_dilemma.narration_path,
			description : loaded_dilemma.description,
			countdown_duration : Duration::from_secs_f32(loaded_dilemma.countdown_duration_seconds),
			options,
			default_option : loaded_dilemma.default_option
		}
	}

	pub fn update_queue(
		mut queue: ResMut<SceneQueue>,
		stats: Res<GameStats>
	) {
		let latest = match stats.dilemma_stats.last() {
			Some(latest) => latest,
			None => panic!("Latest decision not found"),
		};
	   
		// Store the pattern match result in a variable first
		let next_scenes = match queue.current {
			Scene::Dilemma(DilemmaScene::Lab0(_)) => {
				lab_one(latest, stats.as_ref())
			},
			Scene::Dilemma(DilemmaScene::Lab1(_)) => {
				lab_two(latest, stats.as_ref())
			},
			Scene::Dilemma(DilemmaScene::PathInaction(_, stage)) => {
				inaction_path(latest, stats.as_ref(), stage + 1)
			},
			Scene::Dilemma(DilemmaScene::Lab2(_)) => {
				lab_three(latest, stats.as_ref())
			},
			Scene::Dilemma(DilemmaScene::PathDeontological(_, stage)) => {
				deontological_path(latest, stats.as_ref(), stage + 1)
			},
			_ => panic!("Update Memory: Should not reach this branch!")
		};
		
		// Now extend the queue after the match is complete
		queue.replace(next_scenes);
	}
}


fn lab_one(latest : &DilemmaStats, _ : &GameStats) -> Vec<Scene> {
	if latest.num_fatalities > 0 {
		vec![
			Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::Fail)),
			Scene::Ending(EndingScene::IdioticPsychopath)
		]
	} else if latest.num_decisions > 0 {
		if let Some(duration) = latest.duration_remaining_at_last_decision {
			if latest.num_decisions > 10 {
				vec![
					Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::FailVeryIndecisive)),
					Scene::Ending(EndingScene::Leverophile)
				]
			} else if duration < Duration::from_secs(1) {
				vec![
					Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::PassSlow)), 
					Scene::Dialogue(DialogueScene::Lab1b(Lab1bDialogue::DilemmaIntro)),
					Scene::Dilemma(DilemmaScene::Lab1(Lab1Dilemma::NearSightedBandit))
				]
			} else {
				vec![
					Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::PassIndecisive)), 
					Scene::Dialogue(DialogueScene::Lab1b(Lab1bDialogue::DilemmaIntro)),
					Scene::Dilemma(DilemmaScene::Lab1(Lab1Dilemma::NearSightedBandit))
				]
			}
		} else {
			panic!("Duration not recorded for some reason!");
		}
	} else {
		vec![
			Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::Pass)), 
			Scene::Dialogue(DialogueScene::Lab1b(Lab1bDialogue::DilemmaIntro)),
			Scene::Dilemma(DilemmaScene::Lab1(Lab1Dilemma::NearSightedBandit))
		]
	}
}

fn lab_two(latest : &DilemmaStats, stats : &GameStats) -> Vec<Scene> {
	if latest.num_fatalities > 0 {
		if latest.num_decisions > 0 {
			vec![
				Scene::Dialogue(DialogueScene::Lab2a(Lab2aDialogue::FailIndecisive)),
				Scene::Ending(EndingScene::Leverophile)
			]
		} else if stats.total_decisions == 0 {
			vec![
				Scene::Dialogue(DialogueScene::path_inaction(0, PathOutcome::Fail)),
				Scene::Dilemma(DilemmaScene::PATH_INACTION[0])
			]
		} else {
			vec![
				Scene::Dialogue(DialogueScene::Lab2a(Lab2aDialogue::Fail)),
				Scene::Ending(EndingScene::ImpatientPsychopath)
			]
		}
	} else if latest.num_decisions > 0 {
		if let (Some(duration), Some(average_duration)) = (latest.duration_remaining_at_last_decision, stats.overall_avg_time_remaining) {
			if average_duration < Duration::from_secs(1) {
				vec![
					Scene::Dialogue(DialogueScene::Lab2a(Lab2aDialogue::PassSlowAgain)), 
					Scene::Dialogue(DialogueScene::Lab2b(Lab2bDialogue::Intro)),
					Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem))
				]
			} else if duration < Duration::from_secs(1) {
				vec![
					Scene::Dialogue(DialogueScene::Lab2a(Lab2aDialogue::PassSlow)), 
					Scene::Dialogue(DialogueScene::Lab2b(Lab2bDialogue::Intro)),
					Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem))
				]
			} else {
				vec![
					Scene::Dialogue(DialogueScene::Lab2a(Lab2aDialogue::Pass)), 
					Scene::Dialogue(DialogueScene::Lab2b(Lab2bDialogue::Intro)),
					Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem))
				]
			}
		} else {
			panic!("Duration not recorded for some reason!");
		}
	} else {
		vec![
			Scene::Dialogue(DialogueScene::Lab2a(Lab2aDialogue::Pass)), 
			Scene::Dialogue(DialogueScene::Lab2b(Lab2bDialogue::Intro)),
			Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem))
		]
	}
}

fn lab_three(latest : &DilemmaStats, _ : &GameStats) -> Vec<Scene>  {
	if latest.num_fatalities == 5 {
		if latest.num_decisions > 0 {
			vec![
				Scene::Dialogue(DialogueScene::Lab3a(Lab3aDialogue::Fail)),
			]
		} else {
			vec![
				Scene::Dialogue(DialogueScene::path_deontological(0, PathOutcome::Fail)),
				Scene::Dilemma(DilemmaScene::PATH_DEONTOLOGICAL[0])
			]
		}
	} else {
		vec![
			Scene::Dialogue(DialogueScene::Lab3a(Lab3aDialogue::Pass)), 
			Scene::Dialogue(DialogueScene::Lab3b(Lab3bDialogue::Intro))
		]
	} 
}

fn inaction_path(_ : &DilemmaStats, stats : &GameStats, stage : usize) -> Vec<Scene>  {
	if stats.total_decisions > 0 && stage < 6 {
		vec![
			Scene::Dialogue(DialogueScene::path_inaction(stage, PathOutcome::Pass)), 
			Scene::Dialogue(DialogueScene::Lab2b(Lab2bDialogue::Intro)), 
			Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem))
		]
	} else if stage < DilemmaScene::PATH_INACTION.len() {
		vec![
			Scene::Dialogue(DialogueScene::path_inaction(stage, PathOutcome::Fail)), 
			Scene::Dilemma(DilemmaScene::PATH_INACTION[stage])
		]
	} else {
		vec![
			Scene::Ending(EndingScene::TrueNeutral)
		]
	}
}

fn deontological_path(latest : &DilemmaStats, _ : &GameStats, stage : usize) -> Vec<Scene> {
	if latest.num_fatalities == 1 && stage < 1 {
		vec![
			Scene::Dialogue(DialogueScene::path_deontological(stage, PathOutcome::Pass)),  
			Scene::Dialogue(DialogueScene::Lab2b(Lab2bDialogue::Intro))
		]
	} else if latest.num_fatalities == 1 && stage < 2 {
		vec![
			Scene::Dialogue(DialogueScene::path_deontological(stage, PathOutcome::Pass)),  
			Scene::Ending(EndingScene::SelectiveDeontologist)
		]
	} else if stage < DilemmaScene::PATH_DEONTOLOGICAL.len() {
		vec![
			Scene::Dialogue(DialogueScene::path_deontological(stage, PathOutcome::Fail)),
			Scene::Dilemma(DilemmaScene::PATH_DEONTOLOGICAL[stage])
		]
	} else {
		vec![
			Scene::Dialogue(DialogueScene::path_deontological(stage, PathOutcome::Fail)),  
			Scene::Ending(EndingScene::TrueDeontologist)
		]
	}
}
