use bevy::{
	prelude::*,
	ecs::component::StorageType
};

use crate::io::{BottomAnchor, CenterXAnchor}; 

#[derive(Resource)]
pub struct NextButtonConfig{
    entity : Option<Entity>
}

impl NextButtonConfig {
    pub fn empty() -> Self {
        Self {
            entity : None
        }
    }

	pub fn entity(&self) -> Option<Entity> {
		self.entity
	}
}

pub struct NextButton;

#[derive(Bundle)]
pub struct NextButtonBundle{
	marker : NextButton,
	anchor : BottomAnchor,
    center_anchor : CenterXAnchor
}

impl NextButton {
	pub fn transform(windows: &Query<&Window>) -> Transform {
		let button_distance = 100.0;
		let window: &Window = windows.get_single().unwrap();
		let screen_height = window.height();
		let button_y = -screen_height / 2.0 + button_distance; 
        Transform::from_xyz(0.0, button_y, 1.0)
	}
}

impl Component for NextButton {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {            

                // Get an immutable reference to the resource
                let mut previous_entity : Option<Entity> = None;
                if let Some(audio_config) = world.get_resource::<NextButtonConfig>() {
                    if let Some(entity) = audio_config.entity {
                        if world.get_entity(entity).is_ok() {
                            previous_entity = Some(entity);
                        }
                    }
                }

                let mut commands = world.commands();
                if let Some(entity) = previous_entity {
                    commands.entity(entity).despawn_recursive();
                }
                
                if let Some(mut audio_config) = world.get_resource_mut::<NextButtonConfig>() {
                    audio_config.entity = Some(entity);
                }
            }
        );
    }
}

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommonUISystemsActive {
    #[default]
    False,
    True
}

pub struct CommonUIPlugin;
impl Plugin for CommonUIPlugin {
    fn build(&self, app: &mut App) {
        app
		.insert_resource(NextButtonConfig::empty());

        app.register_required_components::<NextButton, BottomAnchor>();
        app.register_required_components::<NextButton, CenterXAnchor>();
    }
}
