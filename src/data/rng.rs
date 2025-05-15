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

use rand_core::RngCore;

impl RngCore for GlobalRng {
    #[inline] fn next_u32(&mut self) -> u32 { self.uniform.next_u32() }
    #[inline] fn next_u64(&mut self) -> u64 { self.uniform.next_u64() }
    #[inline] fn fill_bytes(&mut self, dest: &mut [u8])       { self.uniform.fill_bytes(dest) }
}