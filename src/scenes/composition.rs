use bevy::prelude::*;

use crate::{
    entities::{
        large_fonts::AsciiPlugin, sprites::SpritePlugin, text::TextPlugin, train::TrainPlugin,
    },
    style::ui::IOPlugin,
    systems::{
        backgrounds::BackgroundPlugin, cascade::CascadePlugin, interaction::InteractionPlugin,
        scheduling::TimingPlugin,
    },
};

pub struct SceneCompositionPlugin;

impl Plugin for SceneCompositionPlugin {
    fn build(&self, app: &mut App) {
        ensure_plugin(app, InteractionPlugin);
        ensure_plugin(app, IOPlugin);
        ensure_plugin(app, TimingPlugin);
        ensure_plugin(app, TrainPlugin);
        ensure_plugin(app, BackgroundPlugin);
        ensure_plugin(app, AsciiPlugin);
        ensure_plugin(app, SpritePlugin);
        ensure_plugin(app, TextPlugin);
        ensure_plugin(app, CascadePlugin);
    }
}

fn ensure_plugin<T: Plugin>(app: &mut App, plugin: T) {
    if !app.is_plugin_added::<T>() {
        app.add_plugins(plugin);
    }
}
