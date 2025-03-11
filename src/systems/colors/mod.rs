use bevy::{
    ecs::component::StorageType,
    prelude::*,
    render::primitives::Aabb,
    window::PrimaryWindow,
    time::TimerMode,
};
use rand_pcg::Pcg64Mcg;
use std::time::Duration;
use rand::{Rng, seq::SliceRandom};

use crate::{
    systems::{
        interaction::{
            get_cursor_world_position,
            is_cursor_within_bounds, 
            is_cursor_within_region
        },
        motion::{Bounce, Pulse},
        time::Dilation
    },
    entities::sprites::compound::{
        AssembleShape,
        Plus
    },
    data::rng::GlobalRng,
    startup::render::MainCamera
};

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
pub const OPTION_1_COLOR : Color = Color::srgb(0.1015 * OPTION_GLOW, 0.5195 * OPTION_GLOW, 0.9961 * OPTION_GLOW);
pub const OPTION_2_COLOR : Color = Color::srgb(0.8314 * OPTION_GLOW, 0.0664 * OPTION_GLOW, 0.3477 * OPTION_GLOW);

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ColorsSystemsActive {
    #[default]
    False,
    True
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum ColorSystemOrder {
    ResetFlag,
    CheckColorChangeEvents,
}

pub struct ColorsPlugin;
impl Plugin for ColorsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<ColorsSystemsActive>()
            .add_systems(Update,
                ColorChangeOn::reset_event_flag.in_set(ColorSystemOrder::ResetFlag))
            .add_systems(Update,
                (
                    ColorChangeOn::handle_physics_events,
                    ColorChangeOn::handle_cursor_events,
                    ColorChangeOn::handle_cursor_events_shapes::<Plus>,
                    ColorChangeOn::handle_flicker_events
                ).in_set(ColorSystemOrder::CheckColorChangeEvents).after(ColorSystemOrder::ResetFlag))
            .add_systems(Update, 
                (
                    ColorChangeOn::finalize_color_events, 
                    ColorChangeOn::finalize_color_events_shapes::<Plus>
                ).after(ColorSystemOrder::CheckColorChangeEvents)
            )
            .add_systems(Update, 
                (activate_systems,
                ColorTranslation::translate,
                Fade::despawn_after_fade,
                // Add the flicker event system here if needed:
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
struct ColorPallet {
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
                OPTION_2_COLOR,
            ]
        }
    }
}

#[derive(Clone)]
pub struct ColorTranslation {
    pub initial_color: Vec4,
    pub final_color: Vec4,
    pub timer : Timer,
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
    pub fn new(final_color: Color, duration: Duration, paused: bool) -> ColorTranslation {
        let mut translation = ColorTranslation {
            initial_color: Vec4::default(),
            final_color: final_color.to_vec4(),
            timer: Timer::new(duration, TimerMode::Once),
        };

        if paused {
            translation.timer.pause();
        }
        translation
    }

    pub fn translate(
        time: Res<Time>, 
        dilation: Res<Dilation>,
        mut query: Query<(&mut ColorTranslation, &mut TextColor)>
    ) {
        for (mut motion, mut color) in query.iter_mut() {
            motion.timer.tick(time.delta().mul_f32(dilation.0));

            if !motion.timer.paused() && !motion.timer.finished() {
                let fraction_complete = motion.timer.fraction();
                let difference = motion.final_color - motion.initial_color;
                let new_color = motion.initial_color + difference * fraction_complete;
                color.0 = Color::LinearRgba(LinearRgba {
                    red: new_color.x,
                    green: new_color.y,
                    blue: new_color.z,
                    alpha: new_color.w,
                });
            } else if motion.timer.just_finished() {
                let new_color = motion.final_color;
                color.0 = Color::LinearRgba(LinearRgba {
                    red: new_color.x,
                    green: new_color.y,
                    blue: new_color.z,
                    alpha: new_color.w,
                });
            }
        }
    }

    pub fn start(&mut self) {
        self.timer.unpause();
    }
}

#[derive(Clone)]
pub struct Fade {
    pub duration: Duration,
    pub paused: bool,
}

impl Fade {
    fn despawn_after_fade(
        mut commands: Commands,
        mut query: Query<(Entity, &ColorTranslation), With<Fade>>
    ) {
        for (entity, transition) in query.iter_mut() {
            if transition.timer.finished() {
                commands.entity(entity).despawn_recursive();    
            }
        }   
    }
}

impl Component for Fade {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {
                let fade: Option<Fade> = {
                    let entity_mut = world.entity(entity);
                    entity_mut.get::<Fade>()
                        .map(|fade: &Fade| fade.clone())
                };
                
                if let Some(fade) = fade {
                    world.commands().entity(entity).insert(
                        ColorTranslation::new(
                            Color::NONE,
                            fade.duration,
                            fade.paused
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


/// The Flicker struct controls a flickering effect that periodically
/// switches the rendered color to one selected randomly from a given vector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flicker {
    enacting: bool,
    inter_flicker_timer: Timer,
    flicker_timer: Timer,
    min_flicker_duration: Duration,
    max_flicker_duration: Duration,
    min_interval: Duration,
    max_interval: Duration,
    /// How fast the color toggles on/off during a burst.
    toggle_period: Duration,
}

impl Flicker {
    /// Create a new Flicker with randomized durations.
    /// Note: The RNG is no longer required at creation.
    pub fn new(
        min_flicker_duration: Duration,
        max_flicker_duration: Duration,
        min_interval: Duration,
        max_interval: Duration,
        toggle_period: Duration,
    ) -> Self {
        // Initialize with the minimum interval as the initial downtime.
        Self {
            enacting: false,
            inter_flicker_timer: Timer::new(min_interval, TimerMode::Once),
            // The flicker timer will be set when a burst starts.
            flicker_timer: Timer::new(Duration::ZERO, TimerMode::Once),
            min_flicker_duration,
            max_flicker_duration,
            min_interval,
            max_interval,
            toggle_period,
        }
    }

    /// Update the flicker state.
    /// Returns true only when a burst is active and the sub-cycle (toggle) is "on".
    /// Otherwise it returns false.
    pub fn update(&mut self, dt: Duration, rng: &mut Pcg64Mcg) -> bool {
        if !self.enacting {
            self.inter_flicker_timer.tick(dt);
            if self.inter_flicker_timer.finished() {
                // Begin a new flicker burst with a random duration.
                let flicker_duration = rng.gen_range(self.min_flicker_duration..self.max_flicker_duration);
                self.flicker_timer.set_duration(flicker_duration);
                self.flicker_timer.reset();
                self.flicker_timer.unpause();
                self.enacting = true;
            }
            return false;
        } else {
            self.flicker_timer.tick(dt);
            if self.flicker_timer.finished() {
                // End burst and randomize the next interval.
                let interval = rng.gen_range(self.min_interval..self.max_interval);
                self.inter_flicker_timer.set_duration(interval);
                self.inter_flicker_timer.reset();
                self.inter_flicker_timer.unpause();
                self.enacting = false;
                return false;
            }
            // Compute sub-cycle toggle: alternate on/off every half toggle_period.
            let elapsed = self.flicker_timer.elapsed();
            let cycle = elapsed.as_secs_f32() % self.toggle_period.as_secs_f32();
            return cycle < (self.toggle_period.as_secs_f32() / 2.0);
        }
    }
}

/// The ColorChangeEvent enum now includes a Flicker variant which
/// holds a vector of colors and a Flicker instance.
#[derive(Debug, Clone, PartialEq)]
pub enum ColorChangeEvent {
    Bounce(Vec<Color>),
    Pulse(Vec<Color>),
    Flicker(Vec<Color>, Flicker),
    Hover(Vec<Color>, Option<Vec2>),
    Click(Vec<Color>, Option<Vec2>),
}

impl ColorChangeEvent {
    fn get_color_mut(&mut self) -> &mut Vec<Color> {
        match self {
            ColorChangeEvent::Bounce(colors)
            | ColorChangeEvent::Pulse(colors)
            | ColorChangeEvent::Flicker(colors, _)
            | ColorChangeEvent::Hover(colors, _)
            | ColorChangeEvent::Click(colors, _) => colors,
        }
    }
}

#[derive(Clone, Component)]
#[require(TextColor)]
pub struct ColorChangeOn {
    pub events: Vec<ColorChangeEvent>,
    event_occurring: bool,
}

impl ColorChangeOn {
    pub fn new(events: Vec<ColorChangeEvent>) -> Self {
        Self {
            events,
            event_occurring: true,
        }
    }

    fn randomize_color_events(events: &mut [ColorChangeEvent], rng: &mut Pcg64Mcg) {
        events.iter_mut().for_each(|event| event.get_color_mut().shuffle(rng));
    }

    fn reset_event_flag(mut query: Query<&mut ColorChangeOn>) {
        for mut color_change in query.iter_mut() {
            color_change.event_occurring = false;
        }
    }

    fn handle_physics_events(
        mut query: Query<(&mut ColorChangeOn, &mut TextColor, Option<&Bounce>, Option<&Pulse>)>,
    ) {
        for (mut color_change, mut text_color, bounce_opt, pulse_opt) in query.iter_mut() {
            if color_change.event_occurring {
                continue;
            }
            for event in color_change.events.iter() {
                match event {
                    ColorChangeEvent::Bounce(colors) => {
                        if let Some(bounce) = bounce_opt {
                            if bounce.enacting {
                                text_color.0 = colors[0];
                                color_change.event_occurring = true;
                                break;
                            }
                        } else {
                            warn_once!("Entity with ColorChangeOn Bounce event has no Bounce component!");
                        }
                    }
                    ColorChangeEvent::Pulse(colors) => {
                        if let Some(pulse) = pulse_opt {
                            if pulse.enacting {
                                text_color.0 = colors[0];
                                color_change.event_occurring = true;
                                break;
                            }
                        } else {
                            warn_once!("Entity with ColorChangeOn Pulse event has no Pulse component!");
                        }
                    }
                    _ => {}
                }
                if color_change.event_occurring {
                    break;
                }
            }
        }
    }

    fn handle_cursor_events_shapes<T: AssembleShape + Component>(
        window_query: Single<&Window, With<PrimaryWindow>>,
        mouse_input: Res<ButtonInput<MouseButton>>,
        camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
        mut query: Query<(&mut ColorChangeOn, &mut T, &Transform, &GlobalTransform)>,
    ) {
        for (mut color_change, mut shape, transform, global_transform) in query.iter_mut() {
            if color_change.event_occurring {
                continue;
            }
            for event in color_change.events.iter() {
                match event {
                    ColorChangeEvent::Hover(colors, region) => {
                        if let Some(cursor_position) =
                            get_cursor_world_position(&window_query, &camera_query)
                        {
                            if is_cursor_within_region(
                                cursor_position,
                                transform,
                                global_transform,
                                region.unwrap_or(shape.dimensions()),
                                Vec2::ZERO,
                            ) {
                                shape.set_color(colors[0]);
                                color_change.event_occurring = true;
                                break;
                            }
                        }
                    }
                    ColorChangeEvent::Click(colors, region) => {
                        if let Some(cursor_position) =
                            get_cursor_world_position(&window_query, &camera_query)
                        {
                            if is_cursor_within_region(
                                cursor_position,
                                transform,
                                global_transform,
                                region.unwrap_or(shape.dimensions()),
                                Vec2::ZERO,
                            ) && mouse_input.pressed(MouseButton::Left)
                            {
                                shape.set_color(colors[0]);
                                color_change.event_occurring = true;
                                break;
                            }
                        }
                    }
                    _ => {}
                }
                if color_change.event_occurring {
                    break;
                }
            }
        }
    }

    fn handle_cursor_events(
        window_query: Single<&Window, With<PrimaryWindow>>,
        mouse_input: Res<ButtonInput<MouseButton>>,
        camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
        mut query: Query<(&mut ColorChangeOn, &mut TextColor, &Aabb, &GlobalTransform)>,
    ) {
        for (mut color_change, mut text_color, aabb, transform) in query.iter_mut() {
            if color_change.event_occurring {
                continue;
            }
            for event in color_change.events.iter() {
                match event {
                    ColorChangeEvent::Hover(colors, _) => {
                        if let Some(cursor_position) =
                            get_cursor_world_position(&window_query, &camera_query)
                        {
                            if is_cursor_within_bounds(cursor_position, transform, aabb) {
                                text_color.0 = colors[0];
                                color_change.event_occurring = true;
                                break;
                            }
                        }
                    }
                    ColorChangeEvent::Click(colors, _) => {
                        if let Some(cursor_position) =
                            get_cursor_world_position(&window_query, &camera_query)
                        {
                            if is_cursor_within_bounds(cursor_position, transform, aabb)
                                && mouse_input.pressed(MouseButton::Left)
                            {
                                text_color.0 = colors[0];
                                color_change.event_occurring = true;
                                break;
                            }
                        }
                    }
                    _ => {}
                }
                if color_change.event_occurring {
                    break;
                }
            }
        }
    }

    fn handle_flicker_events(
        time: Res<Time>,
        mut rng: ResMut<GlobalRng>,
        mut query: Query<(&mut ColorChangeOn, &mut TextColor)>,
    ) {
        for (mut color_change, mut text_color) in query.iter_mut() {
            if color_change.event_occurring {
                continue;
            }
            for event in color_change.events.iter_mut() {
                if let ColorChangeEvent::Flicker(colors, ref mut flicker) = event {
                    // Pass the RNG so durations are randomized.
                    if flicker.update(time.delta(), &mut rng.0) {
                        // During "on" phase, choose a random color.
                        if let Some(chosen) = colors.choose(&mut rng.0) {
                            text_color.0 = *chosen;
                            color_change.event_occurring = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    fn finalize_color_events(
        mut rng: ResMut<GlobalRng>,
        mut query: Query<(&mut ColorChangeOn, &mut TextColor, Option<&ColorAnchor>)>,
    ) {
        for (mut color_change, mut text_color, color_anchor_opt) in query.iter_mut() {
            if !color_change.event_occurring {
                ColorChangeOn::randomize_color_events(&mut color_change.events, &mut rng.0);
                if let Some(anchor) = color_anchor_opt {
                    text_color.0 = anchor.0;
                }
            }
        }
    }

    fn finalize_color_events_shapes<T: AssembleShape + Component>(
        mut rng: ResMut<GlobalRng>,
        mut query: Query<(&mut ColorChangeOn, &mut T, Option<&ColorAnchor>)>,
    ) {
        for (mut color_change, mut shape_color, color_anchor_opt) in query.iter_mut() {
            if !color_change.event_occurring {
                ColorChangeOn::randomize_color_events(&mut color_change.events, &mut rng.0);
                if let Some(anchor) = color_anchor_opt {
                    shape_color.set_color(anchor.0);
                }
            }
        }
    }
}