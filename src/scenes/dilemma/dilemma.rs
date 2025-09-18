use std::{
	time::Duration,
	str::FromStr
};
use serde::{
	Deserialize, 
	Serialize,
	de::{Error, Deserializer}
};
use bevy::{
	ecs::{
		component::HookContext, 
		world::DeferredWorld, 
	},
	prelude::*
};
use rand::Rng;

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

use super::{content::*, lever::LeverState};

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
		);
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

#[derive(Debug, Clone, Serialize)]
pub enum RandomizableUsize {
    Fixed(usize),
    Uniform { min: usize, max: usize },
}

impl RandomizableUsize {
    pub fn resolve(&self) -> usize {
        match self {
            RandomizableUsize::Fixed(v) => *v,
            RandomizableUsize::Uniform { min, max } => {
                rand::rng().random_range(*min..=*max)
            }
        }
    }
}

impl<'de> Deserialize<'de> for RandomizableUsize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // This enum helps us accept both raw numbers and strings
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Helper {
            Int(usize),
            Str(String),
        }

        match Helper::deserialize(deserializer)? {
            Helper::Int(v) => Ok(RandomizableUsize::Fixed(v)),
            Helper::Str(s) => {
                // Try parse as plain number first
                if let Ok(v) = usize::from_str(&s) {
                    return Ok(RandomizableUsize::Fixed(v));
                }

                // Try parse as uniform(min,max)
                if let Some(inner) = s.strip_prefix("uniform(").and_then(|s| s.strip_suffix(")")) {
                    let parts: Vec<&str> = inner.split(',').collect();
                    if parts.len() == 2 {
                        let min = parts[0].trim().parse().map_err(D::Error::custom)?;
                        let max = parts[1].trim().parse().map_err(D::Error::custom)?;
                        return Ok(RandomizableUsize::Uniform { min, max });
                    }
                }

                Err(D::Error::custom(format!("Invalid num_humans format: {}", s)))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DilemmaOptionLoader {
	index : usize,
    name : String,
    description : String,
    humans : Option<Vec<Human>>,
    num_humans : Option<RandomizableUsize>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DilemmaStageLoader {
    #[serde(default = "default_repeat")]
    repeat: usize,
    countdown_duration_seconds: f32,
    options: Vec<DilemmaOptionLoader>,
    default_option: Option<usize>,
}

fn default_repeat() -> usize { 1 }

#[derive(Serialize, Deserialize)]
pub struct DilemmaLoader {
	index : String,
    name : String,
	narration_path : String,
    description : String,
	stages : Vec<DilemmaStageLoader>
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
	 fn from_loader(dilemma_option_loader: DilemmaOptionLoader) -> Self {
        let humans = dilemma_option_loader.humans.unwrap_or_default();

        let num_humans = match dilemma_option_loader.num_humans {
            Some(v) => v.resolve(),
            None => humans.len(),
        };

        let description = dilemma_option_loader
            .description
            .replace("{num_humans}", &num_humans.to_string());

        let consequences = DilemmaOptionConsequences {
            total_fatalities: num_humans,
        };

        Self {
            index: dilemma_option_loader.index,
            name: dilemma_option_loader.name,
            description,
            humans,
            consequences,
            num_humans,
        }
    }
}

#[derive(Resource, Clone)]
pub struct DilemmaStage {
	pub countdown_duration : Duration,
    pub options : Vec<DilemmaOption>,
    pub default_option : Option<usize>,
}

#[derive(Resource, Clone)]
pub struct CurrentDilemmaStageIndex(pub usize);

#[derive(Component)]
#[require(TextRaw, Text2d, BequeathTextColor, Pulse)]
#[component(on_insert = DilemmaTimer::on_insert)]
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

	fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {

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

	}
}

#[derive(Resource, Clone)]
pub struct Dilemma {
	pub index : String,
    pub name : String,
	pub narration_path : String,
    pub description : String,
	pub stages : Vec<DilemmaStage>
}

impl Dilemma {
	pub fn new(content: &DilemmaScene) -> Self {
        let loaded_dilemma: DilemmaLoader =
            serde_json::from_str(content.content()).expect("Failed to parse embedded JSON");

        let mut stages: Vec<DilemmaStage> = Vec::new();

        for stage_loader in loaded_dilemma.stages {
            for _ in 0..stage_loader.repeat {
                let options: Vec<DilemmaOption> = stage_loader
                    .options
                    .iter()
                    .map(|o| DilemmaOption::from_loader(o.clone())) // rerun random resolution here
                    .collect();

                stages.push(DilemmaStage {
                    countdown_duration: Duration::from_secs_f32(stage_loader.countdown_duration_seconds),
                    options,
                    default_option: stage_loader.default_option,
                });
            }
        }

        Self {
            index: loaded_dilemma.index,
            name: loaded_dilemma.name,
            narration_path: loaded_dilemma.narration_path,
            description: loaded_dilemma.description,
            stages,
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
			Scene::Dilemma(DilemmaScene::Lab3(Lab3Dilemma::AsleepAtTheJob)) => {
				lab_three_junction(latest, stats.as_ref())
			},
			Scene::Dilemma(DilemmaScene::PathUtilitarian(_, stage)) => {
				utilitarian_path(latest, stats.as_ref(), stage + 1)
			},
			Scene::Dilemma(DilemmaScene::PathDeontological(_, stage)) => {
				deontological_path(latest, stats.as_ref(), stage + 1)
			},
			Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths)) => {
				return
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
				Scene::Dialogue(DialogueScene::Lab3a(Lab3aDialogue::FailIndecisive)),
			]
		} else {
			vec![
				Scene::Dialogue(DialogueScene::Lab3a(Lab3aDialogue::FailInaction)),
				Scene::Dilemma(DilemmaScene::Lab3(Lab3Dilemma::AsleepAtTheJob))
			]
		}
	} else {
		vec![
			Scene::Dialogue(DialogueScene::Lab3a(Lab3aDialogue::PassUtilitarian)), 
			Scene::Dialogue(DialogueScene::Lab3b(Lab3bDialogue::Intro)),
			Scene::Dilemma(DilemmaScene::PATH_UTILITARIAN[0])
		]
	} 
}

fn lab_three_junction(latest : &DilemmaStats, _ : &GameStats) -> Vec<Scene>  {
	if latest.num_fatalities == 5 {
		if latest.num_decisions > 0 {
			vec![
				Scene::Dialogue(DialogueScene::Lab3a(Lab3aDialogue::FailIndecisive))
			]
		} else {
			vec![
				Scene::Dialogue(DialogueScene::path_deontological(0, PathOutcome::Fail)),
				Scene::Dilemma(DilemmaScene::PATH_DEONTOLOGICAL[0])
			]
		}
	} else {
		todo!("?")
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

fn utilitarian_path(latest: &DilemmaStats, _: &GameStats, stage: usize) -> Vec<Scene> {
    let lever_state = latest
        .result
        .expect("LeverState should not be none");

    match (lever_state, stage) {
        (LeverState::Left, 4) => vec![
            Scene::Dialogue(DialogueScene::path_utilitarian(stage, PathOutcome::Pass)),
            Scene::Dialogue(DialogueScene::Lab4(Lab4Dialogue::Outro)),
            Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths)),
        ],

		(LeverState::Right, 4) => vec![
            Scene::Dialogue(DialogueScene::path_utilitarian(stage, PathOutcome::Fail)),
            Scene::Dialogue(DialogueScene::Lab4(Lab4Dialogue::Outro)),
            Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths)),
        ],

        (LeverState::Right, stage) => vec![
            Scene::Dialogue(DialogueScene::path_utilitarian(stage, PathOutcome::Pass)),
            Scene::Dilemma(DilemmaScene::PATH_UTILITARIAN[stage]),
        ],

        (LeverState::Left, _) => vec![
            Scene::Dialogue(DialogueScene::path_utilitarian(stage, PathOutcome::Fail)),
            Scene::Dialogue(DialogueScene::Lab4(Lab4Dialogue::Outro)),
            Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths)),
        ],

		(LeverState::Random, _) => panic!("Lever State should not be random")
    }
}