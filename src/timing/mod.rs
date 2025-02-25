use std::time::Duration;

use bevy::{
    prelude::*, 
    time::Timer
};
use enum_map::{
	Enum, 
	EnumArray, 
	EnumMap
};

use crate::{
	audio::NarrationAudioFinished, 
	dilemma::{
		phases::consequence::DilemmaConsequenceEvents, 
		phases::intro::DilemmaIntroEvents
	}, 
	loading::LoadingEvents
};

pub struct TimingPlugin;
impl Plugin for TimingPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(
            Update,
            (
                TimerPallet::<DilemmaIntroEvents>::tick,
				TimerPallet::<DilemmaIntroEvents>::check_start_conditions,
				TimerPallet::<DilemmaConsequenceEvents>::tick,
				TimerPallet::<DilemmaConsequenceEvents>::check_start_conditions,
				TimerPallet::<LoadingEvents>::tick,
				TimerPallet::<LoadingEvents>::check_start_conditions
            )
        );
    }
}

#[derive(PartialEq, Clone)]
pub enum TimerStartCondition{
    Immediate,
    OnNarrationEnd
}

#[derive(Clone)]
pub struct ConditionalTimer{
	start_condition : TimerStartCondition,
	timer : Timer,
	repeat_duration_seconds : Option<f32>,
	times_finished : u32
}

impl ConditionalTimer {
	
	fn new(
		config : TimerConfig
	) -> Self {

		let mut timer = Timer::from_seconds(config.duration_seconds, TimerMode::Once);

		if config.start_condition != TimerStartCondition::Immediate {
			timer.pause();
		}
		
		Self {
			start_condition : config.start_condition,
			timer,
			repeat_duration_seconds : config.repeat_duration_seconds,
			times_finished : 0
		}
	}

	pub fn just_finished(&self) -> bool {
		self.timer.just_finished()
	}

	pub fn times_finished(&self) -> u32 {
		self.times_finished
	}

	fn start_repeat_timer(&mut self) {
		if let Some(repeat_duration_seconds) = self.repeat_duration_seconds {
			if self.just_finished() && self.times_finished == 1 {
				self.timer.set_duration(
					Duration::from_secs_f32(repeat_duration_seconds)
				);
				self.timer.set_mode(TimerMode::Repeating);
			};
		}
	}

	fn count_completions(&mut self) {
		if self.just_finished() {
			self.times_finished += 1;
		};
	}
}

impl Default for ConditionalTimer {
	fn default() -> Self {
		Self {
			start_condition : TimerStartCondition::Immediate,
			timer : Timer::from_seconds(0.0, TimerMode::Once),
			repeat_duration_seconds : None,
			times_finished : 0
		}
	}
}

pub struct TimerConfig{
	start_condition : TimerStartCondition,
	duration_seconds : f32,
	repeat_duration_seconds : Option<f32>
}

impl TimerConfig {
	pub fn new(
		start_condition : TimerStartCondition,
		duration_seconds : f32,
		repeat_duration_seconds : Option<f32>
	) -> Self {

		Self {
			start_condition,
			duration_seconds,
			repeat_duration_seconds
		}
	}
}
#[derive(Component)]
pub struct TimerPallet<K>(
    pub EnumMap<K, ConditionalTimer>
)
where
    K: Enum + EnumArray<ConditionalTimer> + Send + Sync + Clone + 'static,
    <K as EnumArray<ConditionalTimer>>::Array: Clone + Send + Sync;

impl<K> TimerPallet <K>
where
	K: Enum + EnumArray<ConditionalTimer> + Send + Sync + Clone + 'static,
	<K as EnumArray<ConditionalTimer>>::Array: Clone + Send + Sync
{
	pub fn new(timer_vector: Vec<(K, TimerConfig)>) -> Self {
		Self(
			timer_vector
				.into_iter()
				.map(|(key, config)| (key, ConditionalTimer::new(config)))
				.collect(),
		)
	}

	pub fn tick(time: Res<Time>, mut timer_query: Query<&mut TimerPallet<K>>) {
		let delta = time.delta();
		timer_query.iter_mut().for_each(|mut pallet| {
			pallet.0.iter_mut().for_each(|(_, timer)| {
				timer.timer.tick(delta);
				timer.count_completions();
				timer.start_repeat_timer();
			});
		});
	}

	pub fn check_start_conditions(
		mut ev_narration_finished: EventReader<NarrationAudioFinished>,
		mut timer_query: Query<&mut TimerPallet<K>>,
	) {
		if ev_narration_finished.read().next().is_some() {
			timer_query.iter_mut().for_each(|mut pallet| {
				pallet.0.iter_mut().for_each(|(_, timer)| {
					if timer.start_condition == TimerStartCondition::OnNarrationEnd && timer.timer.paused() {
						timer.timer.unpause();
					}
				});
			});
		}
	}
}