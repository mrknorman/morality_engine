use bevy::prelude::*;
use std::path::PathBuf;

use crate::train::{
	TrainEntities, 
	Train, 
	TrainWhistle,
	TrainText

};
use crate::io_elements::{spawn_text_button, NORMAL_BUTTON, HOVERED_BUTTON, PRESSED_BUTTON};

use crate::game_states::{MainState, GameState};
use crate::audio::play_sound_once;

#[derive(Resource)]
pub struct MenuData {
	title_entity : Entity,
	train_entity : TrainEntities,
	train_audio : Entity,
	signature_entity: Entity,
    button_entity: Entity,
}

pub fn setup_menu(
		mut commands: Commands,
		asset_server : Res<AssetServer>
	) {

	let ascii_art = r#"
	___      ___     ______     _______        __      ___        __  ___________  ___  ___       _______  _____  ___    _______   __    _____  ___    _______  
	|"  \    /"  |   /    " \   /"      \      /""\    |"  |      |" \("     _   ")|"  \/"  |     /"     "|(\"   \|"  \  /" _   "| |" \  (\"   \|"  \  /"     "| 
	 \   \  //   |  // ____  \ |:        |    /    \   ||  |      ||  |)__/  \\__/  \   \  /     (: ______)|.\\   \    |(: ( \___) ||  | |.\\   \    |(: ______) 
	 /\\  \/.    | /  /    ) :)|_____/   )   /' /\  \  |:  |      |:  |   \\_ /      \\  \/       \/    |  |: \.   \\  | \/ \      |:  | |: \.   \\  | \/    |   
	|: \.        |(: (____/ //  //      /   //  __'  \  \  |___   |.  |   |.  |      /   /        // ___)_ |.  \    \. | //  \ ___ |.  | |.  \    \. | // ___)_  
	|.  \    /:  | \        /  |:  __   \  /   /  \\  \( \_|:  \  /\  |\  \:  |     /   /        (:      "||    \    \ |(:   _(  _|/\  |\|    \    \ |(:      "| 
	|___|\__/|___|  \"_____/   |__|  \___)(___/    \___)\_______)(__\_|_)  \__|    |___/          \_______) \___|\____\) \_______)(__\_|_)\___|\____\) \_______)"#;

	let title_entity = commands.spawn(
        (Text2dBundle {
            text : Text {
                sections : vec!(
					TextSection::new(ascii_art, TextStyle {
						font_size : 12.0,
						..default()
					})
				),
                justify : JustifyText::Center, 
				..default()
            },
			transform: Transform::from_xyz(0.0, 150.0, 1.0),
            ..default()
        },
		AudioBundle {
			source: asset_server.load(PathBuf::from("./sounds/static.ogg")),
			settings : PlaybackSettings {
				paused : false,
				volume : bevy::audio::Volume::new(0.06),
				mode:  bevy::audio::PlaybackMode::Loop,
				..default()
			}
		})
    ).id();

	let train_audio = commands.spawn(AudioBundle {
		source: asset_server.load(PathBuf::from("./sounds/train_loop.ogg")),
		settings : PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(0.1),
			mode:  bevy::audio::PlaybackMode::Loop,
			..default()
	}}).id();

	let train_1 = TrainText::new(true, 50);

	let train = Train::new(
		train_1.train_track_text,
		train_1.train_engine_text,
		train_1.carridge_text_vector,
		train_1.smoke_text_frames,
		Vec3::new(100.0, 0.0, 1.0)
	);

	let train_entity = train.spawn(&mut commands);

	let signature_entity = commands.spawn(
        (Text2dBundle {
            text : Text {
                sections : vec!(
					TextSection::new("A game by Michael Norman", TextStyle {
						font_size : 12.0,
						..default()
					})		
				),				
                justify : JustifyText::Center, 

				..default()
            },
			transform: Transform::from_xyz(0.0, 10.0, 1.0),
            ..default()
        },
		AudioBundle {
			source: asset_server.load(PathBuf::from("./sounds/office.ogg")),
			settings : PlaybackSettings {
				paused : false,
				volume : bevy::audio::Volume::new(1.0),
				mode:  bevy::audio::PlaybackMode::Loop,
				..default()
			}
		})
    ).id();

    let button_entity = spawn_text_button(
		"[Click here or Press Enter to Begin]",
		Some(MainState::InGame),
		Some(GameState::Dialogue),
		0.0,
		&mut commands
	);

    commands.insert_resource(MenuData {
		title_entity, 
		train_entity,
		train_audio,
		signature_entity,
		button_entity 
	});
}

pub fn startup_effects(
		next_main_state: &mut ResMut<NextState<MainState>>,
		next_game_state: &mut ResMut<NextState<GameState>>,
		commands: &mut Commands, 
		asset_server : &Res<AssetServer>
	) {

	play_sound_once(
		"sounds/mech_click.ogg", 
		commands, 
		asset_server
	);

	next_main_state.set(MainState::InGame);
	next_game_state.set(GameState::Loading);	
}

pub fn menu(
    mut next_main_state: ResMut<NextState<MainState>>,
	mut next_game_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&Children, &Interaction),
        (Changed<Interaction>, With<Button>, Without<TrainWhistle>),
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

						startup_effects(
							&mut next_main_state,
							&mut next_game_state,
							&mut commands, 
							&asset_server
						);
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
		startup_effects(
			&mut next_main_state,
			&mut next_game_state,
			&mut commands, 
			&asset_server
		);
	}
}

pub fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn_recursive();
	commands.entity(menu_data.title_entity).despawn_recursive();
	commands.entity(menu_data.signature_entity).despawn_recursive();
	commands.entity(menu_data.train_entity.train).despawn_recursive();
	commands.entity(menu_data.train_audio).despawn_recursive();

	if menu_data.train_entity.engine.is_some() {
		commands.entity(menu_data.train_entity.engine.unwrap()).despawn_recursive();
	}
}