use bevy::{
    ecs::component::StorageType, prelude::*, render::primitives::Aabb, window::PrimaryWindow
};
use std::time::Duration;
use rand::seq::SliceRandom; // For shuffling
use rand::thread_rng;

use crate::{interaction::{get_cursor_world_position, is_cursor_within_bounds}, motion::{Bounce, Pulse}, time::Dilation};

pub const PRIMARY_COLOR : Color = Color::Srgba(Srgba::new(3.0, 3.0, 3.0, 1.0));
pub const MENU_COLOR : Color = Color::Srgba(Srgba::new(3.0, 3.0, 3.0, 1.0));
pub const _COOL_GREEN : Color = Color::Srgba(Srgba::new(2.0, 5.0, 2.0, 1.0));
pub const HIGHLIGHT_COLOR : Color = Color::Srgba(Srgba::new(3.0, 3.0, 3.0, 1.0)); 
pub const HOVERED_BUTTON: Color = Color::srgb(0.0, 6.0, 6.0);
pub const CLICKED_BUTTON: Color = Color::srgb(8.0, 8.0, 0.0);
pub const DANGER_COLOR: Color = Color::srgb(8.0, 0.0, 0.0);

pub const DIM_BACKGROUND_COLOR: Color = Color::Srgba(Srgba::new(1.0, 1.0, 1.0, 0.2));
pub const BACKGROUND_COLOR: Color = Color::Srgba(Srgba::new(1.0, 1.0, 1.0, 0.8));

pub const OPTION_GLOW : f32 = 6.0;
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
                ColorTranslation::translate,
                Fade::despawn_after_fade,
                ColorChangeOn::color_change_on
            )
            )           
            .init_resource::<ColorPallet>()
            .insert_resource(ColorPallet::default());
    }
}

fn activate_systems(
	mut state: ResMut<NextState<ColorsSystemsActive>>,
	query: Query<&ColorTranslation>
) {
	if !query.is_empty() {
		state.set(ColorsSystemsActive::True)
	} else {
		state.set(ColorsSystemsActive::False)
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
            pressed : CLICKED_BUTTON,
            danger : DANGER_COLOR,
            options : vec![
                OPTION_1_COLOR,
                OPTION_2_COLOR
            ]
        }
    }

}

#[derive(Clone)]
pub struct ColorTranslation {
    pub initial_color: Vec4,
    pub final_color: Vec4,
	pub timer : Timer
}

impl Component for ColorTranslation {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {
                let color: Option<TextColor> = {
                    let entity_mut = world.entity(entity);
                    entity_mut.get::<TextColor>().cloned()
                };

                match color {
                    Some(color) => {
                        if let Some(mut color_anchor) = world.entity_mut(entity).get_mut::<ColorTranslation>() {
                            color_anchor.initial_color = color.0.to_vec4();
                        } else {
                            warn!("Failed to retrieve ColorAnchor component for entity: {:?}", entity);
                        }
                    }
                    None => {
                        warn!("Color anchor should be inserted after text color! Unable to find TextColor.");
                    }
                }
            }
        );
    }
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
    pub fn new(final_color: Color, duration: Duration) -> ColorTranslation {

		let mut translation = ColorTranslation {
			initial_color : Vec4::default(),
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
        dilation : Res<Dilation>,
		mut query: Query<(&mut ColorTranslation, &mut TextColor)>
	) {
		for (mut motion, mut color) in query.iter_mut() {

			motion.timer.tick(time.delta().mul_f32(dilation.0));

			if !motion.timer.paused() && !motion.timer.finished() {

				let fraction_complete = motion.timer.fraction();
				let difference = motion.final_color - motion.initial_color;

                let new_color = motion.initial_color + difference*fraction_complete;
				color.0 = Color::LinearRgba(LinearRgba{
                    red : new_color.x,
                    green : new_color.y,
                    blue : new_color.z,
                    alpha : new_color.w
                });
			} else if motion.timer.just_finished() {
                let new_color = motion.final_color;
                color.0 = Color::LinearRgba(LinearRgba{
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

#[derive(Clone)]
pub struct Fade(pub Duration);

impl Fade {
    fn despawn_after_fade(
        mut commands : Commands,
        mut query : Query<(Entity, &ColorTranslation), With<Fade>>
    ) {
        for (entity, transition) in query.iter_mut() {
            if transition.timer.finished() {
                commands.entity(entity).despawn_recursive();    
            }
        }   
    }
}

impl Component for Fade{
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {
				// Step 1: Extract components from the pallet
				let fade: Option<Fade> = {
                    let entity_mut = world.entity(entity);
                    entity_mut.get::<Fade>()
                        .map(|fade: &Fade| fade.clone())
                };

                let mut commands= world.commands();

                if let Some(fade) = fade {
                    commands.entity(entity).insert(
                        ColorTranslation::new(
                            Color::NONE,
                            fade.0
                        )
                    );
                }

            }
        );
    }
}

#[derive(Clone)]
pub struct ColorAnchor(pub Color);

impl Default for ColorAnchor {
    fn default() -> Self {
        ColorAnchor(Color::WHITE)
    }
}

impl Component for ColorAnchor {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {
                let color: Option<TextColor> = {
                    let entity_mut = world.entity(entity);
                    entity_mut.get::<TextColor>().cloned()
                };

                match color {
                    Some(color) => {
                        if let Some(mut color_anchor) = world.entity_mut(entity).get_mut::<ColorAnchor>() {
                            color_anchor.0 = color.0;
                        } else {
                            warn!("Failed to retrieve ColorAnchor component for entity: {:?}", entity);
                        }
                    }
                    None => {
                        warn!("Color anchor should be inserted after text color! Unable to find TextColor.");
                    }
                }
            }
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorChangeEvent {
    Bounce(Vec<Color>),
    Pulse(Vec<Color>),
    Hover(Vec<Color>),
    Click(Vec<Color>)
}
#[derive(Clone, Component)]
#[require(TextColor)]
pub struct ColorChangeOn{
    pub events : Vec<ColorChangeEvent>,
    event_occurring : bool
}

impl ColorChangeOn {
    pub fn new(events : Vec<ColorChangeEvent>) -> Self {
        Self {
            events,
            event_occurring : true
        }
    }

    fn randomize_color_events(events: &mut [ColorChangeEvent]) {
        for event in events.iter_mut() {
            match event {
                ColorChangeEvent::Bounce(colors) => {
                    colors.shuffle(&mut thread_rng());
                }
                ColorChangeEvent::Pulse(colors) => {
                    colors.shuffle(&mut thread_rng());
                }
                ColorChangeEvent::Hover(colors) => {
                    colors.shuffle(&mut thread_rng());
                }
                ColorChangeEvent::Click(colors) => {
                    colors.shuffle(&mut thread_rng());
                }
            }
        }
    }

    fn color_change_on(
        windows: Query<&Window, With<PrimaryWindow>>,
        mouse_input: Res<ButtonInput<MouseButton>>,
        camera_q: Query<(&Camera, &GlobalTransform)>,
        mut bounce_query: Query<(
            &mut ColorChangeOn,
            &mut TextColor,
            &Aabb,
            &GlobalTransform,
            Option<&ColorAnchor>,
            Option<&Bounce>,
            Option<&Pulse>,
        )>,
    ) {
        for (mut color_change_on, mut text_color, bound, transform, anchor, bounce, pulse) in bounce_query.iter_mut() {
            let mut event_handled = false;

            for event in color_change_on.events.iter() {
                match event {
                    ColorChangeEvent::Bounce(color) => {
                        if let Some(bounce) = bounce {
                            if bounce.enacting {
                                text_color.0 = color[0];
                                event_handled = true;
                                break; 
                            }
                        } else {
                            warn_once!("Entity with color change on bounce has no bounce!");
                        }
                    }
                    ColorChangeEvent::Pulse(color) => {
                        if let Some(pulse) = pulse {
                            if pulse.enacting {
                                text_color.0 = color[0];
                                event_handled = true;
                                break; 
                            }
                        } else {
                            warn_once!("Entity with color change on bounce has no bounce!");
                        }
                    }
                    ColorChangeEvent::Hover(color) => {
                        if let Some(cursor_position) =
                            get_cursor_world_position(&windows, &camera_q)
                        {
                            if is_cursor_within_bounds(cursor_position, transform,&bound) {
                                text_color.0 = color[0];
                                event_handled = true;
                                break;
                            }
                        }
                    } 
                    ColorChangeEvent::Click(color) => {
                        if let Some(cursor_position) =
                            get_cursor_world_position(&windows, &camera_q)
                        {
                            if is_cursor_within_bounds(cursor_position, transform, &bound)
                                && mouse_input.pressed(MouseButton::Left)
                            {
                                text_color.0 = color[0];
                                event_handled = true;
                                break; // Exit the inner loop
                            }
                        }
                    }
                }
            }
            
            // If no event was handled, reset the color to the anchor's value
            if color_change_on.event_occurring && !event_handled {
                color_change_on.event_occurring = false;
                Self::randomize_color_events(&mut color_change_on.events);

                if let Some(anchor) = anchor {
                    text_color.0 = anchor.0;
                }
            }

            color_change_on.event_occurring = event_handled;
        }
    }
}