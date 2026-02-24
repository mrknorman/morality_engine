use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
};
mod data;
mod entities;
#[forbid(unsafe_code)]
mod scenes;
mod shaders;
mod startup;
mod style;
mod systems;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("The Trolley Algorithm"),
                resizable: true,
                present_mode: PresentMode::Immediate,
                resolution: WindowResolution::new(1280, 960),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run();
}

struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((startup::StartupPlugin, scenes::ScenePlugin));
    }
}
