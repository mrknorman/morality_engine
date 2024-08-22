use bevy::prelude::*;

use crate::{
    audio::{
        ContinuousAudio,
        ContinuousAudioBundle,
        ContinuousAudioPallet,
        MusicAudio,
        BackgroundAudio
    }, 
    game_states::{
        GameState, 
        MainState, 
        StateVector
    }, 
    interaction::{
        InteractionPlugin,
        InputAction
    }, 
    text::{
        TextButtonBundle, 
        TextRawBundle, 
        TextTitleBundle
    }, 
    track::TrackBundle, 
    train::{
        TrainPlugin,
        TrainBundle, 
        STEAM_TRAIN
    },
    io::{
        IOPlugin,
        BottomAnchor
    }
};

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MainState::Menu), setup_menu
        );
        if !app.is_plugin_added::<TrainPlugin>() {
            app.add_plugins(TrainPlugin);
        }
        if !app.is_plugin_added::<InteractionPlugin>() {
            app.add_plugins(InteractionPlugin);
        }
        if !app.is_plugin_added::<IOPlugin>() {
            app.add_plugins(IOPlugin);
        }
    }
}

#[derive(Component)]
pub struct MenuRoot;

fn setup_menu(
        mut commands: Commands, 
        asset_server: Res<AssetServer>,
        windows: Query<&Window>
    ) {

    let text = include_str!("main_menu.txt");

    let menu_translation : Transform = Transform::from_xyz(0.0, 0.0, 0.0);
    let title_translation : Vec3 = Vec3::new(0.0, 150.0, 1.0);
    let train_translation: Vec3 = Vec3::new(50.0, 30.0, 1.0);
    let track_displacement: Vec3 = Vec3::new(-45.0, 0.0, 1.0);
    let track_translation: Vec3 = train_translation + track_displacement;
    let signature_translation : Vec3 = Vec3::new(0.0, 10.0, 1.0);

    let button_distance = 100.0;
    let window = windows.get_single().unwrap();
    let screen_height = window.height();
    let button_y = -screen_height / 2.0 + button_distance; 
    let button_translation: Vec3 = Vec3::new(0.0, button_y, 1.0);

    let next_state_vector = StateVector::new(
        Some(MainState::InGame),
        Some(GameState::Loading),
        None
    );

    commands.spawn(
        (
            MenuRoot,
            StateScoped(MainState::Menu),
            TransformBundle::from_transform(menu_translation),
            VisibilityBundle::default()
        )
    ).with_children(
        |parent| {

            parent.spawn((
                BackgroundAudio,
                ContinuousAudioPallet::new(
                    vec![
                        (
                            "static".to_string(),
                            ContinuousAudio::new(
                                &asset_server, 
                                "./sounds/static.ogg", 
                                0.1
                            ),
                        ),
                        (
                            "office".to_string(),
                            ContinuousAudio::new(
                                &asset_server, 
                                "./sounds/office.ogg", 
                                1.0
                            ),
                        )
                    ]
                )
            ));

            parent.spawn((
                MusicAudio,
                ContinuousAudioBundle::new(
                    &asset_server, 
                    "./music/the_last_decision.ogg", 
                    0.3
                )
            ));

            parent.spawn(
                TextTitleBundle::new(
                    text,
                    title_translation, 
                )
            );
            parent.spawn(
                TrainBundle::new(
                    &asset_server,
                    STEAM_TRAIN,
                    train_translation,
                    0.0
                )
            );
            parent.spawn(
                TrackBundle::new(
                    50, 
                    track_translation
                )
            );
            parent.spawn(
                TextRawBundle::new(
                    "A game by Michael Norman",
                    signature_translation
                )
            );
            parent.spawn(
                (
                    BottomAnchor::new(button_distance),
                    TextButtonBundle::new(
                        &asset_server, 
                        vec![
                            InputAction::PlaySound(String::from("click")),
                            InputAction::ChangeState(next_state_vector)
                        ],
                        vec![KeyCode::Enter],
                        "[Click Here or Press Enter to Begin]",
                        button_translation
                    )
                )
            );
        }
    );
}