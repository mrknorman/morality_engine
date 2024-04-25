use bevy::prelude::*;
use std::{path::PathBuf, time::Duration};
use rand::Rng;

use crate::game_states::MainState;
use crate::audio::play_sound_once;

#[derive(Resource)]
pub struct MenuData {
	title_entity : Entity,
	train_entity : Entity,
	train_audio : Entity,
	train_button : Entity,
	signature_entity: Entity,
    button_entity: Entity,
}

const NORMAL_BUTTON: Color = Color::rgb(1.0, 1.0, 1.0);
const HOVERED_BUTTON: Color = Color::rgb(0.0, 1.0, 1.0);
const PRESSED_BUTTON: Color = Color::rgb(1.0, 1.0, 0.0);

#[derive(Component)]
pub struct TrainWhistle {
	playing : bool
}

#[derive(Component)]
pub struct TrainTrack;

#[derive(Component)]
pub struct TrainSmoke {
	index : usize, 
	smoke : Vec<String>
}


#[derive(Component)]
pub struct TrainPart{
	index : usize,
	initial_position : Vec3
}

#[derive(Component)]
pub struct TrainEngine{
	index : usize,
	initial_position : Vec3,
	timer: Timer
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

	let train = "\n
	 _____                 . . . . . o o o o o
	__|[_]|__ ___________ _______    ____      o
   |[] [] []| [] [] [] [] [_____(__  ][]]_n_n__][.
  _|________|_[_________]_[_________]_|__|________)<
	oo    oo 'oo      oo ' oo    oo 'oo 0000---oo\\_
   ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n";

	let smoke_1 = String::from(". . . . . o o o o o
                    o");

	let smoke_2 = String::from(". . . . o o o o o o
                    .");
	
	let smoke_3 = String::from(". . . o o o o o o .
                    .");

	let smoke_4 = String::from(". . . o o o o o o .
                    .");

	let smoke_5 = String::from(". . o o o o o o . .
                    .");
	
	let smoke_6 = String::from(". o o o o o o . . .
                    .");

	let smoke_7 = String::from("o o o o o o . . . .
                    .");

	let smoke_8 = String::from("o o o o o . . . . .
                    o");
		
	let smoke_9 = String::from("o o o o . . . . . o
                    o");

	let smoke_10 = String::from("o o o . . . . . o o
                    o");

	let smoke_11 = String::from("o o . . . . . o o o
                    o");

	let smoke_12 = String::from("o . . . . . o o o o
                    o");
	
	let smoke_13 = String::from(". . . . . o o o o o
                    o");


	let smoke_parts = vec![smoke_1, smoke_2, smoke_3, smoke_4, smoke_5, smoke_6, smoke_7, smoke_8, smoke_9, smoke_10, smoke_11, smoke_12, smoke_13];

	let back_carridge = "\n
      _____    
  __|[_]|__
 |[] [] []|
_|________|
  oo    oo \n";

  let middle_carridge = "\n
             
 ___________ 
 [] [] [] [] 
_[_________]
'oo      oo '\n";

  let coal_truck = "\n
                                                     
 _______  
 [_____(__  
_[_________]_
  oo    oo ' \n";

  let engine_car = "____      
 ][]]_n_n__][.
_|__|________)<
oo 0000---oo\\_\n";


let track = "\n
                                                         




~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n";


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

	//let button_2

	let train_audio = commands.spawn(AudioBundle {
		source: asset_server.load(PathBuf::from("./sounds/train_loop.ogg")),
		settings : PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(0.1),
			mode:  bevy::audio::PlaybackMode::Loop,
			..default()
	}}).id();

	let train_button = commands
        .spawn((NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
				margin: UiRect {
					top : Val::Px(-53.0),
					left : Val::Px(72.0),
					..default()
				},
                ..default()
            },
            ..default()
		},
		TrainEngine {
			index : 0,
			initial_position : Transform::from_xyz(-0.0, 0.0, 1.0).translation,
			timer :  Timer::new(Duration::from_millis(100), TimerMode::Repeating)
		}
		))
        .with_children(|parent| {
            parent
                .spawn((ButtonBundle {
                    style: Style {
                        width: Val::Px(100.),
                        height: Val::Px(45.),
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
                }, TrainWhistle{playing : false}
				
				))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
						text : Text {
							sections : vec![
								TextSection::new(engine_car, TextStyle {
									font_size: 12.0,
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

	let train_entity = commands.spawn((TrainTrack,
		Text2dBundle {
			text : Text {
				sections : vec!(
					TextSection::new(track, TextStyle {
						font_size : 12.0,
						..default()
					})
				),
				justify : JustifyText::Center, 
				..default()
			},
			transform: Transform::from_xyz(-0.0, 70.0, 1.0),
			..default()
		},)).with_children(|parent| {
			
			parent.spawn((
				Text2dBundle {
					text : Text {
						sections : vec!(
							TextSection::new(coal_truck, TextStyle {
								font_size : 12.0,
								..default()
							})
						),
						justify : JustifyText::Center, 
						..default()
					},
					transform: Transform::from_xyz(30.0, 0.0, 1.0),
					..default()
				},
				TrainPart {
					index : 1,
					initial_position :  Transform::from_xyz(30.0, 0.0, 1.0).translation
				})
			); 

            parent.spawn((
				Text2dBundle {
					text : Text {
						sections : vec!(
							TextSection::new(middle_carridge, TextStyle {
								font_size : 12.0,
								..default()
							})
						),
						justify : JustifyText::Center, 
						..default()
					},
					transform: Transform::from_xyz(-40.0, 0.0, 1.0),
					..default()
				},
				TrainPart {
					index : 2,
					initial_position :  Transform::from_xyz(-40.0, 0.0, 1.0).translation
				})
			); 
			
            parent.spawn((
				Text2dBundle {
					text : Text {
						sections : vec!(
							TextSection::new(back_carridge, TextStyle {
								font_size : 12.0,
								..default()
							})
						),
						justify : JustifyText::Center, 
						..default()
					},
					transform: Transform::from_xyz(-110.0, 0.0, 1.0),
					..default()
				},
				TrainPart {
					index : 2,
					initial_position :  Transform::from_xyz(-110.0, 0.0, 1.0).translation
				})
			); 

			parent.spawn((
				Text2dBundle {
					text : Text {
						sections : vec!(
							TextSection::new(smoke_parts[0].clone(), TextStyle {
								font_size : 12.0,
								..default()
							})
						),
						justify : JustifyText::Center, 
						..default()
					},
					transform: Transform::from_xyz(70.0, 10.0, 1.0),
					..default()
				}, TrainSmoke{index:0, smoke : smoke_parts})
			); 
		
		}
		).id();

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
    commands.insert_resource(MenuData {
		title_entity, 
		train_entity,
		train_audio,
		train_button,
		signature_entity,
		button_entity 
	});
}

pub fn menu(
    mut next_state: ResMut<NextState<MainState>>,
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
	commands.entity(menu_data.signature_entity).despawn_recursive();
	commands.entity(menu_data.train_entity).despawn_recursive();
	commands.entity(menu_data.train_audio).despawn_recursive();
	commands.entity(menu_data.train_button).despawn_recursive();
}

pub fn train_whistle(
    mut interaction_query: Query<
        (&Children, &Interaction, &TrainWhistle),
        (Changed<Interaction>, With<Button>, With<TrainWhistle>),
    >,
	mut text_query: Query<&mut Text>,
	asset_server : Res<AssetServer>,
	mut commands: Commands
	) {
		
	for (children, interaction, _) in &mut interaction_query {

		let text_entity = children.iter().next();

		if let Some(text_entity) = text_entity {
			if let Ok(mut text) = text_query.get_mut(*text_entity) {
				match *interaction {
					Interaction::Pressed => {
						text.sections[0].style.color = PRESSED_BUTTON;

						play_sound_once(
							"sounds/horn.ogg", 
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
}

pub fn wobble_train(
		time: Res<Time>, // Inject the Time resource to access the game time
		mut transform_query: Query<(&mut Transform, &mut TrainPart) >,
		mut button_query: Query<(&mut Style, &mut TrainEngine)>,
		mut smoke_query: Query<(&mut Text, &mut TrainSmoke)>
	) {

    let mut rng = rand::thread_rng(); // Random number generator

	let (mut style, mut train_part) = button_query.single_mut();

	train_part.timer.tick(time.delta());

	if train_part.timer.finished() {

		// Calculate offset using sine and cosine functions for smooth oscillation
		let dx = rng.gen_range(-1.0..=1.0);
		let dy = rng.gen_range(-1.0..=1.0);  

		// Apply the calculated offsets to the child's position
		style.top = Val::Px(train_part.initial_position.x + dx as f32);
		style.left = Val::Px(train_part.initial_position.y + dy as f32);

		let time_seconds = time.elapsed_seconds_f64() as f32; // Current time in seconds

		for (mut transform, train_part) in transform_query.iter_mut() {
				// Calculate offset using sine and cosine functions for smooth oscillation
				let dx = rng.gen_range(-1.0..=1.0);
				let dy = rng.gen_range(-1.0..=1.0);  

				// Apply the calculated offsets to the child's position
				transform.translation.x = train_part.initial_position.x + dx as f32;
				transform.translation.y = train_part.initial_position.y + dy as f32;
		}

		for (mut text, mut smoke_part) in smoke_query.iter_mut() {

			smoke_part.index += 1;

			text.sections[0].value = smoke_part.smoke[smoke_part.index % smoke_part.smoke.len() ].clone();
		}
	}
}