use std::{
	fs::File,
	io::BufReader, 
	path::PathBuf,
	collections::HashMap
};
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
use rand::Rng;
use serde::{
	Deserialize, 
	Serialize
};
use crate::text::TextSprite;
use crate::GlobalRng;

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
					BackgroundSprite::update_positions,
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
    pub fn load_from_json<P: Into<PathBuf>>(
        file_path: P,
        density: f32,
        speed: f32,
    ) -> Self {
        // Convert the `Into<PathBuf>` parameter to a `PathBuf`
        let file_path: PathBuf = file_path.into();

        let file = File::open(&file_path).unwrap_or_else(|err| {
            panic!("Failed to open file {}: {}", file_path.display(), err);
        });

        let reader = BufReader::new(file);
        let sprites: Vec<BackgroundSpriteType> = serde_json::from_reader(reader).unwrap_or_else(|err| {
            panic!("Failed to parse JSON from file {}: {}", file_path.display(), err);
        });

        Background {
            sprites,
            density,
            speed,
        }
    }

	fn on_resize(
		mut commands : Commands,
		background_query : Query<(Entity, Option<&Parent>, &Background, &TextColor)>,
		parent_query : Query<Entity>,
		mut resize_reader: EventReader<WindowResized>,
	) {

		let mut removed_backgrounds = vec![];
		for _ in resize_reader.read() {	
			for (entity, parent, background, color) in background_query.iter() {
				if let Some(entity) = commands.get_entity(entity) {
					entity.despawn_recursive();
					removed_backgrounds.push((parent, background.clone(), color.clone()));
				}
			}
		}

		for (parent, background, color) in removed_backgrounds {
			if let Some(parent) = parent {
				if let Ok(parent_entity) = parent_query.get(parent.get()) {
					commands.entity(parent_entity).with_children(|parent| {
						parent.spawn(
					(
								color,
								background
							)
						);
					});
				}
			} else {
				commands.spawn(
					background
				);
			}
		} 
	}

	pub fn update_speeds(
		windows: Query<&Window>,
		background_query: Query<(&Children, &Background), Without<BackgroundSprite>>,
		mut children_query: Query<(&mut BackgroundSprite, &Transform), Without<Background>>,
	) {

		let window: &Window = windows.get_single().unwrap();
		let screen_height = window.height();

		// Loop over all parents
		for (children, background) in background_query.iter() {
			// Extract the parent's speed
			let parent_speed = background.speed;
	
			// Loop over each child entity
			for &child_entity in children.iter() {
				// If we can get a mutable reference to the child's BackgroundSprite
				if let Ok((mut child_sprite, transform)) = children_query.get_mut(child_entity) {
					// Update the child's speed
					child_sprite.speed = (screen_height - transform.translation.y).max(0.0)*parent_speed;
				}
			}
		}
	}
}

const SPAWN_VARIANCE : f32 = 100.0;

#[derive(Component)]
#[require(TextSprite)]
pub struct BackgroundSprite {
	screen_width : f32,
	screen_width_max : f32,
	speed : f32
}

impl BackgroundSprite {

	pub fn update_positions(
		time: Res<Time>,
		mut rng : ResMut<GlobalRng>,
		mut background_query : Query<(&BackgroundSprite, &mut Transform)>
	) {

		let time_seconds: f32 = time.delta().as_secs_f32();

		for (sprite, mut transform) in background_query.iter_mut() {
			transform.translation.x -= sprite.speed*time_seconds;

			if transform.translation.x <= -sprite.screen_width {
				let random_offset: f32 = rng.0.gen_range(sprite.screen_width..sprite.screen_width_max);
				transform.translation.x = random_offset;
			}
		}
	}
}

impl Component for Background {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {
                let scene: Option<Background> = {
                    let entity_ref: EntityRef<'_> = world.entity(entity);
                    entity_ref.get::<Background>().cloned()
                };

				let color: Option<TextColor> = {
                    let entity_ref: EntityRef<'_> = world.entity(entity);
                    entity_ref.get::<TextColor>().cloned()
                };

				let window: Option<Window> = world
                    .iter_entities()
                    .filter_map(|entity| entity.get::<Window>().cloned())
                    .next();

                if let Some(scene) = scene {

					if let Some(window) = window {

						let screen_width = window.width()/2.0 + 100.0;
						let screen_height = window.height();
						let mut rng = rand::thread_rng();

						for sprite_type in scene.sprites {
							let density: i32 = (scene.density*screen_width*screen_height*sprite_type.frequency) as i32;
							let size_per_range = screen_height / (sprite_type.lods.len() as f32);

							for (i, lod) in sprite_type.lods.into_iter().enumerate() {

								let density = density*(((i as i32 + 1)).pow(2));
								for _ in 0..density {
									let x_range: f32 =  rng.gen_range(-screen_width..screen_width + SPAWN_VARIANCE);
									let y_range: f32 = rng.gen_range(
										size_per_range*(i as f32)..size_per_range*(i as f32 + 1.0)
									) - (screen_height/2.0);
									
									let mut commands = world.commands();

									let translation = Vec3::new(x_range, y_range, 0.0);
									let text = lod.to_string();

									commands.entity(entity).with_children(|parent: &mut ChildBuilder<'_>| {
										let mut entity = parent.spawn(
											(
												BackgroundSprite{
													screen_width,
													screen_width_max : screen_width + SPAWN_VARIANCE,
													speed : (screen_height - translation.y).max(0.0)*scene.speed
												},
												Anchor::BottomCenter,
												Text2d::new(text),
												Transform::from_translation(
													translation
												),
												TextFont{
													font_size: 12.0,
													..default()
												},
												TextLayout{
													justify : JustifyText::Left,
													linebreak : LineBreak::WordBoundary
												},
											)
										);

										if let Some(color) = color {
											entity.insert(color);
										}
									});
								}
							}
						}
					}

                }
            },
        );
    }
}    