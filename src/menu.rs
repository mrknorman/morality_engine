use bevy::prelude::*;

use crate::game_states::MainState;
use crate::audio::play_sound_once;

#[derive(Resource)]
pub struct MenuData {
	title_entity : Entity,
    button_entity: Entity,
}

const NORMAL_BUTTON: Color = Color::rgb(1.0, 1.0, 1.0);
const HOVERED_BUTTON: Color = Color::rgb(0.0, 1.0, 1.0);
const PRESSED_BUTTON: Color = Color::rgb(1.0, 1.0, 0.0);

pub fn setup_menu(mut commands: Commands) {

	let ascii_art = r#"
	___      ___     ______     _______        __      ___        __  ___________  ___  ___       _______  _____  ___    _______   __    _____  ___    _______  
	|"  \    /"  |   /    " \   /"      \      /""\    |"  |      |" \("     _   ")|"  \/"  |     /"     "|(\"   \|"  \  /" _   "| |" \  (\"   \|"  \  /"     "| 
	 \   \  //   |  // ____  \ |:        |    /    \   ||  |      ||  |)__/  \\__/  \   \  /     (: ______)|.\\   \    |(: ( \___) ||  | |.\\   \    |(: ______) 
	 /\\  \/.    | /  /    ) :)|_____/   )   /' /\  \  |:  |      |:  |   \\_ /      \\  \/       \/    |  |: \.   \\  | \/ \      |:  | |: \.   \\  | \/    |   
	|: \.        |(: (____/ //  //      /   //  __'  \  \  |___   |.  |   |.  |      /   /        // ___)_ |.  \    \. | //  \ ___ |.  | |.  \    \. | // ___)_  
	|.  \    /:  | \        /  |:  __   \  /   /  \\  \( \_|:  \  /\  |\  \:  |     /   /        (:      "||    \    \ |(:   _(  _|/\  |\|    \    \ |(:      "| 
	|___|\__/|___|  \"_____/   |__|  \___)(___/    \___)\_______)(__\_|_)  \__|    |___/          \_______) \___|\____\) \_______)(__\_|_)\___|\____\) \_______)"#;

	let train = "\n\n    _____                 . . . . . o o o o o
	__|[_]|__ ___________ _______    ____      o
   |[] [] []| [] [] [] [] [_____(__  ][]]_n_n__][.
  _|________|_[_________]_[_________]_|__|________)<
	oo    oo 'oo      oo ' oo    oo 'oo 0000---oo\\_
   ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n";

	let title_entity = commands.spawn(
        (Text2dBundle {
            text : Text {
                sections : vec!(
					TextSection::new(ascii_art, TextStyle {
						font_size : 12.0,
						..default()
					}),
					TextSection::new(train, TextStyle {
						font_size : 12.0,
						..default()
					}),
					TextSection::new("A game by Michael Norman", TextStyle {
						font_size : 12.0,
						..default()
					})		
				),
                justify : JustifyText::Center, 
				..default()
            },
			transform: Transform::from_xyz(0.0, 50.0, 1.0),
            ..default()
        },
    )).id();

    let button_entity = commands
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
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(278.),
                        height: Val::Px(30.),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color : bevy::prelude::BackgroundColor(
						Color::Rgba{red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0}
					),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
						text : Text {
							sections : vec![
								TextSection::new("[Click here or Press Enter to Begin]", TextStyle {
									font_size: 15.0,
									color: Color::rgb(0.9, 0.9, 0.9),
									..default()
								})
							],
							..default()
						},
						..default()
					}	
				);
                });
        })
        .id();
    commands.insert_resource(MenuData {title_entity, button_entity });
}

pub fn menu(
    mut next_state: ResMut<NextState<MainState>>,
    mut interaction_query: Query<
        (&Children, &Interaction),
        (Changed<Interaction>, With<Button>),
    >,
	mut text_query: Query<&mut Text>,
	keyboard_input: Res<ButtonInput<KeyCode>>,
	mut commands: Commands, 
	asset_server : Res<AssetServer>
	) {
		
	for (children, interaction) in &mut interaction_query {

		let text_entity = children.iter().next();
		if let Some(text_entity) = text_entity {
			if let Ok(mut text) = text_query.get_mut(*text_entity) {
				match *interaction {
					Interaction::Pressed => {
						text.sections[0].style.color = PRESSED_BUTTON;

						play_sound_once(
							"sounds/mech_click.ogg", 
							&mut commands, 
							&asset_server
						);

						next_state.set(MainState::InGame);
					}
					Interaction::Hovered => {
						text.sections[0].style.color = HOVERED_BUTTON;
					}
					Interaction::None => {
						text.sections[0].style.color = NORMAL_BUTTON;
					}
				}
			}
		}
	}

	if keyboard_input.just_pressed(KeyCode::Enter) {
		play_sound_once(
			"sounds/mech_click.ogg", 
			&mut commands, 
			&asset_server
		);

		next_state.set(MainState::InGame);
	}


}

pub fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn_recursive();
	commands.entity(menu_data.title_entity).despawn_recursive();
}