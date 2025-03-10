use bevy::{
	prelude::*,
	ecs::component::StorageType
};

use crate::style::ui::{
    BottomAnchor,
    CenterXAnchor
}; 

pub struct CommonUIPlugin;
impl Plugin for CommonUIPlugin {
    fn build(&self, app: &mut App) {
        app
		.insert_resource(NextButtonConfig::default())
        .insert_resource(CenterLeverConfig::default());

        app.register_required_components::<NextButton, BottomAnchor>();
        app.register_required_components::<NextButton, CenterXAnchor>();
    }
}

#[derive(Resource)]
pub struct NextButtonConfig(pub Option<Entity>);
impl Default for NextButtonConfig {
    fn default() -> Self {
        Self(None)
    }
}

pub struct NextButton;
impl NextButton {
	pub fn transform(window: &Window) -> Transform {
		let button_distance = 100.0;
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


                let mut transform = None;
                if let Some(window) = world
                .iter_entities()
                .filter_map(|entity| entity.get::<Window>().cloned())
                .next() {
                    transform = Some(NextButton::transform(&window));
                };

                // Get an immutable reference to the resource
                let mut previous_entity : Option<Entity> = None;
                if let Some(config) = world.get_resource::<NextButtonConfig>() {
                    if let Some(entity) = config.0 {
                        if world.get_entity(entity).is_ok() {
                            previous_entity = Some(entity);
                        }
                    }
                }

                let mut commands = world.commands();
                if let Some(transform) = transform {
                    commands.entity(entity).insert(transform);
                }

                if let Some(entity) = previous_entity {
                    commands.entity(entity).despawn_recursive();
                }
                
                if let Some(mut config) = world.get_resource_mut::<NextButtonConfig>() {
                    config.0 = Some(entity);
                }
            }
        );
    }
}


#[derive(Resource)]
pub struct CenterLeverConfig(pub Option<Entity>);
impl Default for CenterLeverConfig {
    fn default() -> Self {
        Self(None)
    }
}

pub struct CenterLever;
impl CenterLever {
	pub fn transform(window: &Window) -> Transform {
		let button_distance = 150.0;
		let screen_height = window.height();
		let button_y = -screen_height / 2.0 + button_distance; 
        Transform { 
            translation : Vec3::new(0.0, button_y, 1.0),
            rotation: Quat::from_rotation_z(std::f32::consts::PI / 2.0), 
            ..default()
        }
	}
}

impl Component for CenterLever {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {       

                let mut transform = None;
                if let Some(window) = world
                .iter_entities()
                .filter_map(|entity| entity.get::<Window>().cloned())
                .next() {
                    transform = Some(CenterLever::transform(&window));
                };     

                // Get an immutable reference to the resource
                let mut previous_entity : Option<Entity> = None;
                if let Some(config) = world.get_resource::<CenterLeverConfig>() {
                    if let Some(entity) = config.0 {
                        if world.get_entity(entity).is_ok() {
                            previous_entity = Some(entity);
                        }
                    }
                }

                let mut commands = world.commands();
                if let Some(transform) = transform {
                    commands.entity(entity).insert(transform);
                }
                if let Some(entity) = previous_entity {
                    commands.entity(entity).despawn_recursive();
                }
                if let Some(mut config) = world.get_resource_mut::<CenterLeverConfig>() {
                    config.0 = Some(entity);
                }
            }
        );
    }
}



#[derive(Resource)]
pub struct DilemmaTimerConfig(pub Option<Entity>);
impl Default for DilemmaTimerConfig {
    fn default() -> Self {
        Self(None)
    }
}

pub struct DilemmaTimerPosition;
impl DilemmaTimerPosition {
	pub fn transform(window: &Window) -> Transform {
		let button_distance = 250.0;
		let screen_height = window.height();
		let button_y = -screen_height / 2.0 + button_distance; 
        Transform { 
            translation : Vec3::new(0.0, button_y, 1.0),
            ..default()
        }
	}
}

impl Component for DilemmaTimerPosition {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {       

                let mut transform = None;
                if let Some(window) = world
                .iter_entities()
                .filter_map(|entity| entity.get::<Window>().cloned())
                .next() {
                    transform = Some(DilemmaTimerPosition::transform(&window));
                };     

                // Get an immutable reference to the resource
                let mut previous_entity : Option<Entity> = None;
                if let Some(config) = world.get_resource::<DilemmaTimerConfig>() {
                    if let Some(entity) = config.0 {
                        if world.get_entity(entity).is_ok() {
                            previous_entity = Some(entity);
                        }
                    }
                }

                let mut commands = world.commands();
                if let Some(transform) = transform {
                    commands.entity(entity).insert(transform);
                }
                if let Some(entity) = previous_entity {
                    commands.entity(entity).despawn_recursive();
                }
                
                if let Some(mut config) = world.get_resource_mut::<DilemmaTimerConfig>() {
                    config.0 = Some(entity);
                }
            }
        );
    }
}