use bevy::prelude::*;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;

pub struct RngPlugin;
impl Plugin for RngPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalRng::default());
    }
}

#[derive(Resource)]
pub struct GlobalRng(pub Pcg64Mcg);
 
impl Default for GlobalRng {
    fn default() -> Self {
        GlobalRng(Pcg64Mcg::seed_from_u64(12345))
    }
}