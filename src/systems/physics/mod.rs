use bevy::{
    ecs::{component::HookContext, world::DeferredWorld}, prelude::*
};

use crate::systems::time::Dilation;

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsSystemsActive {
    #[default]
	False,
    True
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {	
		app
		.init_state::<PhysicsSystemsActive>()
		.add_systems(
			Update,
			activate_systems
		).add_systems(
            Update,
            (
				Velocity::enact,
                AngularVelocity::enact,
                Gravity::enact,
                Friction::enact
            )
            .run_if(in_state(PhysicsSystemsActive::True))
        );
    }

}

fn activate_systems(
	mut physics_state: ResMut<NextState<PhysicsSystemsActive>>,
	physics_query: Query<&Velocity>
) {
	if !physics_query.is_empty() {
		physics_state.set(PhysicsSystemsActive::True)
	} else {
		physics_state.set(PhysicsSystemsActive::False)
	}
}

#[derive(Component)]
#[require(Transform)]
pub struct Velocity(pub Vec3);

impl Default for Velocity {
    fn default() -> Self {
        Self(Vec3::ZERO)
    }    
}

impl Velocity {
    fn enact(
        time: Res<Time>,
        dilation : Res<Dilation>,
        mut query : Query<(&mut Transform, &mut Velocity)>, 
    ) {
        let duration_seconds = time.delta_secs()*dilation.0;
        for (mut transform, velocity) in query.iter_mut() {
            transform.translation += velocity.0*duration_seconds;
        }
    }
}

#[derive(Component, Default)]
#[require(Transform)]
pub struct AngularVelocity(pub Vec3); // Radians per second for rotation around X, Y, Z axes


impl AngularVelocity {
    pub fn enact(
        time: Res<Time>,
        dilation: Res<Dilation>,
        mut query: Query<(&mut Transform, &AngularVelocity)>,
    ) {
        let delta_time = time.delta_secs() * dilation.0;

        for (mut transform, angular_velocity) in query.iter_mut() {
            if angular_velocity.0 != Vec3::ZERO {
                let rotation_delta = Quat::from_euler(
                    EulerRot::XYZ,
                    angular_velocity.0.x * delta_time,
                    angular_velocity.0.y * delta_time,
                    angular_velocity.0.z * delta_time,
                );
                transform.rotation = transform.rotation * rotation_delta;
            }
        }
    }
}


#[derive(Component, Clone)]
#[require(Transform, Velocity)]
#[component(on_insert = Gravity::on_insert)]
pub struct Gravity {
    acceleration : Vec3,
    floor_level : Option<f32>,
    pub is_falling : bool
}

impl Gravity{
    fn enact(
        time: Res<Time>,
        dilation : Res<Dilation>,
        mut query : Query<(&mut Gravity, &mut Velocity, &mut Transform)>, 
    ) {
        let duration_seconds = time.delta_secs()*dilation.0;
        for (mut gravity, mut velocity, mut transform) in query.iter_mut() {

            if let Some(floor_level) = gravity.floor_level {
                if transform.translation.y > floor_level {
                    gravity.is_falling = true;
                    velocity.0 += gravity.acceleration*duration_seconds;
                } else if gravity.is_falling && transform.translation.y < floor_level {
                    gravity.is_falling = false;
                    transform.translation.y = floor_level;
                    velocity.0.y = 0.0;
                }
            }
        }
    }

    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        let transform: Option<Transform> = {
            let entity_mut = world.entity(entity);
            entity_mut.get::<Transform>()
                .map(|transform: &Transform| transform.clone())
        };

        if let Some(transform) = transform {
            if let Some(mut gravity) = world.entity_mut(entity).get_mut::<Gravity>() {
                gravity.floor_level = Some(transform.translation.y);
            } else {
                eprintln!("Warning: Entity does not contain a Gravity component!");
            }
        }
    }
}

impl Default for Gravity {
    fn default() -> Self {
        Self{
            acceleration : Vec3::new(0.0, -200.0, 1.0),
            floor_level : Some(0.0),
            is_falling : false
        }
    }

}

#[derive(Component, Clone)]
#[require(Transform, Velocity)]
#[component(on_insert = Friction::on_insert)]
pub struct Friction {
    coefficient : f32,
    floor_level : Option<f32>,
    pub is_sliding : bool
}

impl Friction{
    fn enact(
        time: Res<Time>,
        dilation: Res<Dilation>,
        mut query: Query<(
            &mut Friction,
            &mut Velocity,                 // linear velocity
            Option<&mut AngularVelocity>,   // angular velocity (optional)
            &mut Transform,
        )>,
    ) {
        let dt = time.delta_secs() * dilation.0;

        for (mut friction, mut linear_v, angular_opt, transform) in query.iter_mut() {
            if let Some(floor) = friction.floor_level {
                let on_floor = transform.translation.y <= floor;

                /* ---------- LINEAR FRICTION ---------- */
                if on_floor && linear_v.0.length_squared() > 0.0 {
                    friction.is_sliding = true;

                    let v0 = linear_v.0;                               // copy
                    linear_v.0 -= v0 * friction.coefficient * dt;       // mutate

                    if linear_v.0.length_squared() < 0.01 {
                        linear_v.0 = Vec3::ZERO;
                    }
                }

                /* ---------- ANGULAR FRICTION ---------- */
                if let Some(mut angular) = angular_opt {
                    if on_floor && angular.0.length_squared() > 0.0 {
                        let w0 = angular.0;                             // copy
                        angular.0 -= w0 * friction.coefficient * dt;     // mutate

                        if angular.0.length_squared() < 0.01 {
                            angular.0 = Vec3::ZERO;
                        }
                    }
                }

                /* ---------- RESET SLIDING FLAG ---------- */
                if linear_v.0 == Vec3::ZERO {
                    friction.is_sliding = false;
                }
            }
        }
    }

    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        let transform: Option<Transform> = {
            let entity_mut = world.entity(entity);
            entity_mut.get::<Transform>()
                .map(|transform: &Transform| transform.clone())
        };

        if let Some(transform) = transform {
            if let Some(mut friction) = world.entity_mut(entity).get_mut::<Friction>() {
                friction.floor_level = Some(transform.translation.y);
            } else {
                eprintln!("Warning: Entity does not contain a friction component!");
            }
        }
    }
}

impl Default for Friction {
    fn default() -> Self {
        Self{
            coefficient : 0.5,
            floor_level : Some(0.0),
            is_sliding : false
        }
    }
}