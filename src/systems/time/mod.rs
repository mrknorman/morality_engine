use std::time::Duration;

use bevy::{
    ecs::{component::{
        ComponentHooks, HookContext, Mutable, StorageType
    }, world::DeferredWorld},
    prelude::*
};


#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DilationSystemsActive {
    #[default]
	False,
    True
}


#[derive(Resource)]
pub struct Dilation(pub f32);

pub struct DilationPLugin;
impl Plugin for DilationPLugin {
    fn build(&self, app: &mut App) {
        app
        .init_state::<DilationSystemsActive>()
        .add_systems(
			Update,
			activate_systems
		)
        .add_systems(
            Update,
            (
                DilationTranslation::enact
            )
            .run_if(in_state(DilationSystemsActive::True))
        ) 
        .insert_resource(Dilation(1.0));
    }
}

fn activate_systems(
    mut state: ResMut<NextState<DilationSystemsActive>>,
    query: Query<&DilationTranslation>
) {
    if !query.is_empty() {
        state.set(DilationSystemsActive::True)
    } else {
        state.set(DilationSystemsActive::False)
    }
}

#[derive(Component)]
#[component(on_insert = DilationTranslation::on_insert)]
pub struct DilationTranslation {
    pub initial_dilation: f32,
    pub final_dilation: f32,
	pub timer : Timer
}

impl DilationTranslation {
    pub fn new(final_dilation: f32, duration: Duration) -> DilationTranslation {
		DilationTranslation {
			initial_dilation : 1.0,
			final_dilation,
			timer : Timer::new(
				duration,
				TimerMode::Once
			)
		}
	}

	pub fn enact(
        mut commands : Commands,
		time: Res<Time>, 
		mut dilation : ResMut<Dilation>,
		mut query: Query<(Entity, &mut DilationTranslation)>
	) {
		for (entity, mut motion) in query.iter_mut() {
			motion.timer.tick(time.delta());
			if !motion.timer.paused() && !motion.timer.finished() {
				let fraction_complete = motion.timer.fraction();
				let difference = motion.final_dilation - motion.initial_dilation;
				dilation.0 = motion.initial_dilation + difference*fraction_complete;
			} else if motion.timer.finished() {
				dilation.0 = motion.final_dilation;
                commands.entity(entity).despawn();    
			}
		}
	}

    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        let dilation = world.get_resource::<Dilation>().map(|d| d.0);

        match dilation {
            Some(dilation) => {
                if let Some(mut dilation_translation) = world.entity_mut(entity).get_mut::<DilationTranslation>() {
                    dilation_translation.initial_dilation = dilation;
                } else {
                    warn!(
                        "Failed to retrieve TransformAnchor component for entity: {:?}", 
                        entity
                    );
                }
            }
            None => {
                warn!(

                    "Transform anchor should be inserted after transform! Unable to find Transform.");
            }
        }

        
    }
}