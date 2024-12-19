use std::{
	fs::File,
	io::BufReader, 
	path::PathBuf
};

use bevy::{
	prelude::*, 
	sprite::Anchor, 
	text::LineBreak,
	ecs::component::StorageType,
	window::WindowResized
};
use rand::Rng;
use serde::{
	Deserialize, 
	Serialize
};

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
			);

		app.register_required_components::<Background, Transform>();
		app.register_required_components::<Background, Visibility>();
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackgroundSpriteType{
	name: String,
	frequency : f32,
	lods : Vec<String>
}

#[derive(Clone)]
pub struct Background{
	sprites : Vec<BackgroundSpriteType>,
	brightness : f32,
	pub density : f32,
	pub speed : f32
}

impl Background {

    pub fn load_from_json<P: Into<PathBuf>>(
        file_path: P,
		brightness : f32,
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
			brightness,
            density,
            speed,
        }
    }

	pub fn update_speed( 
		mut background_query : Query<&mut Background>,
		new_speed : f32
	) {
		for mut background in background_query.iter_mut() {
			background.speed = new_speed;
		}
	}

	fn on_resize(
		mut commands : Commands,
		background_query : Query<(Entity,  Option<&Parent>, &Background)>,
		parent_query : Query<Entity>,
		mut resize_reader: EventReader<WindowResized>,
	) {

		let mut removed_backgrounds = vec![];
		for _ in resize_reader.read() {	
			for (entity, parent, background) in background_query.iter() {
				if let Some(entity) = commands.get_entity(entity) {
					entity.despawn_recursive();
					removed_backgrounds.push((parent, background.clone()));
				}
			}
		}

		for (parent, background) in removed_backgrounds {
			if let Some(parent) = parent {
				if let Ok(parent_entity) = parent_query.get(parent.get()) {
					commands.entity(parent_entity).with_children(|parent| {
						parent.spawn(
							background
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
	
}

const BACKGROUND_SIZE : f32 = 2000.0;
const SPAWN_VARIANCE : f32 = 500.0;

#[derive(Component)]
pub struct BackgroundSprite;

impl BackgroundSprite {

	pub fn update_positions(
		windows: Query<&Window>,
		time: Res<Time>, // Inject the Time resource to access the game time
		mut background_query : Query<(&Parent, &mut Transform), With<BackgroundSprite>>,
		parent_query : Query<&Background>
	) {

		let window: &Window = windows.get_single().unwrap();
		let screen_height = window.height()/2.0;

		let time_seconds: f32 = time.delta().as_secs_f32(); // Current time in seconds
		let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
		let random_offset = rng.gen_range(BACKGROUND_SIZE..BACKGROUND_SIZE + SPAWN_VARIANCE);

		for (parent, mut transform) in background_query.iter_mut() {
			if let Ok(master) = parent_query.get(parent.get()) {

				let y : f32 = transform.translation.y;

				transform.translation.x -= (screen_height - y).max(0.0) * master.speed*time_seconds;

				if transform.translation.x <= -2000.0 {
					transform.translation.x = random_offset;
				}
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

				let window: Option<Window> = world
                    .iter_entities()
                    .filter_map(|entity| entity.get::<Window>().cloned())
                    .next();

                if let Some(scene) = scene {

					if let Some(window) = window {

						let screen_height = window.height();
						let mut rng = rand::thread_rng();

						for sprite_type in scene.sprites {
							let density: i32 = (scene.density*sprite_type.frequency) as i32;

							let size_per_range = screen_height / (sprite_type.lods.len() as f32);

							for (i, lod) in sprite_type.lods.into_iter().enumerate() {

								let density = density*(((i as i32 + 1)).pow(2));
								for _ in 0..density {
									let x_range: f32 =  rng.gen_range(-BACKGROUND_SIZE..BACKGROUND_SIZE + SPAWN_VARIANCE);
									let y_range: f32 = rng.gen_range(size_per_range*(i as f32)..size_per_range*(i as f32 + 1.0)) - (screen_height/2.0);
									
									let mut commands = world.commands();

									let translation = Vec3::new(x_range, y_range, 0.0);
									let text = lod.to_string();
									let brightness = scene.brightness;

									commands.entity(entity).with_children(|parent: &mut ChildBuilder<'_>| {
										parent.spawn(
											(
												BackgroundSprite,
												Anchor::BottomCenter,
												Text2d::new(text),
												Transform::from_translation(
													translation
												),
												TextColor(Color::Srgba(Srgba::new(1.0, 1.0, 1.0, brightness))),
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