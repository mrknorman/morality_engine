use bevy::prelude::*;
use rand::Rng;
use rand_pcg::Pcg64Mcg;

#[derive(Resource)]
struct MyGlobalRng(pub Pcg64Mcg);
