use bevy::{
    prelude::*, 
    sprite::Material2dPlugin
};

//#[cfg(debug_assertions)]
use bevy::diagnostic::{
    LogDiagnosticsPlugin, 
    FrameTimeDiagnosticsPlugin
};

use crate::{
    colors::ColorsPlugin, 
    common_ui::CommonUIPlugin, 
    scenes::dilemma::lever::Lever, 
    game_states::GameStatesPlugin, 
    inheritance::InheritancePlugin, 
    motion::MotionPlugin, 
    shaders::PulsingMaterial, 
    stats::StatsPlugin, 
    time::DilationPLugin
};

pub mod shortcuts;
pub mod render;
pub mod rng;
pub mod cursor;

use render::RenderPlugin;
use rng::RngPlugin;
use cursor::CursorPlugin;

pub struct StartupPlugin;
impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(
                Lever::default()
            )
            .add_plugins((
                Material2dPlugin::<PulsingMaterial>::default(),
                bevy_svg::prelude::SvgPlugin
            ))
            .add_plugins((
                CursorPlugin,
                RenderPlugin,
                RngPlugin,
                GameStatesPlugin,
                StatsPlugin,
                CommonUIPlugin,
                MotionPlugin,
                ColorsPlugin,
                InheritancePlugin,
                DilationPLugin
            ))
            .add_systems(
                Update, (
                shortcuts::close_on_esc, 
            ));

            //#[cfg(debug_assertions)]
            app
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_plugins(LogDiagnosticsPlugin::default());
    }
}
