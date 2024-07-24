use rand::Rng;
use bevy::{prelude::*, sprite::Anchor, text::{BreakLineOn, Text2dBounds}};
use crate::game_states::{MainState, GameState, SubState};

pub const SMALL_CACTUS : &str = "
  |
(_|_)
  |";

pub const LARGE_CACTUS: &str = "
    _  _
   | || | _
  -| || || |
   | || || |-
    \\_  || |
      |  _/
     -| |
      |_|-    
";

#[derive(Component)]
pub struct BackgroundSprite {
	pub speed : f32
}

impl BackgroundSprite {

	pub fn move_background_spites(
		state : Res<State<SubState>>,
		time: Res<Time>, // Inject the Time resource to access the game time
		mut background_query : Query<(&mut Transform, &BackgroundSprite)>
	) {

		let time_seconds: f32 = time.delta().as_millis() as f32 / 1000.0; // Current time in seconds
		let mut rng = rand::thread_rng();

		for (mut transform, sprite) in background_query.iter_mut() {
			let y : f32 = transform.translation.y;

			transform.translation.x -= (500.0 - y ) * sprite.speed*time_seconds;

			if transform.translation.x < -1000.0 && (*state.get() == SubState::Intro || (rng.gen::<f64>() > 0.8)) {
				transform.translation.x = 1000.0;
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
		commands : &mut Commands,
		text : &str,
		speed : f32,
		translation: Vec3
	) {
		commands.spawn(
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

	pub fn spawn_multi(
		commands: &mut Commands, 
		small_text : &str, 
		large_text : &str,
		ground_cover : &str,
		speed : f32,
		n : i32
	) {
		let mut rng = rand::thread_rng();
	
		for _ in 0..n*2 {
			let x_small = rng.gen_range(-1000.0..1000.0);
			let y_small = rng.gen_range(-100.0..350.0);
	
			BackgroundSprite::spawn(
			    commands,
				small_text,
				speed,
				Vec3::new(x_small, y_small, 0.0),
			);
		}
	
		for _ in 0..n {
			let x_large = rng.gen_range(-1000.0..1000.0);
			let y_large = rng.gen_range(-500.0..-100.0);
	
			BackgroundSprite::spawn(
				commands,
				large_text,
				speed,
				Vec3::new(x_large, y_large, 0.0),
			);
		}

		for _ in 0..n*70 {
			let x_large: f32 = rng.gen_range(-1000.0..1000.0);
			let y_large: f32 = rng.gen_range(-500.0..400.0);
	
			BackgroundSprite::spawn(
				commands,
				ground_cover,
				speed,
				Vec3::new(x_large, y_large, 0.0),
			);
		}

		for _ in 0..n*10 {
			let x_large = rng.gen_range(-1000.0..1000.0);
			let y_large = rng.gen_range(-500.0..0.0);
	
			BackgroundSprite::spawn(
				commands,
				"\\|/",
				speed,
				Vec3::new(x_large, y_large, 0.0),
			);
		}
	}
}