use std::iter::zip;

use bevy::{
    audio::Volume, ecs::{component::HookContext, world::DeferredWorld}, prelude::*
};

use crate::{
    entities::{
		person::{
			BloodSprite, Emoticon, EmotionSounds, PersonSprite
		}, text::{CharacterSprite, TextSprite}, track::Track
	}, scenes::dilemma::{
		Dilemma, dilemma::CurrentDilemmaStage, lever::{
	    	Lever, 
       		LeverState
    	}
	},
	systems::{
		audio::{
			ContinuousAudio, ContinuousAudioPallet, TransientAudio, TransientAudioPallet, continuous_audio
		}, colors::{
			ColorAnchor, 
			ColorChangeEvent, 
			ColorChangeOn, 
			ColorTranslation, 
			DANGER_COLOR,
			OPTION_1_COLOR, 
			OPTION_2_COLOR
		}, 
		inheritance::BequeathTextColor,
		motion::{
			Bounce, 
			TransformMultiAnchor
		}, time::Dilation 
	}  
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
        ).add_systems(
			Update,
			Junction::update_main_track_color
			.run_if(in_state(JunctionSystemsActive::True))
			.run_if(resource_changed::<Lever>)
		);
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

#[derive(Clone, Component)]
#[require(Transform, Visibility)]
#[component(on_insert = Junction::on_insert)]
pub struct Junction{
	pub dilemma : Dilemma
}

impl Junction {
	const BRANCH_SEPARATION : Vec3 = Vec3::new(0.0, -100.0, 0.0);
	const TRUNK_TRANSLATION : Vec3 = Vec3::new(-11800.0, 0.0, 0.0);
	const TURNOUT_TRANSLATION : Vec3 = Vec3::new(2000.0, 0.0, 0.0);
	const FATALITY_OFFSET : f32 = -1780.0;

	fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
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
				asset_server.load("./audio/effects/male_scream_1.ogg"),
				1.0,
				false,
				0.5,
				true
			),
			TransientAudio::new(
				asset_server.load("./audio/effects/male_scream_2.ogg"),
				1.0,
				false,
				0.5,
				true
			),
			TransientAudio::new(
				asset_server.load("./audio/effects/male_scream_3.ogg"),
				1.0,
				false,
				0.5,
				true
			)
		];

		let splat_audio = vec![
			TransientAudio::new(
				asset_server.load("./audio/effects/blood.ogg"),
				1.0,
				true,
				0.5,
				true
			)
		];

		
		let pop_audio = vec![
			TransientAudio::new(
				asset_server.load("./audio/effects/pop.ogg"),
				1.0,
				true,
				0.7,
				true
			)
		];

		let crowd_audio = asset_server.load(
			"./audio/effects/crowd_panic.ogg"
		);

		let current_stage = world.get_resource::<CurrentDilemmaStage>().expect("No current stage found!").clone().0;
		let mut commands = world.commands();
		if let Some(junction) = junction {
			let dilemma: Dilemma = junction.dilemma; 

			let track_colors: Vec<Color> = vec![OPTION_1_COLOR, OPTION_2_COLOR];
			let branch_y_positions = vec![
				Transform::default(),
				Transform::from_translation(Junction::BRANCH_SEPARATION)
			];
			
			let mut track_entities = vec![];
			commands.entity(entity).with_children(
				|junction| {
					let mut trunk_entity = junction.spawn((
						TrunkTrack,
						Track::new(2000),
						Transform::from_translation(Junction::TRUNK_TRANSLATION),
					));
					
					if let Some(color) = color {
						trunk_entity.insert(color);
					}
					if let Some(color_translation) = color_translation {
						trunk_entity.insert(color_translation);
					}
					track_entities.push(trunk_entity.id());

					let turnout_entity = junction.spawn((
						Turnout,
						Transform::from_translation(Junction::TURNOUT_TRANSLATION),
						TransformMultiAnchor(branch_y_positions.clone())
					)).with_children( |turnout| {
						for (branch_index, ((option, y_position), color)) in zip(
							zip(dilemma.stages[current_stage].options.clone(), branch_y_positions.clone()), track_colors
						).enumerate() {
							track_entities.push(turnout.spawn((
								BranchTrack{index : branch_index},
								Track::new(300),
								TextColor(color),
								y_position		
							)).with_children(|track: &mut ChildSpawnerCommands<'_>| {

									if option.consequences.total_fatalities >= 500 {
										track.spawn((
											CrowdPanicNoise,
											ContinuousAudioPallet::new(
												vec![
													ContinuousAudio{
														key : EmotionSounds::Exclaim,
														source : AudioPlayer::<AudioSource>(crowd_audio.clone()), 
														settings : PlaybackSettings{
															volume : Volume::Linear(0.5),
															..continuous_audio()
														},
														dilatable : true
													}
												]
											)
										));
									}

									for fatality_index in 0..option.consequences.total_fatalities {
										let transform = if fatality_index >= 450 {
											break;
										} else {
											// Define parameters
											let columns_per_row = 150;
											let row_height = 10.0;
											let column_width = 10.0;
											
											// Calculate position
											let row = fatality_index / columns_per_row;
											let col = fatality_index % columns_per_row;
											
											let row_x_adjustment = if row % 2 == 1 { 5.0 } else { 0.0 };
											
											Transform::from_xyz(
												Junction::FATALITY_OFFSET + row_x_adjustment + col as f32 * column_width,
												row as f32 * row_height,
												0.0
											)
										};

										let mut entity_commands = track.spawn(
											(
												transform,
												GlobalTransform::from_xyz(1000.0, 1000.0, 1000.0), //Here to avoid immediate collision error
												TextSprite,
												PersonSprite::default(),
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
										);
										
										entity_commands.with_children(
											|parent| {
												parent.spawn(
													Emoticon::default()
												);
											}
										);	

										if fatality_index < 5 {
											entity_commands.insert(
												TransientAudioPallet::new(
													vec![
														(
														EmotionSounds::Exclaim,
														audio_vector.clone()
													),
													(
														EmotionSounds::Splat,
														splat_audio.clone()
													),
													(
														EmotionSounds::Pop,
														pop_audio.clone()
													)										
													]
												)
											);
										} else {
											entity_commands.insert(
												TransientAudioPallet::new(
													vec![
														(
															EmotionSounds::Splat,
															splat_audio.clone()
														),
														(
															EmotionSounds::Pop,
															pop_audio.clone()
														)
													]
												)
											);
										}
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

	pub fn cleanup(
		mut commands : Commands,
		junction_query : Query<Entity, With<Junction>>,
		body_parts_query : Query<Entity, (With<CharacterSprite>, Without<ChildOf>)>,
		blood_query : Query<Entity, With<BloodSprite>>,
	){
		for entity in junction_query.iter() {
			commands.entity(entity).despawn();
		}

		for entity in body_parts_query.iter() {
            if let Ok(mut entity_cmds) = commands.get_entity(entity) {
				entity_cmds.despawn();
			}
        }

		for entity in blood_query.iter() {
			commands.entity(entity).despawn();
		}
	}

	pub fn update_main_track_color(
		lever: Option<Res<Lever>>,
		mut track_query: Query<&mut TextColor, With<TrunkTrack>>,
	) {
		for mut track in track_query.iter_mut() {
			if let Some(lever) = &lever {
				let lever = lever.0;
				let color = match lever {
					LeverState::Left => OPTION_1_COLOR,
					LeverState::Right => OPTION_2_COLOR,
					LeverState::Random => return
				};
				track.0 = color;
			}
		}
		
	}
	
	pub fn switch_junction(
		time : Res<Time>,
		dilation : Res<Dilation>,
		mut movement_query: Query<(&TransformMultiAnchor, &mut Transform), With<Turnout>>,
		lever: Option<Res<Lever>>,
	) {
		// Early return if lever is missing
		let Some(lever) = lever else {
			return;
		};
	
		const DISTANCE_THRESHOLD: f32 = 0.01;
		const PROPORTIONAL_SPEED_FACTOR: f32 = 10.0;
	
		for (lever_transform, mut transform) in movement_query.iter_mut() {
			let target_position = match lever.0 {
				LeverState::Right => {
					Vec3::new(
						transform.translation.x,
						-lever_transform.0[1].translation.y,
						1.0,
					)
				}
				LeverState::Left => {
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
		mut crowd_audio: Query<&mut ContinuousAudioPallet<EmotionSounds>>,
		lever: Option<Res<Lever>>
	) {
		if let Some(lever) = lever {		
			for (children, track) in lever_query.iter_mut(){
				for child in children.iter() {
					if let Ok(mut person) = text_query.get_mut(child) {
						person.in_danger = (Some(track.index) == lever.0.to_int()) && !(lever.0 == LeverState::Random);
					}
				}
			}
		}
	}
}

#[derive(Component)]
pub struct CrowdPanicNoise;