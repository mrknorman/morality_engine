use bevy::prelude::*;

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum IOSystemsActive {
    #[default]
    False,
    True
}

pub struct IOPlugin;
impl Plugin for IOPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_state::<IOSystemsActive>()
        .add_systems(
            Update,
            activate_systems
        ).add_systems(
            Update,
            (
                BottomAnchor::bottom_anchor
            )
            .run_if(
                in_state(IOSystemsActive::True)
            )
        );
    }
}

fn activate_systems(
	mut anchor_state: ResMut<NextState<IOSystemsActive>>,
	anchor_query: Query<&BottomAnchor>,
) {
	if !anchor_query.is_empty() {
		anchor_state.set(IOSystemsActive::True)
	} else {
		anchor_state.set(IOSystemsActive::False)
	}
}

#[derive(Component)]
pub struct BottomAnchor {
    distance: f32,
}

impl BottomAnchor {

	pub fn new(distance : f32) -> Self {
		Self {
			distance
		}
	}

	fn bottom_anchor(
		windows: Query<&Window>,
		mut query: Query<(&BottomAnchor, &mut Transform)>,
	) {
		let window = windows.get_single().unwrap();
		let screen_height = window.height();
	
		for (bottom_anchor, mut transform) in query.iter_mut() {
			if screen_height >= 3.0 * bottom_anchor.distance {
				let button_y = -screen_height / 2.0 + bottom_anchor.distance;
				transform.translation.y = button_y;
			}
		}
	}
}