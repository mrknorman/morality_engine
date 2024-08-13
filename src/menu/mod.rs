use bevy::prelude::*;
use std::path::PathBuf;

use crate::{
    train::{TrainPlugin, Train, STEAM_TRAIN},
    track::Track,
    io_elements::{
        spawn_text_button, 
        NORMAL_BUTTON, 
        HOVERED_BUTTON, 
        PRESSED_BUTTON
    },
    game_states::{
        GameState, 
        MainState, 
        SubState
    },
    audio::play_sound_once,
    text::{
        TextTitle,
        TextComponent,
        TextRaw
    }
};

const MAIN_MENU: MainState = MainState::Menu;

#[derive(Component, Resource)]
pub struct MainMenu;

#[derive(Resource)]
pub struct MenuData {
    button_entity: Entity
}

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MAIN_MENU), setup_menu)
            .add_systems(
                Update,
                menu.run_if(in_state(MAIN_MENU)),
            )
            .add_systems(OnExit(MAIN_MENU), cleanup_menu)
            .add_plugins(TrainPlugin::new(MAIN_MENU));
    }
}


fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {

    let text = include_str!("main_menu.txt");

    let menu_entity = commands.spawn(
        (
            MainMenu,
            TransformBundle::from_transform(
                Transform::from_xyz(0.0, 0.0, 0.0)
            ),
            VisibilityBundle::default()
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
    // TO DO BACKGROUND NOISE: PathBuf::from("./sounds/static.ogg") and ./sounds/office.ogg

    spawn_train(&mut commands, &asset_server, menu_entity);
    let button_entity = spawn_button(&mut commands);

    commands.insert_resource(MenuData {
        button_entity
    });
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
    track.spawn(commands);

    let train = Train::new(
		STEAM_TRAIN,
        train_translation,
        0.0,
    );
    train.spawn(commands, asset_server, Some(parent));
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
            (Changed<Interaction>, With<Button>),
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
}