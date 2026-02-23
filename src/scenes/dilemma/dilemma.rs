use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use rand::Rng;
use serde::{
    de::{Deserializer, Error},
    Deserialize, Serialize,
};
use std::{path::PathBuf, str::FromStr, time::Duration};

use crate::{
    data::{
        states::DilemmaPhase,
        stats::GameStats,
    },
    entities::text::{scaled_font_size, TextRaw},
    scenes::{flow::next_scenes_for_current_dilemma, SceneFlowMode, SceneQueue},
    systems::{inheritance::BequeathTextColor, motion::Pulse, time::Dilation},
};

use super::content::*;

fn has_dilemma_timers(q: Query<&DilemmaTimer>) -> bool {
    !q.is_empty()
}

pub struct DilemmaPlugin;
impl Plugin for DilemmaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (DilemmaTimer::update, DilemmaTimer::start_pulse)
                .run_if(has_dilemma_timers)
                .run_if(in_state(DilemmaPhase::Decision)),
        )
        .add_systems(OnExit(DilemmaPhase::Consequence), Dilemma::update_queue);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Culpability {
    Uninvolved,
    Forced,
    Accidental,
    Negligent,
    Willing,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Gender {
    Male,
    Female,
    NonBinary,
    Other,
    None,
    Unknown,
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
    Unknown,
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
    Unknown,
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
            RandomizableUsize::Uniform { min, max } => rand::rng().random_range(*min..=*max),
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

                Err(D::Error::custom(format!(
                    "Invalid randomizable usize format: {}",
                    s
                )))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DilemmaOptionLoader {
    index: usize,
    name: String,
    description: String,
    humans: Option<Vec<Human>>,
    num_humans: Option<RandomizableUsize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DilemmaStageLoader {
    #[serde(default = "default_repeat")]
    repeat: usize,
    countdown_duration_seconds: f32,
    options: Vec<DilemmaOptionLoader>,
    option_count: Option<RandomizableUsize>,
    default_option: Option<usize>,
    #[serde(default = "default_speed")]
    speed: f32,
}

fn default_repeat() -> usize {
    1
}
fn default_speed() -> f32 {
    70.0
}

#[derive(Serialize, Deserialize)]
pub struct DilemmaLoader {
    index: String,
    name: String,
    narration_path: PathBuf,
    description: String,
    stages: Vec<DilemmaStageLoader>,
    music_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Human {
    fatality_probability: f64,
    culpability: Option<Culpability>,
    name: Option<String>,
    gender: Option<Gender>,
    age: Option<u64>,
    iq: Option<u64>,
    highest_education: Option<EducationLevel>,
    occupation: Option<Job>,
}

#[derive(Debug, Clone, Copy)]
pub struct DilemmaOptionConsequences {
    pub total_fatalities: usize,
}

#[derive(Debug, Clone)]
pub struct DilemmaOption {
    pub index: usize,
    pub name: String,
    pub description: String,
    humans: Vec<Human>,
    pub consequences: DilemmaOptionConsequences,
    pub num_humans: usize,
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
    pub countdown_duration: Duration,
    pub options: Vec<DilemmaOption>,
    pub default_option: Option<usize>,
    pub speed: f32,
}

#[derive(Resource, Clone)]
pub struct CurrentDilemmaStageIndex(pub usize);

#[derive(Component)]
#[require(TextRaw, Text2d, BequeathTextColor, Pulse)]
#[component(on_insert = DilemmaTimer::on_insert)]
pub struct DilemmaTimer {
    pub timer: Timer,
    pulse_start_time: Duration,
    pulse_speedup_time: Duration,
    sped_up: bool,
}

impl DilemmaTimer {
    pub fn new(
        duration: Duration,
        pulse_start_time: Duration,
        pulse_speedup_time: Duration,
    ) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Once),
            pulse_start_time,
            pulse_speedup_time,
            sped_up: false,
        }
    }

    pub fn update(
        time: Res<Time>,
        dilation: Res<Dilation>,
        mut timer_query: Query<(&mut DilemmaTimer, &mut Text2d)>,
        mut next_game_state: ResMut<NextState<DilemmaPhase>>,
    ) {
        for (mut timer, mut text) in timer_query.iter_mut() {
            timer.timer.tick(time.delta().mul_f32(dilation.0));
            text.0 = format!("{:.2}\n", timer.timer.remaining_secs());
            if timer.timer.is_finished() {
                *next_game_state = NextState::PendingIfNeq(DilemmaPhase::Consequence);
            }
        }
    }

    pub fn start_pulse(mut timer_query: Query<(&mut DilemmaTimer, &mut Pulse)>) {
        for (mut timer, mut pulse) in timer_query.iter_mut() {
            pulse.active = timer.timer.remaining()
                <= timer.pulse_start_time
                    + pulse.interval_timer.duration()
                    + pulse.pulse_timer.duration();

            if (timer.timer.remaining()
                <= timer.pulse_speedup_time
                    + Duration::from_secs_f32(0.5)
                    + pulse.pulse_timer.duration())
                && !timer.sped_up
            {
                pulse.interval_timer = Timer::new(
                    Duration::from_secs_f32(0.5) - pulse.pulse_timer.duration().mul_f32(2.0),
                    TimerMode::Once,
                );

                timer.sped_up = true;
            }
        }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let timer_duration_seconds = world
            .entity(entity)
            .get::<DilemmaTimer>()
            .unwrap()
            .timer
            .duration()
            .as_secs_f32();

        let mut commands = world.commands();
        commands
            .entity(entity)
            .insert(Text2d::new(format!("{:.2}\n", timer_duration_seconds)));

        commands.entity(entity).insert(TextFont {
            font_size: scaled_font_size(50.0),
            ..default()
        });

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                TextSpan::new("seconds remaining.\n".to_string()),
                TextFont {
                    font_size: scaled_font_size(12.0),
                    ..default()
                },
            ));
        });
    }
}

#[derive(Resource, Clone)]
pub struct Dilemma {
    pub index: String,
    pub name: String,
    pub narration_path: PathBuf,
    pub description: String,
    pub stages: Vec<DilemmaStage>,
    pub music_path: PathBuf,
}

impl Dilemma {
    pub fn new(content: &DilemmaScene) -> Self {
        let loaded_dilemma: DilemmaLoader =
            serde_json::from_str(content.content()).expect("Failed to parse embedded JSON");

        let mut stages: Vec<DilemmaStage> = Vec::new();

        for stage_loader in loaded_dilemma.stages {
            for _ in 0..stage_loader.repeat {
                let total_option_templates = stage_loader.options.len();
                assert!(
                    total_option_templates > 0,
                    "Dilemma stage must define at least one option"
                );
                let resolved_option_count = stage_loader
                    .option_count
                    .as_ref()
                    .map(RandomizableUsize::resolve)
                    .unwrap_or(total_option_templates)
                    .clamp(1, total_option_templates.max(1));

                let options: Vec<DilemmaOption> = stage_loader
                    .options
                    .iter()
                    .take(resolved_option_count)
                    .map(|o| DilemmaOption::from_loader(o.clone())) // rerun random resolution here
                    .collect();

                let default_option = stage_loader
                    .default_option
                    .map(|idx| idx.min(options.len().saturating_sub(1)));

                stages.push(DilemmaStage {
                    countdown_duration: Duration::from_secs_f32(
                        stage_loader.countdown_duration_seconds,
                    ),
                    options,
                    default_option,
                    speed: stage_loader.speed,
                });
            }
        }

        Self {
            index: loaded_dilemma.index,
            name: loaded_dilemma.name,
            narration_path: loaded_dilemma.narration_path,
            description: loaded_dilemma.description,
            stages,
            music_path: loaded_dilemma.music_path,
        }
    }

    pub fn update_queue(mut queue: ResMut<SceneQueue>, stats: Res<GameStats>) {
        if queue.flow_mode() == SceneFlowMode::SingleLevel {
            return;
        }

        let latest = match stats.dilemma_stats.last() {
            Some(latest) => latest,
            None => {
                warn!("cannot update scene queue without latest dilemma stats");
                return;
            }
        };

        let Some(next_scenes) = next_scenes_for_current_dilemma(queue.current, latest, stats.as_ref())
        else {
            return;
        };

        queue.replace(next_scenes);
    }
}
