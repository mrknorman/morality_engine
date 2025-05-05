use bevy::{
	ecs::{component::{HookContext, Mutable, StorageType}, world::DeferredWorld}, prelude::*
};

pub struct CommonUIPlugin;
impl Plugin for CommonUIPlugin {
    fn build(&self, app: &mut App) {
        app
		.insert_resource(NextButtonConfig::default())
        .insert_resource(CenterLeverConfig::default());
    }
}

#[derive(Resource)]
pub struct NextButtonConfig(pub Option<Entity>);
impl Default for NextButtonConfig {
    fn default() -> Self {
        Self(None)
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
#[component(on_insert = NextButton::on_insert)]
pub struct NextButton;

impl NextButton {
	pub fn transform(window: &Window) -> Transform {
		let button_distance = 100.0;
		let screen_height = window.height();
		let button_y = -screen_height / 2.0 + button_distance; 
        Transform::from_xyz(0.0, button_y, 1.0)
	}

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let transform = world
            .iter_entities()
            .find_map(|e| e.get::<Window>().cloned())
            .map(|window| NextButton::transform(&window));
    
        let previous = world
            .get_resource::<NextButtonConfig>()
            .and_then(|cfg| cfg.0)
            .filter(|&e| world.get_entity(e).is_ok());
    
        let mut cmd = world.commands();
    
        if let Some(t) = transform {
            cmd.entity(entity).insert(t);
        }
        if let Some(old) = previous {
            cmd.entity(old).despawn();
        }
    
        if let Some(mut cfg) = world.get_resource_mut::<NextButtonConfig>() {
            cfg.0 = Some(entity);
        }
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
    type Mutability = Mutable;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, context| {       

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
                    commands.entity(context.entity).insert(transform);
                }
                if let Some(entity) = previous_entity {
                    commands.entity(entity).despawn();
                }
                if let Some(mut config) = world.get_resource_mut::<CenterLeverConfig>() {
                    config.0 = Some(context.entity);
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
    type Mutability = Mutable;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, context| {       
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
                    commands.entity(context.entity).insert(transform);
                }
                if let Some(entity) = previous_entity {
                    commands.entity(entity).despawn();
                }
                
                if let Some(mut config) = world.get_resource_mut::<DilemmaTimerConfig>() {
                    config.0 = Some(context.entity);
                }
            }
        );
    }
}