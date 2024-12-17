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
    pub fn to_int(&self) -> Option<usize> {
        match self {
            LeverState::Left => Some(0),
            LeverState::Right => Some(1),
            LeverState::Random => None, // Associating Random with None or -1
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
			Text2d::new(start_text), 
			TextFont{
				font_size : 25.0,
				..default()
			},
			TextColor(color),
			TextLayout{
				justify : JustifyText::Center, 
				..default()
			}, 
			Transform { 
				translation,
				rotation: Quat::from_rotation_z(std::f32::consts::PI / 2.0), 
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
	mut lever_text_query : Query<(&mut Text2d, &mut TextColor), With<LeverText>>,
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
			let (mut text, mut color) = lever_text_query.single_mut();
			text.0 = LEVER_LEFT.to_string();
			color.0 = OPTION_1_COLOR;		
		} else if keyboard_input.just_pressed(KeyCode::Digit2) && (unwrapped_lever.state != LeverState::Right) {
			_ = Some(play_sound_once("sounds/switch.ogg", &mut commands, &asset_server));
			commands.insert_resource(Lever {
				state : LeverState::Right,
				text_entity : unwrapped_lever.text_entity
			});
			let (mut text, mut color) = lever_text_query.single_mut();
			text.0 = LEVER_RIGHT.to_string();
			color.0 = OPTION_2_COLOR;	
		}
	}
}