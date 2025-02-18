use bevy::prelude::*;
use crate::colors::{
    OPTION_1_COLOR,
    OPTION_2_COLOR
};
#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LeverSystemsActive {
    #[default]
	False,
    True
}

pub struct LeverPlugin;

impl Plugin for LeverPlugin {
    fn build(&self, app: &mut App) {	
		app
		.init_state::<LeverSystemsActive>()
		.add_systems(
			Update,
			activate_systems
		).add_systems(
            Update,
            Lever::update
            .run_if(in_state(LeverSystemsActive::True))
            .run_if(resource_changed::<Lever>)
        );
    }
}

fn activate_systems(
        mut state: ResMut<NextState<LeverSystemsActive>>,
        query: Query<&Lever>
    ) {
        
	if !query.is_empty() {
		state.set(LeverSystemsActive::True)
	} else {
		state.set(LeverSystemsActive::False)
	}
}


pub const LEVER_LEFT : &str = 
"____(@)
|.-.|/  
|| |/   
|| /|   
||/||   
|| ||   
||_||   
|._.|   
'---'   ";

pub const LEVER_MIDDLE : &str = 
"  ___    
|.-.|    
|| ||    
|| ||    
||--------(@)
|| ||    
||_||    
|._.|    
'---'    ";

pub const LEVER_RIGHT : &str = 
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

impl Default for LeverText {
	fn default() -> Self {
		Self
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Component, Resource)]
#[require(LeverText)]
pub struct Lever(pub LeverState);


impl Lever {
    pub fn update(
        lever: Option<Res<Lever>>,
        mut lever_text_query: Query<(&mut Text2d, &mut TextColor), With<Lever>>,
    ) {
        let lever = match lever {
            Some(lever) => lever,
            None => return,
        };

        match lever.0 {
            LeverState::Left => {
                if let Ok(( mut text, mut color)) = lever_text_query.get_single_mut() {
                    text.0 = LEVER_LEFT.to_string();
                    color.0 = OPTION_1_COLOR;
                } else {
                    warn!("No single lever Text2d entity found to update.");
                }
            }

            LeverState::Right => {
                if let Ok((mut text, mut color)) = lever_text_query.get_single_mut() {
                    text.0 = LEVER_RIGHT.to_string();
                    color.0 = OPTION_2_COLOR;
                } else {
                    warn!("No single lever Text2d entity found to update.");
                }
            }

            LeverState::Random => {
                if let Ok(( mut text, mut color)) = lever_text_query.get_single_mut() {
                    text.0 = LEVER_MIDDLE.to_string();
                    color.0 = Color::WHITE;
                } else {
                    warn!("No single lever Text2d entity found to update.");
                }
            }
        }
    }
}
