use std::{
	collections::HashMap,
	time::Duration
};
use bevy::{
    prelude::*, 
    time::Timer
};

use crate::audio::NarrationAudioFinished;

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TimingSystemsActive {
    #[default]
    False,
    True
}

pub struct TimingPlugin;
impl Plugin for TimingPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_state::<TimingSystemsActive>()
        .add_systems(
            Update,
            activate_systems
        ).add_systems(
            Update,
            (
                TimerPallet::tick,
				TimerPallet::check_start_conditions
            )
            .run_if(
                in_state(TimingSystemsActive::True)
            )
        );
    }
}

fn activate_systems(
	mut audio_state: ResMut<NextState<TimingSystemsActive>>,
	delay_query: Query<&TimerPallet>,
) {
	if !delay_query.is_empty() {
		audio_state.set(TimingSystemsActive::True)
	} else {
		audio_state.set(TimingSystemsActive::False)
	}
}

#[derive(PartialEq)]
pub enum TimerStartCondition{
    Immediate,
    OnNarrationEnd
}

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
pub struct TimerPallet{
	pub timers : HashMap<String, ConditionalTimer>
}

impl TimerPallet {
	pub fn new(
		timer_vector : Vec<(String, TimerConfig)>		
	) -> Self {

		let mut timers: HashMap<String, ConditionalTimer> = HashMap::new();

		for (
			name, 
			config
		) in timer_vector {

			timers.insert(
				name, ConditionalTimer::new(config)
			);
		}

		Self {
			timers
		}
	}

	pub fn tick(
		time : Res<Time>,
		mut timer_query : Query<&mut TimerPallet>
	) {
		for mut pallet in timer_query.iter_mut() {

			let timers = &mut pallet.timers;
			for (_, timer) in timers {
				timer.timer.tick(time.delta());
				timer.count_completions();
				timer.start_repeat_timer();
			}
        }
	}

	pub fn check_start_conditions(
		mut ev_narration_finished: EventReader<NarrationAudioFinished>,
		mut timer_query : Query<&mut TimerPallet>
	) {

		for _ in ev_narration_finished.read() {
			for mut pallet in timer_query.iter_mut() {
				let timers = &mut pallet.timers;
				for (_, timer) in timers {
					if timer.start_condition == TimerStartCondition::OnNarrationEnd {
						if timer.timer.paused() {
							timer.timer.unpause();
						}
					}
				}
			}
		}
	}
}