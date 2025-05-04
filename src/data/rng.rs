use bevy::prelude::*;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use noise::Perlin;


pub struct RngPlugin;
impl Plugin for RngPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalRng::default());
    }
}

#[derive(Resource)]
pub struct GlobalRng{
    pub uniform : Pcg64Mcg,
    pub perlin : Perlin
}
 
impl Default for GlobalRng {
    fn default() -> Self {
        GlobalRng{
            uniform : Pcg64Mcg::seed_from_u64(12345),
            perlin : Perlin::new(12345)
        }
    }
}