use bevy::{
    prelude::*, 
    sprite_render::Material2dPlugin
};

//#[cfg(debug_assertions)]
use bevy::diagnostic::{
    LogDiagnosticsPlugin, 
    FrameTimeDiagnosticsPlugin
};

use crate::startup::textures::DigitSheet;
use crate::systems::audio::{OneShotAudio, OneShotAudioPallet};
use crate::systems::resize::ResizePlugin;
use crate::{
    systems::{
        colors::ColorsPlugin, 
        inheritance::InheritancePlugin, 
        motion::MotionPlugin,
        particles::ParticlePlugin,
        time::DilationPLugin
    },
    data::{
        stats::StatsPlugin, 
        states::GameStatesPlugin, 
        rng::RngPlugin
    },
    style::common_ui::CommonUIPlugin, 
    scenes::dilemma::lever::Lever, 
    shaders::PulsingMaterial
};

pub mod shortcuts;
pub mod render;
pub mod cursor;
pub mod textures;

use render::RenderPlugin;
use cursor::CursorPlugin;
pub struct StartupPlugin;
impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DigitSheet>()
            .insert_resource(
                Lever::default()
            )
            .add_plugins(
                Material2dPlugin::<PulsingMaterial>::default(),
            )
            .add_plugins((
                ParticlePlugin,
                CursorPlugin,
                RenderPlugin,
                RngPlugin,
                ResizePlugin,
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
            ))
            .add_systems(
                Startup,
                crt_start_up
            );

            //#[cfg(debug_assertions)]
            app
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(LogDiagnosticsPlugin::default());
    }
}


fn crt_start_up(
    mut commands : Commands,
    asset_server : Res<AssetServer>
    ) {

    commands.spawn(
OneShotAudioPallet::new(
            vec![
                OneShotAudio{
                    source : asset_server.load(
                        "./audio/effects/crt/activate.ogg"
                    ),
                    dilatable : true,
                    speed : 1.0,
                    volume : 2.0,
                    ..default()
                }
            ]
        )
    );
}
