mod merlin;

use std::time::Duration;

use bevy::{
    ecs:: component::{
        ComponentHooks, StorageType
    },
    prelude::*
};
use enum_map::{Enum, enum_map};

use crate::{
    audio::{TransientAudio, TransientAudioPallet}, colors::{ColorAnchor, ColorChangeEvent, ColorChangeOn, Flicker, OPTION_1_COLOR, OPTION_2_COLOR}, interaction::{ActionPallet, Clickable, InputAction}, motion::{Bounce, Wobble}, text::{
       get_text_height, get_text_width, TextTitle
    }
};

#[derive(Clone)]
pub struct AsciiString(pub String);

#[derive(Component, Clone)]
#[require(TextTitle)]
pub struct AsciiChar;

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AsciiSystemsActive {
    #[default]
    False,
    True
}

pub struct AsciiPlugin;
impl Plugin for AsciiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<AsciiSystemsActive>()
            .add_systems(Update, 
                activate_systems
            );
        
        app.register_required_components::<AsciiString, Transform>();
        app.register_required_components::<AsciiString, Visibility>();
    }
}

fn activate_systems(
    mut ascii_state: ResMut<NextState<AsciiSystemsActive>>,
    ascii_query: Query<&AsciiString>
) {
    ascii_state.set(if ascii_query.is_empty() {
        AsciiSystemsActive::False
    } else {
        AsciiSystemsActive::True
    });
}

#[derive(Component, Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEmotion {
    Happy,
    Afraid
}


#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsciiActions {
    BounceOnClick
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsciiSounds{
    Bounce
}

impl Component for AsciiString {
    const STORAGE_TYPE: StorageType = StorageType::Table;
   
    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(|mut world, entity, _| {
            // Get the text and components early to avoid borrow conflicts
            let (text, color, beep, emotion) = {
                let entity_ref = world.entity(entity);
                let text = match entity_ref.get::<AsciiString>() {
                    Some(text) => text.0.clone(),
                    None => return,
                };
               
                let color = entity_ref.get::<TextColor>().cloned();
                let emotion = entity_ref.get::<TextEmotion>().cloned();
                
                let asset_server = world.get_resource::<AssetServer>().unwrap();
                let beep: Handle<AudioSource> = asset_server.load("./audio/effects/beep.ogg");
               
                (text, color, beep, emotion)
            };
           
            let mut commands = world.commands();
            let mut translation = Vec3::ZERO;
            
            for line in text.lines() {
                let word_vector = merlin::get_vector(line.to_string());
                if word_vector.is_empty() {
                    continue;
                }
               
                let line_height = get_text_height(&word_vector[0]) + 15.0;
               
                for ascii_char in word_vector {
                    let char_width = get_text_width(&ascii_char);
                   
                    commands.entity(entity).with_children(|parent| {
                        // Spawn entity with base components
                        let mut entity_cmd = parent.spawn((
                            AsciiChar,
                            ColorAnchor::default(),
                            Text2d::new(ascii_char),
                            Transform::from_translation(translation),
                        ));
                        
                        // Add color if available
                        if let Some(color) = &color {
                            entity_cmd.insert(color.clone());
                        }
                        
                        // Add emotion-specific components
                        match &emotion {
                            Some(TextEmotion::Happy) => {
                                entity_cmd
                                    .insert((    
                                        Bounce::default(),
                                        Clickable::new(vec![AsciiActions::BounceOnClick]),
                                        ActionPallet::<AsciiActions, AsciiSounds>(
                                            enum_map! {
                                                AsciiActions::BounceOnClick => vec![
                                                    InputAction::Bounce,
                                                    InputAction::PlaySound(AsciiSounds::Bounce)
                                                ]
                                            }
                                        ),
                                        TransientAudioPallet::new(vec![
                                            (AsciiSounds::Bounce, vec![
                                                TransientAudio::new(beep.clone(), 0.1, true, 1.0, true)
                                            ])
                                        ]),
                                        ColorChangeOn::new(vec![
                                            ColorChangeEvent::Bounce(vec![OPTION_1_COLOR, OPTION_2_COLOR])
                                        ])
                                ));
                            },
                            Some(TextEmotion::Afraid) => {
                                entity_cmd.insert((
                                    Wobble::default(),
                                    ColorChangeOn::new(vec![
                                        ColorChangeEvent::Flicker(
                                            vec![OPTION_1_COLOR, OPTION_2_COLOR],
                                            Flicker::new(
                                                Duration::from_secs_f32(0.1),
                                                Duration::from_secs_f32(1.0),
                                                Duration::from_secs_f32(2.0),
                                                Duration::from_secs_f32(3.0),
                                                Duration::from_secs_f32(0.5)
                                            )
                                        )
                                    ])
                                ));
                            },
                            None => {}
                        }
                    });
                   
                    translation.x += char_width - 15.0;
                }
               
                // Reset for next line
                translation.y -= line_height;
                translation.x = 0.0;
            }
        });
    }
}