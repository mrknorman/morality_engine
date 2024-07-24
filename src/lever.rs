use bevy::prelude::*;
use crate::audio::play_sound_once;

pub const OPTION_1_COLOR : Color = Color::WHITE;
pub const OPTION_2_COLOR : Color = Color::WHITE;

const LEVER_LEFT : &str = 
"____(@)
|.-.|/  
|| |/   
|| /|   
||/||   
|| ||   
||_||   
|._.|   
'---'   ";

const LEVER_MIDDLE : &str = 
"  ___    
|.-.|    
|| ||    
|| ||    
||--------(@)
|| ||    
||_||    
|._.|    
'---'    ";

const LEVER_RIGHT : &str = 
"____   
|.-.|   
|| ||   
|| ||   
||\\||   
|| \\|   
||_|\\   
|._.|\\  
'---'(@)";

#[derive(Component)]
pub struct LeverText;

#[derive(PartialEq)]
pub enum LeverState{
	Left,
	Right,
	Random
}

impl LeverState {
    pub fn to_int(&self) -> usize {
        match self {
            LeverState::Left => 1,
            LeverState::Right => 2,
            LeverState::Random => 0, // Associating Random with None or -1
        }
    }
}

#[derive(Resource)]
pub struct Lever{
	pub state : LeverState,
	pub text_entity : Entity
}

impl Lever {
	pub fn spawn(
			translation : Vec3,
			start_state: LeverState,
			commands : &mut Commands
		) -> Entity {
		
		let (start_text, color) = if start_state == LeverState::Left {
			(LEVER_LEFT, OPTION_1_COLOR)
		} else if start_state == LeverState::Right {
			(LEVER_RIGHT, OPTION_2_COLOR)
		} else {
			(LEVER_MIDDLE, Color::WHITE)
		};
		
		let text_entity = commands.spawn((
			LeverText,
			Text2dBundle {
				text : Text {
					sections : vec!(
						TextSection::new(start_text, TextStyle {
							font_size : 25.0,
							color,
							..default()
						})
					),
					justify : JustifyText::Center, 
					..default()
				},
				transform: Transform { 
					translation,
					rotation: Quat::from_rotation_z(std::f32::consts::PI / 2.0), 
					..default()
				},
				..default()
			}
		)).id(); 

		commands.insert_resource(Lever{
			state : start_state,
			text_entity
		});

		text_entity
	}

	pub fn despawn(
		&self,
		commands : &mut Commands
	) {
		commands.entity(self.text_entity).despawn_recursive();
	}
}

pub fn check_level_pull(
    keyboard_input: Res<ButtonInput<KeyCode>>,
	mut lever_text_query : Query<&mut Text, With<LeverText>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    lever: Option<Res<Lever>>,
) {

	if lever.is_some() {
		// Variables to hold new state and audio entity
		let unwrapped_lever: Res<Lever> = lever.unwrap();

		// Determine the new state and despawn the current audio if necessary
		if keyboard_input.just_pressed(KeyCode::Digit1) && (unwrapped_lever.state != LeverState::Left) {
			_ = Some(play_sound_once("sounds/switch.ogg", &mut commands, &asset_server));
			commands.insert_resource(Lever {
				state : LeverState::Left,
				text_entity : unwrapped_lever.text_entity
			});
			lever_text_query.single_mut().sections[0] = TextSection::new(LEVER_LEFT, TextStyle {
				font_size : 25.0,
				color : OPTION_1_COLOR,
				..default()
			});
		
		} else if keyboard_input.just_pressed(KeyCode::Digit2) && (unwrapped_lever.state != LeverState::Right) {
			_ = Some(play_sound_once("sounds/switch.ogg", &mut commands, &asset_server));
			commands.insert_resource(Lever {
				state : LeverState::Right,
				text_entity : unwrapped_lever.text_entity
			});
			lever_text_query.single_mut().sections[0] = TextSection::new(LEVER_RIGHT, TextStyle {
				font_size : 25.0,
				color : OPTION_2_COLOR,
				..default()
			});
		}
	}
}