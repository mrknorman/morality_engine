mod merlin;

use bevy::{
    ecs:: component::{
        ComponentHooks, StorageType
    },
    prelude::*
};

use crate::{
    motion::Bouncy, text::{
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

                let mut commands = world.commands();

                if let Some(text) = text {

                    let mut translation = Vec3::new(0.0, 0.0, 1.0); 

                    for line in text.0.lines() {
                        
                        let word_vector: Vec<String> = merlin::get_vector(line.to_string());
                        let line_height = get_text_height(&word_vector[0]) + 15.0;

                        for ascii_char in word_vector {
                            let char_width = get_text_width(&ascii_char);

                            commands.entity(entity).with_children(|parent| {
                                parent.spawn(
                                    (
                                        AsciiChar,
                                        Bouncy::default(),
                                        Text2d::new(ascii_char),
                                        Transform::from_translation(translation)
                                    )
                                );
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