use bevy::prelude::*;

use crate::{
    audio::{
        ContinuousAudio,
        ContinuousAudioPallet
    }, 
    game_states::{
        GameState, 
        MainState, 
        StateVector
    }, 
    interaction::{
        InputAction, 
        InteractionPlugin
    }, 
    text::{
        TextButtonBundle, 
        TextRawBundle, 
        TextTitleBundle
    }, 
    track::TrackBundle, 
    train::{
        TrainBundle, 
        TrainPlugin,
        STEAM_TRAIN
    }
};

const MAIN_MENU: MainState = MainState::Menu;

#[derive(Component)]
pub struct MainMenu;

#[derive(Resource)]
pub struct MenuData {
    entity: Entity
}

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MAIN_MENU), setup_menu)
            .add_systems(OnExit(MAIN_MENU), cleanup_menu)
            .add_plugins(TrainPlugin::new(MAIN_MENU));

        if !app.is_plugin_added::<InteractionPlugin<MainState>>() {
            app.add_plugins(InteractionPlugin::new(MAIN_MENU));
        }
    }
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {

    let text = include_str!("main_menu.txt");

    let train_translation: Vec3 = Vec3::new(50.0, 30.0, 1.0);
    let track_displacement: Vec3 = Vec3::new(-45.0, 0.0, 1.0);
    let track_translation: Vec3 = train_translation + track_displacement;
    
    let state_vector = StateVector::new(
        Some(MainState::InGame),
        Some(GameState::Loading),
        None
    );

    let entity = commands.spawn(
        (
            MainMenu,
            TransformBundle::from_transform(
                Transform::from_xyz(0.0, 0.0, 0.0)
            ),
            VisibilityBundle::default(),
            ContinuousAudioPallet::new(
                vec![
                    (
                        "static".to_string(),
                        ContinuousAudio::new(
                            &asset_server, 
                            "./sounds/static.ogg", 
                            0.05
                        ),
                    ),
                    (
                        "office".to_string(),
                        ContinuousAudio::new(
                            &asset_server, 
                            "./sounds/office.ogg", 
                            1.0
                        ),
                    ),
                    (
                        "danse".to_string(),
                        ContinuousAudio::new(
                            &asset_server, 
                            "./music/danse.ogg", 
                            0.3
                        ),
                    )
                ]
            )
        )
    ).with_children(
        |parent| {
            parent.spawn(
                TextTitleBundle::new(
                    text,
                    Vec3::new(0.0, 150.0, 1.0), 
                )
            );
            parent.spawn(
                TextRawBundle::new(
                    "A game by Michael Norman",
                    Vec3::new(0.0, 10.0, 1.0)
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
                TextButtonBundle::new(
                    &asset_server, 
                    vec![
                        InputAction::PlaySound("click".to_string()),
                        InputAction::ChangeState(state_vector)
                    ],
                    vec![KeyCode::Enter],
                    "[Click here or Press Enter to Begin]".to_string(),
                    Vec3::new(0.0,-150.0,1.0)
                )
            );
        }
    ).id();
        
    //let button_entity = spawn_button(&mut commands);
    commands.insert_resource(MenuData{entity});
}


fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.entity).despawn_recursive();
}