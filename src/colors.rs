use bevy::prelude::*;
use std::time::Duration;

pub const PRIMARY_COLOR : Color = Color::Srgba(Srgba::new(3.0, 3.0, 3.0, 1.0));
pub const MENU_COLOR : Color = Color::Srgba(Srgba::new(2.0, 5.0, 2.0, 1.0));
pub const HIGHLIGHT_COLOR : Color = Color::Srgba(Srgba::new(3.0, 3.0, 3.0, 1.0)); 
pub const HOVERED_BUTTON: Color = Color::srgb(0.0, 6.0, 6.0);
pub const PRESSED_BUTTON: Color = Color::srgb(8.0, 8.0, 0.0);
pub const DANGER_COLOR: Color = Color::srgb(8.0, 0.0, 0.0);

pub const DIM_BACKGROUND_COLOR: Color = Color::Srgba(Srgba::new(1.0, 1.0, 1.0, 0.2));
pub const BACKGROUND_COLOR: Color = Color::Srgba(Srgba::new(1.0, 1.0, 1.0, 0.8));


pub const OPTION_GLOW : f32 = 2.0;
pub const OPTION_1_COLOR : Color = Color::srgb(0.1015*OPTION_GLOW, 0.5195*OPTION_GLOW, 0.9961*OPTION_GLOW);
pub const OPTION_2_COLOR : Color = Color::srgb(0.8314*OPTION_GLOW, 0.0664*OPTION_GLOW, 0.3477*OPTION_GLOW);


#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ColorsSystemsActive {
    #[default]
    False,
    True
}
pub struct ColorsPlugin;
impl Plugin for ColorsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<ColorsSystemsActive>()
            .add_systems(Update, 
                (activate_systems,
                ColorTranslation::translate)
            )           
            .init_resource::<ColorPallet>()
            .insert_resource(ColorPallet::default());
    }
}

fn activate_systems(
	mut colors_state: ResMut<NextState<ColorsSystemsActive>>,
	colors_query: Query<&ColorTranslation>
) {
	if !colors_query.is_empty() {
		colors_state.set(ColorsSystemsActive::True)
	} else {
		colors_state.set(ColorsSystemsActive::False)
	}
}


#[derive(Resource)]
struct ColorPallet{
    pub primary : Color,
    pub highlight : Color,
    pub hovered : Color,
    pub pressed : Color,
    pub danger : Color,
    pub options : Vec<Color>
}

impl Default for ColorPallet {
    fn default() -> Self {
        Self {
            primary : PRIMARY_COLOR,
            highlight : HIGHLIGHT_COLOR,
            hovered : HOVERED_BUTTON,
            pressed : PRESSED_BUTTON,
            danger : DANGER_COLOR,
            options : vec![
                OPTION_1_COLOR,
                OPTION_2_COLOR
            ]
        }
    }

}

#[derive(Component, Clone)]
#[require(Transform)]
pub struct ColorTranslation {
    pub initial_color: Vec4,
    pub final_color: Vec4,
	pub timer : Timer
}

trait ColorExt {
    fn to_vec4(self) -> Vec4;
}

impl ColorExt for Color {
    fn to_vec4(self) -> Vec4 {
        let color = self.to_linear();
        Vec4::new(color.red, color.green, color.blue, color.alpha)
    }
}

impl ColorTranslation {
    pub fn new(initial_color: Color, final_color: Color, duration: Duration) -> ColorTranslation {

		let mut translation = ColorTranslation {
			initial_color : initial_color.to_vec4(),
			final_color : final_color.to_vec4(),
			timer : Timer::new(
				duration,
				TimerMode::Once
			)
		};

		translation.timer.pause();
		translation
	}

	pub fn translate(
		time: Res<Time>, 
		mut query: Query<(&mut ColorTranslation, &mut TextColor)>
	) {
		for (mut motion, mut color) in query.iter_mut() {

			motion.timer.tick(time.delta());

			if !motion.timer.paused() && !motion.timer.finished() {

				let fraction_complete = motion.timer.fraction();
				let difference = motion.final_color - motion.initial_color;

                let new_color = motion.initial_color + difference*fraction_complete;
				color.0 =  Color::LinearRgba(LinearRgba{
                    red : new_color.x,
                    green : new_color.y,
                    blue : new_color.z,
                    alpha : new_color.w
                });
			}
		}
	}

	pub fn start(&mut self) {
		self.timer.unpause();
	}
}