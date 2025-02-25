mod merlin;

use bevy::{
    ecs:: component::{
        ComponentHooks, StorageType
    },
    prelude::*
};
use enum_map::{Enum, enum_map};

use crate::{
    audio::{TransientAudio, TransientAudioPallet}, colors::{ColorAnchor, ColorChangeEvent, ColorChangeOn, OPTION_1_COLOR, OPTION_2_COLOR}, interaction::{ActionPallet, Clickable, InputAction}, motion::Bounce, text::{
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

        hooks.on_insert(
            |mut world, entity, _component_id| {
				// Step 1: Extract components from the pallet
				let text: Option<AsciiString> = {
                    let entity_mut = world.entity(entity);
                    entity_mut.get::<AsciiString>()
                        .map(|text: &AsciiString| text.clone())
                };

                let color: Option<TextColor> = {
                    let entity_mut = world.entity(entity);
                    entity_mut.get::<TextColor>()
                        .map(|text: &TextColor| text.clone())
                };

                let asset_server = world.get_resource::<AssetServer>().unwrap();

                let beep = asset_server.load("sounds/beep.ogg");

                let mut commands = world.commands();

                if let Some(text) = text {
                    let mut translation = Vec3::new(0.0, 0.0, 1.0); 

                    for line in text.0.lines() {
                        let word_vector: Vec<String> = merlin::get_vector(line.to_string());
                        let line_height = get_text_height(&word_vector[0]) + 15.0;

                        for ascii_char in word_vector {
                            let char_width = get_text_width(&ascii_char);


                            commands.entity(entity).with_children(|parent| {
                                let mut entity = parent.spawn((
                                    AsciiChar,
                                    Bounce::default(),
                                    Clickable::new(vec!(AsciiActions::BounceOnClick)),
                                    ActionPallet::<AsciiActions, AsciiSounds>(
                                        enum_map!(
                                            AsciiActions::BounceOnClick => vec![
                                                InputAction::Bounce,
                                                InputAction::PlaySound(AsciiSounds::Bounce)
                                            ]
                                        )
                                    ),
                                    ColorChangeOn::new(vec![ColorChangeEvent::Bounce(vec![OPTION_1_COLOR, OPTION_2_COLOR])]),
                                    ColorAnchor::default(),
                                    Text2d::new(ascii_char),
                                    Transform::from_translation(translation),
                                    TransientAudioPallet::new(
                                        vec![(
                                            AsciiSounds::Bounce,
                                            vec![
                                                TransientAudio::new(
                                                    beep.clone(), 
                                                    0.1, 
                                                    true,
                                                    1.0,
                                                    true
                                                )
                                            ]
                                        )]
                                    )
                                ));
                                
                                if let Some(color) = color {
                                    entity.insert(color);
                                }
                            });
                            translation += Vec3::new(char_width - 15.0, 0.0, 0.0);
                        }

                        translation += Vec3::new(0.0, -line_height, 0.0);
                        translation.x = 0.0;
                    }
                }
            }
        );
    }
}