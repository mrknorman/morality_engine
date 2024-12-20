use bevy::prelude::*;

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InheritanceSystemsActive {
    #[default]
	False,
    True
}
pub struct InheritancePlugin;

impl Plugin for InheritancePlugin {
    fn build(&self, app: &mut App) {	
		app
		.init_state::<InheritanceSystemsActive>()
		.add_systems(
			Update,
			activate_systems
		).add_systems(
            Update,
            (
				BequeathTextColor::bequeath
            )
            .run_if(in_state(InheritanceSystemsActive::True))
        );
    }
}

fn activate_systems(
	mut inheritance_state: ResMut<NextState<InheritanceSystemsActive>>,
	inheritance_query: Query<&BequeathTextColor>
) {
	if !inheritance_query.is_empty() {
		inheritance_state.set(InheritanceSystemsActive::True)
	} else {
		inheritance_state.set(InheritanceSystemsActive::False)
	}
}

#[derive(Component)]
#[require(TextColor)]
pub struct BequeathTextColor;

impl BequeathTextColor {
    fn bequeath(
        parent_query : Query<(&Children, &TextColor), With<BequeathTextColor>>,
        mut child_query : Query<&mut TextColor, Without<BequeathTextColor>>
    ) {

        for (children, parent_color) in parent_query.iter() {
            for &child in children.iter() {

                if let Ok(mut child_color) = child_query.get_mut(child) {
                    child_color.0 = parent_color.0;
                }
            }
        }
    }
}
