use bevy::prelude::*;

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
                Gravity::enact
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
        mut query : Query<(&mut Transform, &mut Velocity)>, 
    ) {
        let duration_seconds = time.delta().as_secs_f32();
        for (mut transform, velocity) in query.iter_mut() {
            transform.translation += velocity.0*duration_seconds;
        }
    }
}

#[derive(Component)]
#[require(Velocity)]
pub struct Gravity {
    acceleration : Vec3,
    floor_level : Option<f32>,
    pub is_falling : bool
}

impl Gravity{
    fn enact(
        time: Res<Time>,
        mut query : Query<(&mut Gravity, &mut Velocity, &mut Transform)>, 
    ) {
        let duration_seconds = time.delta().as_secs_f32();
        for (mut gravity, mut velocity, mut transform) in query.iter_mut() {

            if let Some(floor_level) = gravity.floor_level {
                if transform.translation.y > floor_level {
                    gravity.is_falling = true;
                    velocity.0 += gravity.acceleration*duration_seconds;
                } else if gravity.is_falling && transform.translation.y < floor_level {
                    //println!("Not falling!");
                    gravity.is_falling = false;
                    transform.translation.y = floor_level;
                    velocity.0 = Vec3::ZERO;
                }
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