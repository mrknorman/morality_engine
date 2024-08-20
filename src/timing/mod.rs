use std::{
    collections::HashMap, 
    time::Duration
};
use bevy::{
    prelude::*, 
    time::Timer
};

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
                TimerPallet::tick
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


#[derive(Component)]
pub struct TimerPallet{
	pub timers : HashMap<String, Timer>
}

impl TimerPallet {
	pub fn new(
		timer_vector : Vec<(String, f32)>		
	) -> Self{

		let mut timers: HashMap<String, Timer> = HashMap::new();

		for (name, duration_seconds) in timer_vector {
			timers.insert(
				name, Timer::from_seconds(duration_seconds, TimerMode::Once)
			);
		}

		Self {
			timers
		}
	}

	pub fn tick(
		time : Res<Time>,
		mut delay_query : Query<&mut TimerPallet>
	) {
		for mut delay in delay_query.iter_mut() {

			let timers = &mut delay.timers;
			for (_, timer) in timers {
				timer.tick(time.delta());
			}
        }
	}
}