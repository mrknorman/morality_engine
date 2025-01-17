use std::iter::zip;

use bevy::{
    ecs::component::StorageType, prelude::*,
};

use crate::{
    audio::{
        DilatableAudio, 
		TransientAudio, 
		TransientAudioPallet
    }, colors::{
        ColorAnchor, 
		ColorChangeEvent, 
		ColorChangeOn, 
		ColorTranslation, 
		DANGER_COLOR,
		OPTION_1_COLOR, 
		OPTION_2_COLOR
    }, dilemma::{
		lever::{
	    	Lever, 
       		LeverState
    	}, 
		Dilemma
	}, 
	inheritance::BequeathTextColor, 
	motion::{
		Bounce, 
		TransformMultiAnchor
	}, 
	person::{
        Emoticon, 
		PersonSprite
    }, 
	time::Dilation, 
	track::Track
};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum JunctionSystemsActive {
    #[default]
	False,
    True
}

pub struct JunctionPlugin;
impl Plugin for JunctionPlugin {
    fn build(&self, app: &mut App) {	
		app
		.init_state::<JunctionSystemsActive>()
		.add_systems(
			Update,
			activate_systems
		).add_systems(
            Update,
            (
				Junction::switch_junction,
				Junction::check_if_person_in_path_of_train
			)
            .run_if(in_state(JunctionSystemsActive::True))
        )
		.register_required_components::<Junction, Transform>()
		.register_required_components::<Junction, Visibility>()
		;
    }
}

fn activate_systems(
        mut state: ResMut<NextState<JunctionSystemsActive>>,
        query: Query<&Junction>
    ) {
        
	if !query.is_empty() {
		state.set(JunctionSystemsActive::True)
	} else {
		state.set(JunctionSystemsActive::False)
	}
}
#[derive(Component)]
pub struct TrunkTrack;

#[derive(Component)]
pub struct BranchTrack{
	index : usize
}

#[derive(Component)]
#[require(Visibility, Transform)]
pub struct Turnout;

#[derive(Clone)]
pub struct Junction{
	pub dilemma : Dilemma
}

impl Component for Junction {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {
				let junction: Option<Junction> = {
                    let entity_mut = world.entity(entity);
                    entity_mut.get::<Junction>()
                        .map(|train: &Junction| train.clone())
                };

				let color: Option<TextColor> = {
                    let entity_mut = world.entity(entity);
                    entity_mut.get::<TextColor>()
                        .map(|color: &TextColor| color.clone())
                };
				
				let color_translation: Option<ColorTranslation> = {
                    let entity_mut = world.entity(entity);
                    entity_mut.get::<ColorTranslation>()
                        .map(|translation: &ColorTranslation| translation.clone())
                };

				let asset_server = world.get_resource::<AssetServer>().unwrap();
				let audio_vector = vec![											
					TransientAudio::new(
						asset_server.load("sounds/male_scream_1.ogg"),
						1.0,
						false,
						1.0
					),
					TransientAudio::new(
						asset_server.load("sounds/male_scream_2.ogg"),
						1.0,
						false,
						1.0
					),
					TransientAudio::new(
						asset_server.load("sounds/male_scream_3.ogg"),
						1.0,
						false,
						1.0
					)
				];

				let mut commands = world.commands();
				if let Some(junction) = junction {
					let dilemma = junction.dilemma; 

					let track_colors: Vec<Color> = vec![OPTION_1_COLOR, OPTION_2_COLOR];
					let branch_y_positions = vec![
						Transform::from_xyz(0.0, 0.0, 1.0),
						Transform::from_xyz(0.0, 100.0, 1.0)	
					];

					let mut initial_y_position = match dilemma.default_option {
						None =>  Transform::from_xyz(0.0, 50.0, 1.0),
						Some(ref option) => branch_y_positions[*option],
					};	

					initial_y_position.translation.x -= 2000.0;
			
					let mut track_entities = vec![];
					commands.entity(entity).with_children(
						|junction| {
							let mut junction_entity = junction.spawn((
								TrunkTrack,
								Track::new(600),
								initial_y_position
							));
							
							if let Some(color) = color {
								junction_entity.insert(color);
							}
							if let Some(color_translation) = color_translation {
								junction_entity.insert(color_translation);
							}
							track_entities.push(junction_entity.id());

							let turnout_entity = junction.spawn((
								Turnout,
								Transform::from_xyz( 1240.0, 0.0, 0.0),
								TransformMultiAnchor(branch_y_positions.clone())
							)).with_children( |turnout| {
								for (branch_index, ((option, y_position), color)) in zip(
                                    zip(dilemma.options, branch_y_positions.clone()), track_colors
                                ).enumerate() {
									track_entities.push(turnout.spawn((
										BranchTrack{index : branch_index},
										Track::new(300),
										TextColor(color),
										y_position		
									)).with_children(|track: &mut ChildBuilder<'_>| {
											for fatality_index in 0..option.consequences.total_fatalities {
												track.spawn(
													(
														PersonSprite::default(),
														Transform::from_xyz(
															-1060.0 + fatality_index as f32 * 10.0,
															0.0,
															0.0 
														),
														TransientAudioPallet::new(
															vec![
																("exclamation".to_string(),
																audio_vector.clone(),
																Some(DilatableAudio)
															)]
														),
														Bounce::new(
															false,
															40.0, 
															60.0,
															1.0,
															2.0
														),
														ColorChangeOn::new(vec![ColorChangeEvent::Bounce(vec![DANGER_COLOR])]),
														ColorAnchor::default(),
														BequeathTextColor
													)
												).with_children(
													|parent| {
														parent.spawn(
															Emoticon::default()
														);
													}
												);	
											}
										}
									).id());
								}
							}).id();

							track_entities.push(turnout_entity);
						}
					);
				}
			}
        );
    }
}

impl Junction  {
	
	pub fn switch_junction(
		time : Res<Time>,
		dilation : Res<Dilation>,
		mut movement_query: Query<(&TransformMultiAnchor, &mut Transform), With<Turnout>>,
		mut track_query: Query<&mut TextColor, With<TrunkTrack>>,
		lever: Option<Res<Lever>>,
	) {
		// Early return if lever is missing
		let Some(lever) = lever else {
			return;
		};
		
		// Get main track once before the loop
		let Ok(mut main_track) = track_query.get_single_mut() else {
			return;
		};
	
		const DISTANCE_THRESHOLD: f32 = 0.01;
		const PROPORTIONAL_SPEED_FACTOR: f32 = 10.0;
	
		for (lever_transform, mut transform) in movement_query.iter_mut() {
			let target_position = match lever.0 {
				LeverState::Right => {
					main_track.0 = OPTION_2_COLOR;
					Vec3::new(
						transform.translation.x,
						-lever_transform.0[1].translation.y,
						1.0,
					)
				}
				LeverState::Left => {
					main_track.0 = OPTION_1_COLOR;
					Vec3::new(
						transform.translation.x,
						-lever_transform.0[0].translation.y,
						1.0,
					)
				}
				LeverState::Random => return
			};
	
			let distance = (target_position - transform.translation).length();
			
			if distance > DISTANCE_THRESHOLD {
				let direction = (target_position - transform.translation).normalize();
				let movement_speed = distance * PROPORTIONAL_SPEED_FACTOR * time.delta_secs()*dilation.0;
				transform.translation += direction * movement_speed;
			} else {
				transform.translation = target_position;
			}
		}
	}

	pub fn check_if_person_in_path_of_train(
		mut lever_query: Query<(&Children, &BranchTrack)>,
		mut text_query: Query<&mut PersonSprite>,
	lever: Option<Res<Lever>>
	) {
		if let Some(lever) = lever {		
			for (children, track) in lever_query.iter_mut(){
				for &child in children.iter() {
					if let Ok(mut person) = text_query.get_mut(child) {
						person.in_danger = (Some(track.index) == lever.0.to_int()) && !(lever.0 == LeverState::Random);
					}
				}
			}
		}
	}
}