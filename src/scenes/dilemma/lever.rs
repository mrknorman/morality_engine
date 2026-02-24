use crate::{
    entities::text::scaled_font_size,
    startup::cursor::CursorMode,
    systems::{
        colors::option_color,
        interaction::ClickableCursorIcons,
    },
};
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
    sprite::Anchor,
};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LeverSystemsActive {
    #[default]
    False,
    True,
}

pub struct LeverPlugin;
impl Plugin for LeverPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LeverSystemsActive>()
            .add_systems(Update, activate_systems)
            .add_systems(
                Update,
                Lever::update
                    .run_if(in_state(LeverSystemsActive::True).and(resource_changed::<Lever>)),
            );
    }
}

fn activate_systems(mut state: ResMut<NextState<LeverSystemsActive>>, query: Query<&LeverRoot>) {
    if !query.is_empty() {
        state.set(LeverSystemsActive::True)
    } else {
        state.set(LeverSystemsActive::False)
    }
}

pub const LEVER_SHAFT: &str = "  .----.  \n /@@@@@\\ \n |@@()@@| \n \\@@@@@/ \n  '----'  \n    ||    \n    ||    \n    ||    \n    ||    \n    ||    \n    ||    \n    ||    \n    ||    ";
pub const LEVER_BASE: &str = "      _____________________      \n      / /_______@@________\\ \\      \n     / /___________________\\ \\    \n    / /_____________________\\ \\   \n  [__________________________] \n  |__________________________| ";

const LEVER_MIN_ANGLE_DEG: f32 = -45.0;
const LEVER_MAX_ANGLE_DEG: f32 = 45.0;
const LEVER_DEFAULT_OPTIONS: usize = 1;
const LEVER_FONT_SIZE: f32 = 12.0;
const LEVER_SHAFT_PIVOT_Y: f32 = 16.0;
const LEVER_SHAFT_PIVOT_Z: f32 = 0.2;

#[derive(Component, Default)]
pub struct LeverBase;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct LeverShaftPivot;

#[derive(Component)]
pub struct LeverShaft;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeverState {
    Selected(usize),
    Random,
}

impl LeverState {
    pub fn to_int(&self) -> Option<usize> {
        match self {
            LeverState::Selected(index) => Some(*index),
            LeverState::Random => None,
        }
    }

    pub fn from_option_index(option_index: Option<usize>) -> Self {
        option_index.map_or(Self::Random, Self::Selected)
    }
}

#[derive(Component, Clone, Copy)]
#[require(LeverBase, ClickableCursorIcons = LeverRoot::default_cursor_icons())]
#[component(on_insert = LeverRoot::on_insert)]
pub struct LeverRoot;

#[derive(Resource, Clone, Copy)]
pub struct Lever(pub LeverState, pub usize);

impl LeverRoot {
    fn default_cursor_icons() -> ClickableCursorIcons {
        ClickableCursorIcons {
            on_hover: CursorMode::PullLeft,
        }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let mut has_shaft = false;
        if let Some(children) = world.entity(entity).get::<Children>() {
            for child in children.iter() {
                if let Ok(child_ref) = world.get_entity(child) {
                    if child_ref.contains::<LeverShaftPivot>() {
                        has_shaft = true;
                        break;
                    }
                }
            }
        }
        if has_shaft {
            return;
        }

        world.commands().entity(entity).with_children(|parent| {
            parent
                .spawn((
                    Name::new("lever_shaft_pivot"),
                    LeverShaftPivot,
                    Transform::from_xyz(0.0, LEVER_SHAFT_PIVOT_Y, LEVER_SHAFT_PIVOT_Z),
                ))
                .with_children(|shaft_parent| {
                    shaft_parent.spawn((
                        Name::new("lever_shaft"),
                        LeverShaft,
                        Text2d::new(LEVER_SHAFT),
                        TextFont {
                            font_size: scaled_font_size(LEVER_FONT_SIZE),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TextLayout {
                            justify: Justify::Center,
                            ..default()
                        },
                        Anchor::BOTTOM_CENTER,
                    ));
                });
        });
    }
}

impl Lever {
    pub fn with_option_count(state: LeverState, option_count: usize) -> Self {
        let normalized_count = option_count.max(LEVER_DEFAULT_OPTIONS);
        let normalized_state = match state {
            LeverState::Selected(index) => {
                LeverState::Selected(index.min(normalized_count.saturating_sub(1)))
            }
            LeverState::Random => LeverState::Random,
        };
        Self(normalized_state, normalized_count)
    }

    pub fn set_state_and_options(&mut self, state: LeverState, option_count: usize) {
        let normalized_count = option_count.max(LEVER_DEFAULT_OPTIONS);
        self.0 = match state {
            LeverState::Selected(index) => {
                LeverState::Selected(index.min(normalized_count.saturating_sub(1)))
            }
            LeverState::Random => LeverState::Random,
        };
        self.1 = normalized_count;
    }

    fn normalized_option_count(&self) -> usize {
        self.1.max(LEVER_DEFAULT_OPTIONS)
    }

    pub fn selected_index(&self) -> Option<usize> {
        let option_count = self.normalized_option_count();
        match self.0 {
            LeverState::Selected(index) => Some(index.min(option_count.saturating_sub(1))),
            LeverState::Random => None,
        }
    }

    pub fn set_selected_index(&mut self, selected_index: Option<usize>) {
        self.0 = match selected_index {
            Some(index) => LeverState::Selected(index.min(self.normalized_option_count() - 1)),
            None => LeverState::Random,
        };
    }

    fn shaft_angle_radians(&self) -> f32 {
        let Some(selected_index) = self.selected_index() else {
            return 0.0;
        };
        let option_count = self.normalized_option_count();
        if option_count <= 1 {
            return 0.0;
        }
        let step = (LEVER_MAX_ANGLE_DEG - LEVER_MIN_ANGLE_DEG) / (option_count - 1) as f32;
        // Option 1 should point left; higher option indices move toward right.
        let angle_deg = LEVER_MAX_ANGLE_DEG - step * selected_index as f32;
        angle_deg.to_radians()
    }

    pub fn update(
        lever: Option<Res<Lever>>,
        mut base_query: Query<
            (&mut TextColor, &mut ClickableCursorIcons),
            (With<LeverRoot>, With<LeverBase>),
        >,
        mut shaft_pivot_query: Query<&mut Transform, (With<LeverShaftPivot>, Without<LeverBase>)>,
        mut shaft_query: Query<&mut TextColor, (With<LeverShaft>, Without<LeverBase>)>,
    ) {
        let lever = match lever {
            Some(lever) => lever,
            None => return,
        };

        let selected_index = lever.selected_index();
        let color = selected_index.map_or(Color::WHITE, option_color);
        let hover_cursor = match selected_index {
            Some(0) => CursorMode::PullRight,
            Some(_) => CursorMode::PullLeft,
            None => CursorMode::PullLeft,
        };
        for (mut text_color, mut icons) in base_query.iter_mut() {
            text_color.0 = color;
            icons.on_hover = hover_cursor;
        }
        for mut text_color in shaft_query.iter_mut() {
            text_color.0 = color;
        }

        let shaft_rotation = Quat::from_rotation_z(lever.shaft_angle_radians());
        for mut transform in shaft_pivot_query.iter_mut() {
            transform.rotation = shaft_rotation;
        }
    }
}

impl Default for Lever {
    fn default() -> Self {
        Lever(LeverState::Random, LEVER_DEFAULT_OPTIONS)
    }
}
