use std::{
	fs::File,
	io::BufReader, 
	path::PathBuf
};

use bevy::{
	prelude::*, 
	sprite::Anchor, 
	text::BreakLineOn,
	ecs::component::StorageType,
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
					BackgroundSprite::update_positions
				)
				.run_if(in_state(BackgroundSystemsActive::True))
			);
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
}

const BACKGROUND_SIZE : f32 = 2000.0;
const SPAWN_VARIANCE : f32 = 500.0;

#[derive(Component)]
pub struct BackgroundSprite {
	pub speed : f32
}

impl BackgroundSprite {

	pub fn update_positions(
		windows: Query<&Window>,
		time: Res<Time>, // Inject the Time resource to access the game time
		mut background_query : Query<(&mut Transform, &BackgroundSprite)>
	) {

		let window: &Window = windows.get_single().unwrap();
		let screen_height = window.height();

		let time_seconds: f32 = time.delta().as_secs_f32(); // Current time in seconds
		let mut rng: rand::prelude::ThreadRng = rand::thread_rng();

		for (mut transform, sprite) in background_query.iter_mut() {
			let y : f32 = transform.translation.y;

			transform.translation.x -= (screen_height/2.0 - y).max(0.0) * sprite.speed*time_seconds;

			if transform.translation.x <= -2000.0 {
				transform.translation.x = rng.gen_range(BACKGROUND_SIZE..BACKGROUND_SIZE + SPAWN_VARIANCE);
			}
		}
	}

	pub fn update_speed( 
		mut background_query : Query<&mut BackgroundSprite>,
		new_speed : f32
	) {
		for mut sprite in background_query.iter_mut() {
			sprite.speed = new_speed;
		}
	}

	pub fn spawn(
		parent: &mut ChildBuilder<'_>,
		text : &str,
		speed : f32,
		translation: Vec3
	) {
		parent.spawn(
			(Text2dBundle {
				text : Text {
					sections : vec![
						TextSection::new(
							text,
							TextStyle {
								font_size: 12.0,
								..default()
						})
					],
					justify : JustifyText::Left, 
					linebreak_behavior: BreakLineOn::WordBoundary
				},
				transform: Transform::from_translation(
					translation
				),
				text_anchor : Anchor::BottomCenter,
				..default()
			},
			BackgroundSprite{
				speed
			}
			)
		);
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

									commands.entity(entity).with_children(|parent: &mut ChildBuilder<'_>| {
										BackgroundSprite::spawn(
											parent,
											lod.as_str(),
											scene.speed,
											Vec3::new(x_range, y_range, 0.0),
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