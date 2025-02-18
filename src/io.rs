use bevy::{
	ecs::component::StorageType, prelude::*
};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum IOSystemsActive {
    #[default]
    False,
    True,
}

pub struct IOPlugin;
impl Plugin for IOPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<IOSystemsActive>()
            .add_systems(Update, activate_systems)
            .add_systems(
                Update,
                (
                    BottomAnchor::bottom_anchor,
                    CenterXAnchor::center_x_anchor,
                )
                .run_if(in_state(IOSystemsActive::True)),
            );
    }
}

fn activate_systems(
    mut anchor_state: ResMut<NextState<IOSystemsActive>>,
    anchor_query: Query<&BottomAnchor>,
	center_query: Query<&CenterXAnchor>,

) {
    if !anchor_query.is_empty() || !center_query.is_empty() {
        anchor_state.set(IOSystemsActive::True);
    } else {
        anchor_state.set(IOSystemsActive::False);
    }
}

#[derive(Component)]
pub struct BottomAnchor {
    distance: f32,
}

impl Default for BottomAnchor {
    fn default() -> Self {
        BottomAnchor{
            distance: 100.0
        }
    }
}

impl BottomAnchor {
    fn bottom_anchor(
        windows: Query<&Window>,
        mut query: Query<(&BottomAnchor, &mut Transform, Option<&Parent>)>,
        parent_query: Query<&Transform, Without<BottomAnchor>>, // Query for parent Transform
    ) {
        // Return early if there's not exactly one window
        let window = match windows.get_single() {
            Ok(window) => window,
            Err(_) => {
                return;
            }
        };

        let screen_height = window.height();

        for (bottom_anchor, mut transform, parent) in query.iter_mut() {
            let mut button_y = -screen_height / 2.0 + bottom_anchor.distance;

            // If the entity has a parent, subtract the parent's position.
            if let Some(parent) = parent {
                if let Ok(parent_transform) = parent_query.get(parent.get()) {
                    button_y -= parent_transform.translation.y;
                }
            }

            transform.translation.y = button_y;
        }
    }
}


pub struct CenterXAnchor;

impl Default for CenterXAnchor {
    fn default() -> Self {
        CenterXAnchor
    }
}

impl CenterXAnchor {

    fn center_x_anchor(
        mut query: Query<(&CenterXAnchor, &mut Transform, &mut Visibility, Option<&Parent>)>,
        parent_query: Query<&Transform, Without<CenterXAnchor>>, // Query for parent Transform
    ) {
        for (_center_x_anchor, mut transform, mut visibility, parent) in query.iter_mut() {
            let mut button_x = 0.0; // Centered on the X axis

            // If the entity has a parent, subtract the parent's position
            if let Some(parent) = parent {
                if let Ok(parent_transform) = parent_query.get(parent.get()) {
                    button_x -= parent_transform.translation.x;
                }
            }

			*visibility = Visibility::Inherited;

            transform.translation.x = button_x;
        }
    }
}

impl Component for CenterXAnchor {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {        
                // Get the component ID for Visibility
                // Try to get mutable access to the Visibility component
                if let Some(mut visibility) = world.get_mut::<Visibility>(entity) {
                    // Modify the Visibility component
                    *visibility = Visibility::Hidden;
                } else {
                    // If the entity doesn't have Visibility, add it
                    world.commands().entity(entity).insert( Visibility::Hidden);
                }
            }
        );
    }
}
