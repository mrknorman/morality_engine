use bevy::prelude::*;
use std::path::PathBuf;

use crate::train::{TrainEntities, Train, TrainWhistle, STEAM_TRAIN};
use crate::io_elements::{spawn_text_button, NORMAL_BUTTON, HOVERED_BUTTON, PRESSED_BUTTON};
use crate::game_states::{GameState, MainState, SubState};
use crate::audio::play_sound_once;

const MAIN_MENU: MainState = MainState::Menu;

#[derive(Resource)]
pub struct MenuData {
    title_entity: Entity,
    train_entity: TrainEntities,
    train_audio: Entity,
    signature_entity: Entity,
    button_entity: Entity,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MAIN_MENU), setup_menu)
            .add_systems(
                Update,
                (menu, Train::whistle, Train::wobble).run_if(in_state(MAIN_MENU)),
            )
            .add_systems(OnExit(MAIN_MENU), cleanup_menu);
    }
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let title_entity = spawn_title(&mut commands, &asset_server);
    let train_audio = spawn_train_audio(&mut commands, &asset_server);
    let train_entity = spawn_train(&mut commands);
    let signature_entity = spawn_signature(&mut commands, &asset_server);
    let button_entity = spawn_button(&mut commands);

    commands.insert_resource(MenuData {
        title_entity,
        train_entity,
        train_audio,
        signature_entity,
        button_entity,
    });
}

fn spawn_title(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    let ascii_art = include_str!("main_menu.txt");
    commands
        .spawn((
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection::new(
                        ascii_art,
                        TextStyle {
                            font_size: 12.0,
                            ..default()
                        },
                    )],
                    justify: JustifyText::Center,
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 150.0, 1.0),
                ..default()
            },
            AudioBundle {
                source: asset_server.load(PathBuf::from("./sounds/static.ogg")),
                settings: PlaybackSettings {
                    paused: false,
                    volume: bevy::audio::Volume::new(0.06),
                    mode: bevy::audio::PlaybackMode::Loop,
                    ..default()
                },
            },
        ))
        .id()
}

fn spawn_train_audio(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    commands
        .spawn(AudioBundle {
            source: asset_server.load(PathBuf::from("./sounds/train_loop.ogg")),
            settings: PlaybackSettings {
                paused: false,
                volume: bevy::audio::Volume::new(0.1),
                mode: bevy::audio::PlaybackMode::Loop,
                ..default()
            },
        })
        .id()
}

fn spawn_train(commands: &mut Commands) -> TrainEntities {

    let train = Train::new(
        Some(STEAM_TRAIN.engine.to_string()),
		STEAM_TRAIN.carriages.iter().map(|&s| s.to_string()).collect(),
		STEAM_TRAIN.smoke.as_ref().map(|sm| sm.iter().map(|&s| s.to_string()).collect()).unwrap(),
        Vec3::new(100.0, 0.0, 1.0),
        50.0,
    );
    train.spawn(commands)
}

fn spawn_signature(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    commands
        .spawn((
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection::new(
                        "A game by Michael Norman",
                        TextStyle {
                            font_size: 12.0,
                            ..default()
                        },
                    )],
                    justify: JustifyText::Center,
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 10.0, 1.0),
                ..default()
            },
            AudioBundle {
                source: asset_server.load(PathBuf::from("./sounds/office.ogg")),
                settings: PlaybackSettings {
                    paused: false,
                    volume: bevy::audio::Volume::new(1.0),
                    mode: bevy::audio::PlaybackMode::Loop,
                    ..default()
                },
            },
        ))
        .id()
}

fn spawn_button(commands: &mut Commands) -> Entity {
    spawn_text_button(
        "[Click here or Press Enter to Begin]",
        Some(MainState::InGame),
        Some(GameState::Dialogue),
        Some(SubState::None),
        0.0,
        commands,
    )
}

fn startup_effects(
    next_main_state: &mut ResMut<NextState<MainState>>,
    next_game_state: &mut ResMut<NextState<GameState>>,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    play_sound_once("sounds/mech_click.ogg", commands, asset_server);
    next_main_state.set(MainState::InGame);
    next_game_state.set(GameState::Loading);
}

fn menu(
    mut next_main_state: ResMut<NextState<MainState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&Children, &Interaction),
        (Changed<Interaction>, With<Button>, Without<TrainWhistle>),
    >,
    mut text_query: Query<&mut Text>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (children, interaction) in &mut interaction_query {
        if let Some(&text_entity) = children.first() {
            if let Ok(mut text) = text_query.get_mut(text_entity) {
                text.sections[0].style.color = match *interaction {
                    Interaction::Pressed => {
                        startup_effects(
                            &mut next_main_state,
                            &mut next_game_state,
                            &mut commands,
                            &asset_server,
                        );
                        PRESSED_BUTTON
                    }
                    Interaction::Hovered => HOVERED_BUTTON,
                    Interaction::None => NORMAL_BUTTON,
                };
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::Enter) {
        startup_effects(
            &mut next_main_state,
            &mut next_game_state,
            &mut commands,
            &asset_server,
        );
    }
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn_recursive();
    commands.entity(menu_data.title_entity).despawn_recursive();
    commands.entity(menu_data.signature_entity).despawn_recursive();
    commands.entity(menu_data.train_entity.train).despawn_recursive();
    commands.entity(menu_data.train_audio).despawn_recursive();

    if let Some(engine) = menu_data.train_entity.engine {
        commands.entity(engine).despawn_recursive();
    }
}