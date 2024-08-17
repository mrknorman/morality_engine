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
        TextButton, 
        TextComponent, 
        TextRaw, 
        TextTitle
    }, 
    track::Track, 
    train::{
        Train, 
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
        TextTitle::new(
                    text,
                    Vec3::new(0.0, 150.0, 1.0), 
                )
            );
            parent.spawn(
                TextRaw::new(
                    "A game by Michael Norman",
                    Vec3::new(0.0, 10.0, 1.0)
                )
            );
        }
    ).id();

    let state_vector = StateVector::new(
        Some(MainState::InGame),
        Some(GameState::Loading),
        None
    );

    spawn_train(&mut commands, &asset_server, entity);
    
    TextButton::new(
        &mut commands, 
        &asset_server, 
        Some(entity),  
        vec![
            InputAction::PlaySound("click".to_string()),
            InputAction::ChangeState(state_vector)
        ],
        vec![KeyCode::Enter],
        "[Click here or Press Enter to Begin]".to_string(),
        Vec3::new(0.0,-150.0,1.0)
    );

    // Add background audio:
    let mut entity_commands = commands.entity(entity);
    
    //let button_entity = spawn_button(&mut commands);
    commands.insert_resource(MenuData{entity});
}

fn spawn_train(
        commands: &mut Commands,  
        asset_server: &Res<AssetServer>,
        parent : Entity
    ) {

    let train_translation: Vec3 = Vec3::new(50.0, 30.0, 1.0);
    let track_displacement: Vec3 = Vec3::new(-45.0, 0.0, 1.0);
    let track_translation: Vec3 = train_translation + track_displacement;

    let track : Track = Track::new(50, Color::WHITE, track_translation);
    track.spawn(commands, Some(parent));

    let train = Train::new(
		STEAM_TRAIN,
        train_translation,
        0.0,
    );
    train.spawn(commands, asset_server, Some(parent));
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.entity).despawn_recursive();
}