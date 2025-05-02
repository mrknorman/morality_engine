use std::time::Duration;

use bevy::{
	ecs::component::{Mutable, StorageType}, prelude::*
};
use rand::Rng;
use crate::{
	systems::{
		physics::{
			Gravity, 
			PhysicsPlugin, 
			Velocity
		}, 
		time::Dilation, 
	},
	data::rng::GlobalRng
};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MotionSystemsActive {
    #[default]
	False,
    True
}

pub struct MotionPlugin;

impl Plugin for MotionPlugin {
    fn build(&self, app: &mut App) {	
		app
		.init_state::<MotionSystemsActive>()
		.add_systems(
			Update,
			activate_systems
		).add_systems(
            Update,
            (
				Wobble::enact,
				Bounce::enact,
				Pulse::enact,
                PointToPointTranslation::enact
            )
            .run_if(in_state(MotionSystemsActive::True))
        )
		.register_required_components::<TransformAnchor, Transform>()
		;

		if !app.is_plugin_added::<PhysicsPlugin>() {
			app.add_plugins(PhysicsPlugin);
		}
    }
}

fn activate_systems(
    mut state: ResMut<NextState<MotionSystemsActive>>,
    query: Query<(), Or<(
        With<Bounce>,
        With<Wobble>,
        With<PointToPointTranslation>,
        With<Pulse>
    )>>
) {
    if !query.is_empty() {
        state.set(MotionSystemsActive::True)
    } else {
        state.set(MotionSystemsActive::False)
    }
}

pub struct PointToPointTranslation {
    pub initial_position: Vec3,
    pub final_position: Vec3,
	pub timer : Timer
}

impl Component for PointToPointTranslation {
    const STORAGE_TYPE: StorageType = StorageType::Table;
	type Mutability = Mutable;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_insert(
            |mut world, context| {
                let transform: Option<Transform> = {
                    let entity_mut = world.entity(context.entity);
                    entity_mut.get::<Transform>().cloned()
                };

                match transform {
                    Some(transform) => {
                        if let Some(mut transform_anchor) = world.entity_mut(context.entity).get_mut::<PointToPointTranslation>() {
                            transform_anchor.initial_position = transform.translation;
                        } else {
                            warn!(
								"Failed to retrieve TransformAnchor component for entity: {:?}", 
								context.entity
							);
                        }
                    }
                    None => {
                        warn!(

							"Transform anchor should be inserted after transform! Unable to find Transform.");
                    }
                }
            }
        );
    }
}

impl PointToPointTranslation {
    pub fn new(final_position: Vec3, duration: Duration, paused : bool) -> PointToPointTranslation {
		let mut translation = PointToPointTranslation {
			initial_position : Vec3::default(),
			final_position,
			timer : Timer::new(
				duration,
				TimerMode::Once
			)
		};

		if paused {
			translation.timer.pause();
		}
		translation
	}

	pub fn enact(
		time: Res<Time>, 
		dilation : Res<Dilation>,
		mut query: Query<(&mut PointToPointTranslation, &mut Transform)>
	) {
		for (mut motion, mut transform) in query.iter_mut() {

			motion.timer.tick(time.delta().mul_f32(dilation.0));

			if !motion.timer.paused() && !motion.timer.finished() {

				let fraction_complete = motion.timer.fraction();
				let difference = motion.final_position - motion.initial_position;
				transform.translation = motion.initial_position + difference*fraction_complete;
			} else if motion.timer.just_finished() {
				transform.translation = motion.final_position;
			}
		}
	}

	pub fn start(&mut self) {
		self.timer.unpause();
	}
}

#[derive(Clone)]
pub struct TransformAnchor(pub Transform);
impl Default for TransformAnchor {
    fn default() -> Self {
        Self(Transform::default())
    }
}

#[derive(Clone, Component)]
pub struct TransformMultiAnchor(pub Vec<Transform>);
impl Default for TransformMultiAnchor {
    fn default() -> Self {
        Self(vec![Transform::default()])
    }
}

impl Component for TransformAnchor {
    const STORAGE_TYPE: StorageType = StorageType::Table;
	type Mutability = Mutable;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_insert(
            |mut world, context| {
                let transform: Option<Transform> = {
                    let entity_mut = world.entity(context.entity);
                    entity_mut.get::<Transform>().cloned()
                };

                match transform {
                    Some(transform) => {
                        if let Some(mut transform_anchor) = world.entity_mut(context.entity).get_mut::<TransformAnchor>() {
                            transform_anchor.0.clone_from(&transform);
                        } else {
                            warn!("Failed to retrieve TransformAnchor component for entity: {:?}", context.entity);
                        }
                    }
                    None => {
                        warn!("Transform anchor should be inserted after transform! Unable to find Transform.");
                    }
                }
            }
        );
    }
}

#[derive(Component, Clone)]
#[require(TransformAnchor)]
pub struct Wobble{
	pub timer: Timer
}

impl Default for Wobble {
    fn default() -> Self {
        Wobble::new()
    }
}

impl Wobble {
	pub fn new() -> Wobble {
		Wobble{
			timer : Timer::new(
				Duration::from_millis(100), 
				TimerMode::Repeating
			)
		}
	}

	pub fn enact(
		time: Res<Time>, // Inject the Time resource to access the game time
		dilation : Res<Dilation>,
		mut rng : ResMut<GlobalRng>,
		mut wobble_query: Query<(&mut Transform, &mut Wobble, &TransformAnchor)>
	) {
		for (mut transform, mut wobble, translation_anchor) in wobble_query.iter_mut() {
			if wobble.timer.tick(time.delta().mul_f32(dilation.0)).finished() {
				let dx = rng.uniform.gen_range(-1.0..=1.0);
				let dy = rng.uniform.gen_range(-1.0..=1.0);  

				transform.translation.x = translation_anchor.0.translation.x + dx as f32;
				transform.translation.y = translation_anchor.0.translation.y + dy as f32;
			}
		}
	}
}

#[derive(Component)]
#[require(Gravity)]
pub struct Bounce {
	pub active : bool,
	pub enacting : bool,
	pub initial_velocity_min : f32,
	pub initial_velocity_max : f32,
	pub interval_min : f32,
	pub interval_max : f32,
	pub timer : Timer
}

impl Bounce {
	pub fn new(
		active :  bool,
		initial_velocity_min : f32,
		initial_velocity_max : f32,
		interval_min : f32,
		interval_max : f32
	) -> Self {

		Self {
			active,
			enacting : false,
			initial_velocity_min,
			initial_velocity_max,
			interval_min,
			interval_max,
			timer : Timer::from_seconds(
				rand::random::<f32>() + 0.5,
				TimerMode::Once
			)
		}
	}

    pub fn enact(
		time: Res<Time>,
		mut rng : ResMut<GlobalRng>,
		mut query: Query<(
			&mut Bounce,
			&mut Velocity,
			&Gravity
		)>
	) {
		for (mut bounce, mut velocity, gravity) in query.iter_mut() {

			if bounce.enacting || !bounce.active {
				bounce.timer.pause();
			} else {
				bounce.timer.unpause();
			}

			bounce.timer.tick(time.delta());
			if bounce.enacting && !gravity.is_falling {
				let new_duration_seconds = rand::random::<f32>()
				* (bounce.interval_max - bounce.interval_min) 
				+ bounce.interval_min;

				bounce.timer.set_duration(
					Duration::from_secs_f32(new_duration_seconds)
				);
				bounce.timer.reset();
			} else if bounce.timer.just_finished() && !gravity.is_falling && bounce.active {
				velocity.0 = Vec3::new(
					0.0, 
					rng.uniform.gen_range(
						bounce.initial_velocity_min..bounce.initial_velocity_max
					), 
					0.0
				);
			}

			bounce.enacting = bounce.timer.finished() && gravity.is_falling;
		}
	}
}

impl Default for Bounce {
	fn default() -> Self {
		Bounce::new(
			true,
			40.0, 
			60.0,
			1.0,
			2.0
		)	
	}
}

#[derive(Component)]
#[require(TransformAnchor)]
pub struct Pulse{
	pub active : bool,
	pub enacting : bool,
	diverging : bool,
	scale : f32,
	pub interval_timer : Timer,
	pub pulse_timer : Timer,
}

impl Pulse {
	pub fn new(
		active : bool,
		scale : f32,
		interval: Duration, 
		duration : Duration
	) -> Self {

		let half_duration = duration.div_f32(2.0);
		let separation_duration = interval - duration;

		Self {
			active,
			enacting : false,
			diverging : true,
			scale,
			interval_timer : Timer::new(
				separation_duration,
				TimerMode::Once
			),
			pulse_timer : Timer::new(
				half_duration,
				TimerMode::Repeating
			)
		}
	}

	pub fn enact(
		time : Res<Time>,
		dilation : Res<Dilation>,
		mut query : Query<(&mut Self, &mut Transform, &mut TransformAnchor)>
	) {

		for (
			mut pulse, 
			mut transform, 
			anchor
		) in query.iter_mut() {

			if pulse.enacting || !pulse.active {
				pulse.interval_timer.pause();
			} else {
				pulse.interval_timer.unpause();
			}

			let time_delta = time.delta().mul_f32(dilation.0);

			pulse.interval_timer.tick(time_delta);
			pulse.pulse_timer.tick(time_delta);

			if pulse.enacting {
				if pulse.pulse_timer.just_finished() {
					if !pulse.diverging {	
						pulse.pulse_timer.pause();
						pulse.interval_timer.reset();
						pulse.enacting = false;
					}

					pulse.diverging = !pulse.diverging;
				}

				if pulse.diverging {
					transform.scale = anchor.0.scale * (1.0 + pulse.scale * pulse.pulse_timer.fraction());
				} else {
					transform.scale = anchor.0.scale * (1.0 + pulse.scale * pulse.pulse_timer.fraction_remaining());
				};
			} else {
				transform.scale = anchor.0.scale;
			}

			if pulse.interval_timer.just_finished() {
				pulse.enacting = true;
				pulse.interval_timer.pause();
				pulse.pulse_timer.reset();
				pulse.pulse_timer.unpause();
			}
		}
	}
} 

impl Default for Pulse {
	fn default() -> Self {
		Pulse::new(
			true,
			0.2,
			Duration::from_secs_f32(1.0),
			Duration::from_secs_f32(0.4)
		)	
	}
}