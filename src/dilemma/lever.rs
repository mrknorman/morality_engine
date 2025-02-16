use bevy::prelude::*;
use crate::{
	audio::{DilatableAudio, TransientAudio, TransientAudioPallet},
	colors::{
		OPTION_1_COLOR,
		OPTION_2_COLOR
	}, time::Dilation
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
        mut commands: Commands,
        dilation : Res<Dilation>,
        lever: Option<Res<Lever>>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut lever_text_query: Query<(Entity, &mut Text2d, &mut TextColor, &TransientAudioPallet), With<Lever>>,
        mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>
    ) {
        let lever = match lever {
            Some(lever) => lever,
            None => return,
        };
    
        for key in keyboard_input.get_just_pressed() {
            match key {
                KeyCode::Digit1 if lever.0 != LeverState::Left => {
                    commands.insert_resource(Lever(LeverState::Left));
    
                    if let Ok((entity, mut text, mut color, pallet)) = lever_text_query.get_single_mut() {
                        TransientAudioPallet::play_transient_audio(
                            entity,
                            &mut commands,
                            pallet,
                            "lever".to_string(),
                            dilation.0,
                            &mut audio_query
                        );
    
                        text.0 = LEVER_LEFT.to_string();
                        color.0 = OPTION_1_COLOR;
                    } else {
                        warn!("No single lever Text2d entity found to update.");
                    }
                }
    
                KeyCode::Digit2 if lever.0 != LeverState::Right => {
                    commands.insert_resource(Lever(LeverState::Right));
    
                    if let Ok((entity, mut text, mut color, pallet)) = lever_text_query.get_single_mut() {
                        TransientAudioPallet::play_transient_audio(
                            entity,
                            &mut commands,
                            pallet,
                            "lever".to_string(),
                            dilation.0,
                            &mut audio_query
                        );
    
                        text.0 = LEVER_RIGHT.to_string();
                        color.0 = OPTION_2_COLOR;
                    } else {
                        warn!("No single lever Text2d entity found to update.");
                    }
                }
    
                _ => {}
            }
        }
    }
}
