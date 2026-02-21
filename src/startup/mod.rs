use bevy::{prelude::*, sprite_render::Material2dPlugin};

//#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use crate::startup::textures::DigitSheet;
use crate::systems::audio::{OneShotAudio, OneShotAudioPallet};
use crate::systems::resize::ResizePlugin;
use crate::systems::ui::discrete_slider::DiscreteSliderPlugin;
use crate::systems::ui::hover_box::HoverBoxPlugin;
use crate::systems::ui::menu::MenusPlugin;
use crate::systems::ui::scroll::ScrollPlugin;
use crate::systems::ui::window::UiWindowPlugin;
use crate::{
    data::{rng::RngPlugin, states::GameStatesPlugin, stats::StatsPlugin},
    scenes::dilemma::lever::Lever,
    shaders::PulsingMaterial,
    style::common_ui::CommonUIPlugin,
    systems::{
        colors::ColorsPlugin, inheritance::InheritancePlugin, motion::MotionPlugin,
        particles::ParticlePlugin, time::DilationPLugin,
    },
};

pub mod cursor;
pub mod debug;
pub mod pause;
pub mod render;
pub mod system_menu;
pub mod textures;

use cursor::CursorPlugin;
use render::RenderPlugin;
pub struct StartupPlugin;
impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DigitSheet>()
            .insert_resource(Lever::default())
            .add_plugins(Material2dPlugin::<PulsingMaterial>::default())
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
                DilationPLugin,
                MenusPlugin,
                debug::DebugPlugin,
                pause::PausePlugin,
            ))
            .add_plugins(DiscreteSliderPlugin)
            .add_plugins(HoverBoxPlugin)
            .add_plugins(ScrollPlugin)
            .add_systems(Startup, crt_start_up);

        if !app.is_plugin_added::<UiWindowPlugin>() {
            app.add_plugins(UiWindowPlugin);
        }

        //#[cfg(debug_assertions)]
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(LogDiagnosticsPlugin::default());
    }
}

fn crt_start_up(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(OneShotAudioPallet::new(vec![OneShotAudio {
        source: asset_server.load("./audio/effects/crt/activate.ogg"),
        dilatable: true,
        speed: 1.0,
        volume: 2.0,
        ..default()
    }]));
}
