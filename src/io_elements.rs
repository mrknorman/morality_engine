use bevy::{prelude::*, time::Timer};

use crate::{
    audio::play_sound_once,
    game_states::{
        GameState, 
        MainState, 
        SubState,
        StateVector
    },
    interaction::{
        NORMAL_BUTTON,
        HOVERED_BUTTON,
        PRESSED_BUTTON
    }
};

#[derive(Component)]
pub struct TextButton{
    timer : Timer,
    next_main_state : Option<MainState>,
    next_game_state : Option<GameState>,
    next_sub_state : Option<SubState>
}

pub fn spawn_text_button (
        text : &str,
        next_main_state : Option<MainState>,
        next_game_state : Option<GameState>,
        next_sub_state : Option<SubState>,
        wait_timer_seconds : f32,
        commands: &mut Commands
    ) -> Entity {

        let visibility: Visibility = if wait_timer_seconds == 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        commands
            .spawn(NodeBundle {
                style: Style {
                    // center button
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect {
                        top : Val::Px(50.0),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn((ButtonBundle {
                        style: Style {
                            width: Val::Px(278.),
                            height: Val::Px(30.),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        visibility,
                        background_color : bevy::prelude::BackgroundColor(
                            Color::srgb(0.0, 0.0, 0.0)
                        ),
                        ..default()
                    },
                    TextButton{
                        timer : Timer::from_seconds(
                            wait_timer_seconds, 
                            TimerMode::Once
                        ),
                        next_game_state,
                        next_main_state,
                        next_sub_state
                    }))
                    .with_children(|parent| {
                        parent.spawn(TextBundle {
                            text : Text {
                                sections : vec![
                                    TextSection::new(
                                        text, 
                                        TextStyle {
                                            font_size: 15.0,
                                            color: Color::srgb(0.9, 0.9, 0.9),
                                            ..default()
                                        }
                                    )
                                ],
                                ..default()
                            },
                            ..default()
                        }	
                    );
                    });
            }).id()
    }

pub fn show_text_button(
        time : Res<Time>,
        mut button_query : Query<(&mut TextButton, &mut Visibility)>
    ) {
    
    	for (mut text, mut visibility) in button_query.iter_mut() {
            text.timer.tick(time.delta());

            if text.timer.just_finished() {
                *visibility = Visibility::Visible;
            }
        }
}

pub fn text_button_interaction(
    asset_server: Res<AssetServer>,
    mut next_main_state: ResMut<NextState<MainState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_sub_state: ResMut<NextState<SubState>>,
    mut interaction_query: Query<
        (&Children, &Interaction, &TextButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands
) {
    for (children, interaction, button) in interaction_query.iter_mut() {
        let text_entity: Option<_> = children.iter().next();
        if let Some(text_entity) = text_entity {
            if let Ok(mut text) = text_query.get_mut(*text_entity) {
                let mut pressed: bool = false;
                match *interaction {
                    Interaction::Pressed => {
                        text.sections[0].style.color = PRESSED_BUTTON;
                        pressed = true;
                    }
                    Interaction::Hovered => {
                        text.sections[0].style.color = HOVERED_BUTTON;
                    }
                    Interaction::None => {
                        text.sections[0].style.color = NORMAL_BUTTON;
                    }
                }

                if pressed {
                    play_sound_once(
                        "sounds/mech_click.ogg", 
                        &mut commands, 
                        &asset_server
                    );

                    StateVector::new(
                        button.next_main_state.clone(),
                        button.next_game_state.clone(),
                        button.next_sub_state.clone()
                    ).set_state(
                        &mut next_main_state, 
                        &mut next_game_state, 
                        &mut next_sub_state
                    );
                }
            }
        }
    }
}

pub fn check_if_enter_pressed(
    mut next_main_state: ResMut<NextState<MainState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_sub_state: ResMut<NextState<SubState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    button_query : Query<&mut TextButton>,
    asset_server : Res<AssetServer>,
    mut commands: Commands
    ) {

    let button = button_query.get_single().unwrap();

    if keyboard_input.just_pressed(KeyCode::Enter) {

        play_sound_once(
            "sounds/mech_click.ogg", 
            &mut commands, 
            &asset_server
        );
        
        StateVector::new(
            button.next_main_state.clone(),
            button.next_game_state.clone(),
            button.next_sub_state.clone()
        ).set_state(
            &mut next_main_state, 
            &mut next_game_state, 
            &mut next_sub_state
        );
    }
}