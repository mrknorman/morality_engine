use std::{
	fs::File,
	io::BufReader, 
	path::PathBuf,
	collections::HashMap
};
use serde::{
	Deserialize, 
	Serialize
};
use rand::Rng;

use bevy::{
	prelude::*, 
	sprite::Anchor, 
	text::LineBreak,
	ecs::{
		component::StorageType,
		system::SystemId
	},
	window::WindowResized
};

use crate::{
    systems::physics::Velocity, 
    entities::text::TextSprite,
    startup::rng::GlobalRng
};

pub mod content;
use content::BackgroundTypes;

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BackgroundSystemsActive {
    #[default]
    False,
    True
}

pub struct BackgroundPlugin;
impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<BackgroundSystemsActive>()
            .add_systems(Update, 
                activate_systems
            ).add_systems(
				Update,
				(
					BackgroundSprite::wrap_sprites,
					Background::on_resize
				)
				.run_if(in_state(BackgroundSystemsActive::True))
			)
			.init_resource::<BackgroundSystems>()
			.register_required_components::<Background, Transform>()
			.register_required_components::<Background, Visibility>();
    }
}

fn activate_systems(
    mut graph_state: ResMut<NextState<BackgroundSystemsActive>>,
    graph_query: Query<&Background>
) {
    graph_state.set(if graph_query.is_empty() {
        BackgroundSystemsActive::False
    } else {
        BackgroundSystemsActive::True
    });
}

#[derive(Resource)]
pub struct BackgroundSystems(pub HashMap<String, SystemId>);

impl FromWorld for BackgroundSystems {
    fn from_world(world: &mut World) -> Self {
        let mut my_item_systems = BackgroundSystems(HashMap::new());

        my_item_systems.0.insert(
            "update_background_speeds".into(),
            world.register_system(Background::update_speeds)
        );

        my_item_systems
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackgroundSpriteType{
	name: String,
	frequency : f32,
	lods : Vec<String>
}

#[derive(Clone)]
pub struct Background{
	sprites : Vec<BackgroundSpriteType>,
	pub density : f32,
	pub speed : f32
}

impl Background {
    pub fn new(
        background_type : BackgroundTypes,
        density: f32,
        speed: f32,
    ) -> Self {
        let sprites: Vec<BackgroundSpriteType> = serde_json::from_str(background_type.content()).unwrap_or_else(|err| {
            panic!("Failed to parse JSON from file {}", err);
        });

        Background {
            sprites,
            density,
            speed,
        }
    }

	fn on_resize(
		mut commands: Commands,
		sprite_query: Query<Entity, With<BackgroundSprite>>,
		background_query: Query<(Entity, &Background)>,
		mut resize_reader: EventReader<WindowResized>,
	) {
		if resize_reader.is_empty() {
			return;
		}
		resize_reader.clear();
	
		sprite_query.iter().for_each(|entity| {
			if let Some(entity_cmds) = commands.get_entity(entity) {
				entity_cmds.despawn_recursive();
			}
		});
	
		background_query.iter().for_each(|(entity, background)| {
			if let Some(mut entity_cmds) = commands.get_entity(entity) {
				entity_cmds.insert(background.clone());
			}
		});
	}

	pub fn update_speeds(
		windows: Query<&Window>,
		background_query: Query<(&Children, &Background), Without<BackgroundSprite>>,
		mut children_query: Query<(&mut Velocity, &Transform), Without<Background>>,
	) {
		let window = windows.get_single().expect("Expected a single window");
		let screen_height = window.height();
	
		background_query.iter().for_each(|(children, background)| {
			let parent_speed = background.speed;
			children.iter().for_each(|child_entity| {
				if let Ok((mut velocity, transform)) = children_query.get_mut(*child_entity) {
					let x_speed = (screen_height - transform.translation.y).max(0.0) * parent_speed;
					velocity.0 = Vec3::new(x_speed, 0.0, 0.0);
				}
			});
		});
	}
	
}

const SPAWN_VARIANCE : f32 = 100.0;

#[derive(Component)]
#[require(TextSprite)]
pub struct BackgroundSprite {
	screen_width : f32,
	random_offset : f32
}

impl BackgroundSprite {
    pub fn wrap_sprites(mut background_query: Query<(&BackgroundSprite, &mut Transform)>) {
        background_query.iter_mut().for_each(|(sprite, mut transform)| {
            if transform.translation.x <= -sprite.screen_width {
                transform.translation.x = sprite.random_offset;
            }
        });
    }
}

impl Component for Background {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_insert(|mut world, entity, _component_id| {
            // Retrieve the Background and TextColor components.
            let entity_ref = world.entity(entity);
            let scene = entity_ref.get::<Background>().cloned();
            let color = entity_ref.get::<TextColor>().cloned();

            // Retrieve the first available Window.
            let window = world
                .iter_entities()
                .filter_map(|entity| entity.get::<Window>().cloned())
                .next();

            if let (Some(scene), Some(window)) = (scene, window) {
                let screen_width = window.width() / 2.0 + 100.0;
                let screen_height = window.height();

                // A helper closure to compute a perspective scale factor.
                // Assumes y = -screen_height/2 is near and y = screen_height/2 is far.
                let perspective_scale = |y: f32| -> f32 {
                    let t = (y + screen_height / 2.0) / screen_height; // normalized [0,1]
                    1.0 - 0.5 * t // near: scale ~1.0, far: scale ~0.5
                };

                // Iterate over each sprite type.
                for sprite_type in scene.sprites {
                    let num_lods = sprite_type.lods.len() as f32;
                    let size_per_range = screen_height / num_lods;

                    for (i, lod) in sprite_type.lods.into_iter().enumerate() {
                        // Define near and far distances for this LOD slice.
                        let d0 = i as f32 * size_per_range;
                        let d1 = (i as f32 + 1.0) * size_per_range;
                        // Use the quadratic area difference to determine density.
                        let raw_density =
                            scene.density * sprite_type.frequency * (d1.powi(2) - d0.powi(2));
                        // Optionally, you could add an exponential falloff here if desired.
                        let density = raw_density.round() as i32;

                        for _ in 0..density {
                            // Borrow the RNG resource briefly to generate spawn positions.
                            let (x_range, y_range, random_offset) = {
                                if let Some(mut rng) = world.get_resource_mut::<GlobalRng>() {
                                    (
                                        rng.0.gen_range(-screen_width..screen_width + SPAWN_VARIANCE),
                                        {
                                            // Sample a y value in the current LOD's range then shift it.
                                            let raw_y = rng.0.gen_range(d0..d1);
                                            raw_y - (screen_height / 2.0)
                                        },
                                        rng.0.gen_range(screen_width..screen_width + SPAWN_VARIANCE),
                                    )
                                } else {
                                    warn!("GlobalRng not found! Cannot spawn background.");
                                    return;
                                }
                            };

                            let translation = Vec3::new(x_range, y_range, 0.0);
                            let scale_factor = perspective_scale(translation.y);
                            let text = lod.to_string();

                            // Spawn the background sprite as a child entity.
                            let mut commands = world.commands();
                            commands.entity(entity).with_children(|parent| {
                                let mut child_entity = parent.spawn((
                                    BackgroundSprite {
                                        screen_width,
                                        random_offset,
                                    },
                                    Velocity(
                                        // Scale the horizontal velocity based on depth.
                                        Vec3::new(
                                            (screen_height / 2.0 - translation.y).max(0.0)
                                                * scene.speed
                                                * scale_factor,
                                            0.0,
                                            0.0,
                                        )
                                    ),
                                    Anchor::BottomCenter,
                                    Text2d::new(text),
                                    // Apply both translation and scale to simulate perspective.
                                    Transform {
                                        translation,
                                        scale: Vec3::splat(scale_factor),
                                        ..Default::default()
                                    },
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextLayout {
                                        justify: JustifyText::Left,
                                        linebreak: LineBreak::WordBoundary,
                                    },
                                ));

                                if let Some(ref color) = color {
                                    child_entity.insert(color.clone());
                                }
                            });
                        }
                    }
                }
            }
        });
    }
}

