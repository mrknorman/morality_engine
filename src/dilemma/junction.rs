use std::iter::zip;

use bevy::{
    ecs::component::StorageType, prelude::*,
};

use crate::{
    dilemma::{
        Dilemma,
        dilemma::LeverTrackTransform
    },
    dilemma::lever::{
	    Lever, 
        LeverState
    },
    person::{
        PersonSprite,
        Emoticon
    },
    colors::{
        ColorTranslation,
        ColorChangeOn,
        ColorChangeEvent,
        ColorAnchor,
        OPTION_1_COLOR,
        OPTION_2_COLOR,
        DANGER_COLOR
    },
    audio::{
        TransientAudioPallet,
        TransientAudio
    },
    track::Track,
    motion::Bounce,
    inheritance::BequeathTextColor
};


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

					let track_colors = vec![OPTION_1_COLOR, OPTION_2_COLOR];
					let branch_y_positions = vec![0.0, 100.0];			

					let initial_y_position = match dilemma.default_option {
						None =>  50.0,
						Some(ref option) => branch_y_positions[*option],
					};	
			
					let mut track_entities = vec![];
					commands.entity(entity).with_children(
						|junction| {
							let mut junction_entity = junction.spawn((
								TrunkTrack,
								Track::new(600),
								Transform::from_xyz(-2000.0, initial_y_position, 1.0),			
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
								LeverTrackTransform{
									branch_y_positions : branch_y_positions.clone()
								}
							)).with_children( |turnout| {
								for (branch_index, ((option, y_position), color)) in zip(
                                    zip(dilemma.options, branch_y_positions.clone()), track_colors
                                ).enumerate() {
									track_entities.push(turnout.spawn((
										BranchTrack{index : branch_index},
										Track::new(300),
										TextColor(color),
										Transform::from_xyz(0.0, y_position, 0.0)		
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
																audio_vector.clone()
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

pub fn switch_junction(
    mut movement_query: Query<(&mut LeverTrackTransform, &mut Transform)>,
    mut track_query: Query<&mut TextColor, With<TrunkTrack>>,
    lever: Option<Res<Lever>>,
    time: Res<Time>,  // Add time resource to manage frame delta time
) {

if lever.is_some() {

    let unwrapped_lever: Res<Lever> = lever.unwrap();

    for (
        lever_transform, 
        mut transform
    ) in movement_query.iter_mut() {
        let left_position: Vec3 = Vec3::new(transform.translation.x, -lever_transform.branch_y_positions[0], 1.0);
        let right_position: Vec3 = Vec3::new(transform.translation.x, -lever_transform.branch_y_positions[1], 1.0);

        let distance_threshold = 0.01; // Small threshold to determine when to snap to the final position
        let proportional_speed_factor = 0.1; // Factor to adjust the proportional speed
        let bounce_amplitude = 0.02; // Amplitude of the bounce effect
        let bounce_frequency = 10.0; // Frequency of the bounce effect

        let main_track = track_query.get_single_mut(); 

        if unwrapped_lever.0 == LeverState::Right {
            main_track.unwrap().0 = OPTION_2_COLOR;
            let distance = (right_position - transform.translation).length();
            if distance > distance_threshold {
                let direction = (right_position - transform.translation).normalize();
                let movement_speed = distance * proportional_speed_factor;
                transform.translation += direction * movement_speed;
            } else {
                let bounce_offset = bounce_amplitude * (time.elapsed_secs() * bounce_frequency).sin();
                transform.translation = right_position + Vec3::new(bounce_offset, 0.0, 0.0);
            }
        } else if unwrapped_lever.0 == LeverState::Left {
            main_track.unwrap().0 = OPTION_1_COLOR;
            let distance = (left_position - transform.translation).length();
            if distance > distance_threshold {
                let direction = (left_position - transform.translation).normalize();
                let movement_speed = distance * proportional_speed_factor;
                transform.translation += direction * movement_speed;
            } else {
                let bounce_offset = bounce_amplitude * (time.elapsed_secs() * bounce_frequency).sin();
                transform.translation = left_position + Vec3::new(bounce_offset, 0.0, 0.0);
            }
        }
    }
} else {
    panic!("Lever motion check with non-existant lever!")
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