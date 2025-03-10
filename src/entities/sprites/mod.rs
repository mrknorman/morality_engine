use bevy::prelude::*;

pub mod compound;
pub mod window;

use window::WindowPlugin;
use compound::CompoundPlugin;

pub struct SpritePlugin;
impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins((CompoundPlugin, WindowPlugin));
    }
}