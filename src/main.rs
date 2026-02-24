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
    if let Some(export_path) = campaign_graph_export_path_from_args() {
        match scenes::flow::visualize::export_campaign_graph_html(&export_path) {
            Ok(summary) => {
                println!(
                    "Campaign graph viewer exported to {} (routes: {}, validation errors: {})",
                    export_path,
                    summary.route_count,
                    summary.validation_error_count
                );
            }
            Err(error) => {
                eprintln!(
                    "Failed to export campaign graph viewer to {}: {}",
                    export_path, error
                );
                std::process::exit(1);
            }
        }
        return;
    }

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

fn campaign_graph_export_path_from_args() -> Option<String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let export_flag_index = args.iter().position(|arg| arg == "--export-campaign-graph")?;
    Some(
        args.get(export_flag_index + 1)
            .cloned()
            .unwrap_or_else(|| String::from("docs/campaign_graph_view.html")),
    )
}

struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((startup::StartupPlugin, scenes::ScenePlugin));
    }
}
