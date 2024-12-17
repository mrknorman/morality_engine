use bevy::prelude::*;

#[derive(Component)]
pub struct Velocity(Vec3);

impl Default for Velocity {
    fn default() -> Self {
        Self(Vec3::default())
    }    
}

#[derive(Component)]
#[require(Velocity)]
pub struct GraviationalMass {
    floor_level : f32
}