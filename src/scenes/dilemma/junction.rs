use std::iter::zip;

use bevy::{
    audio::Volume,
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{
    entities::{
        person::{BloodSprite, Emoticon, EmotionSounds, PersonSprite},
        text::{scaled_text_units, CharacterSprite, TextSprite},
        track::Track,
        train::Train,
    },
    scenes::dilemma::{dilemma::DilemmaStage, lever::Lever},
    systems::{
        audio::{
            continuous_audio, ContinuousAudio, ContinuousAudioPallet, OneShotAudio,
            OneShotAudioPallet, TransientAudio, TransientAudioPallet,
        },
        colors::{
            option_color, ColorAnchor, ColorChangeEvent, ColorChangeOn, ColorTranslation,
            BACKGROUND_COLOR, DANGER_COLOR,
        },
        inheritance::BequeathTextColor,
        motion::{Bounce, TransformMultiAnchor},
        physics::DespawnOffscreen,
        time::Dilation,
    },
};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum JunctionSystemsActive {
    #[default]
    False,
    True,
}

pub struct JunctionPlugin;
impl Plugin for JunctionPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<JunctionSystemsActive>()
            .add_systems(Update, activate_systems)
            .add_systems(
                Update,
                (
                    Junction::switch_junction,
                    Junction::check_if_person_in_path_of_train,
                    Junction::sync_mass_panic_audio,
                    Junction::play_mass_rescue_cheer_on_lever_pull,
                )
                    .chain()
                    .run_if(in_state(JunctionSystemsActive::True)),
            )
            .add_systems(
                Update,
                Junction::update_main_track_color
                    .run_if(in_state(JunctionSystemsActive::True))
                    .run_if(resource_changed::<Lever>),
            );
    }
}

fn activate_systems(mut state: ResMut<NextState<JunctionSystemsActive>>, query: Query<&Junction>) {
    if !query.is_empty() {
        state.set(JunctionSystemsActive::True)
    } else {
        state.set(JunctionSystemsActive::False)
    }
}
#[derive(Component)]
pub struct TrunkTrack;

#[derive(Component)]
pub struct BranchTrack {
    index: usize,
}

#[derive(Component)]
#[require(Visibility, Transform)]
pub struct Turnout;

#[derive(Clone, Component)]
#[require(Transform, Visibility, TextColor = Junction::track_color(), DespawnOffscreen = Junction::despawn_margin())]
#[component(on_insert = Junction::on_insert)]
pub struct Junction {
    pub stage: DilemmaStage,
}

impl Junction {
    pub const BRANCH_SEPARATION: Vec3 = Vec3::new(0.0, scaled_text_units(-100.0), 0.0);
    const TRUNK_TRANSLATION: Vec3 =
        Vec3::new(scaled_text_units(-7000.0) / Track::LENGTH as f32, 0.0, 0.0);
    const TURNOUT_TRANSLATION: Vec3 = Vec3::new(scaled_text_units(1281.5), 0.0, 0.0);
    const FATALITY_OFFSET: f32 = scaled_text_units(-1070.0);
    const MASS_RESCUE_CHEER_SOUND: &str = "./audio/effects/cheer.ogg";
    const MASS_RESCUE_CHEER_THRESHOLD: usize = 100;
    const MASS_PANIC_SOUND: &str = "./audio/effects/crowd_panic.ogg";
    const MASS_PANIC_THRESHOLD: usize = 100;

    fn track_color() -> TextColor {
        TextColor(BACKGROUND_COLOR)
    }

    fn despawn_margin() -> DespawnOffscreen {
        DespawnOffscreen { margin: 10.0 }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let junction: Option<Junction> = {
            let entity_mut = world.entity(entity);
            entity_mut
                .get::<Junction>()
                .map(|train: &Junction| train.clone())
        };

        let color: Option<TextColor> = {
            let entity_mut = world.entity(entity);
            entity_mut
                .get::<TextColor>()
                .map(|color: &TextColor| color.clone())
        };

        let color_translation: Option<ColorTranslation> = {
            let entity_mut = world.entity(entity);
            entity_mut
                .get::<ColorTranslation>()
                .map(|translation: &ColorTranslation| translation.clone())
        };

        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let audio_vector = vec![
            TransientAudio::new(
                asset_server.load("./audio/effects/male_scream_1.ogg"),
                1.0,
                false,
                0.3,
                true,
            ),
            TransientAudio::new(
                asset_server.load("./audio/effects/male_scream_2.ogg"),
                1.0,
                false,
                0.3,
                true,
            ),
            TransientAudio::new(
                asset_server.load("./audio/effects/male_scream_3.ogg"),
                1.0,
                false,
                0.3,
                true,
            ),
        ];

        let splat_audio = vec![TransientAudio::new(
            asset_server.load("./audio/effects/blood.ogg"),
            1.0,
            true,
            0.5,
            true,
        )];

        let pop_audio = vec![TransientAudio::new(
            asset_server.load("./audio/effects/pop.ogg"),
            1.0,
            true,
            0.7,
            true,
        )];

        let mut commands = world.commands();
        if let Some(junction) = junction {
            let stage: DilemmaStage = junction.stage;
            let branch_y_positions: Vec<Transform> = (0..stage.options.len())
                .map(|branch_index| {
                    Transform::from_translation(Junction::BRANCH_SEPARATION * branch_index as f32)
                })
                .collect();

            let mut track_entities = vec![];
            commands.entity(entity).with_children(|junction| {
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

                let turnout_entity = junction
                    .spawn((
                        Turnout,
                        Transform::from_translation(Junction::TURNOUT_TRANSLATION),
                        TransformMultiAnchor(branch_y_positions.clone()),
                    ))
                    .with_children(|turnout| {
                        for (branch_index, (option, y_position)) in
                            zip(stage.options.clone(), branch_y_positions.clone()).enumerate()
                        {
                            track_entities.push(
                                turnout
                                    .spawn((
                                        BranchTrack {
                                            index: branch_index,
                                        },
                                        Track::new(300),
                                        TextColor(option_color(branch_index)),
                                        y_position,
                                    ))
                                    .with_children(|track: &mut ChildSpawnerCommands<'_>| {
                                        for fatality_index in
                                            0..option.consequences.total_fatalities
                                        {
                                            let transform = if fatality_index >= 450 {
                                                break;
                                            } else {
                                                // Define parameters
                                                let columns_per_row = 150;
                                                let row_height = scaled_text_units(10.0);
                                                let column_width = scaled_text_units(10.0);

                                                // Calculate position
                                                let row = fatality_index / columns_per_row;
                                                let col = fatality_index % columns_per_row;

                                                let row_x_adjustment = if row % 2 == 1 {
                                                    scaled_text_units(5.0)
                                                } else {
                                                    0.0
                                                };

                                                Transform::from_xyz(
                                                    Junction::FATALITY_OFFSET
                                                        + row_x_adjustment
                                                        + col as f32 * column_width,
                                                    row as f32 * row_height,
                                                    0.0,
                                                )
                                            };

                                            let mut entity_commands = track.spawn((
                                                transform,
                                                GlobalTransform::from_xyz(
                                                    100000.0, 100000.0, 100000.0,
                                                ), //Here to avoid immediate collision error
                                                TextSprite,
                                                PersonSprite::default(),
                                                Bounce::new(false, 40.0, 60.0, 1.0, 2.0),
                                                ColorChangeOn::new(vec![ColorChangeEvent::Bounce(
                                                    vec![DANGER_COLOR],
                                                )]),
                                                ColorAnchor::default(),
                                                BequeathTextColor,
                                            ));

                                            entity_commands.with_children(|parent| {
                                                parent.spawn(Emoticon::default());
                                            });

                                            if fatality_index < 5 {
                                                entity_commands.insert(TransientAudioPallet::new(
                                                    vec![
                                                        (
                                                            EmotionSounds::Exclaim,
                                                            audio_vector.clone(),
                                                        ),
                                                        (EmotionSounds::Splat, splat_audio.clone()),
                                                        (EmotionSounds::Pop, pop_audio.clone()),
                                                    ],
                                                ));
                                            } else {
                                                entity_commands.insert(TransientAudioPallet::new(
                                                    vec![
                                                        (EmotionSounds::Splat, splat_audio.clone()),
                                                        (EmotionSounds::Pop, pop_audio.clone()),
                                                    ],
                                                ));
                                            }
                                        }
                                    })
                                    .id(),
                            );
                        }
                    })
                    .id();

                track_entities.push(turnout_entity);
            });
        }
    }

    pub fn cleanup(
        mut commands: Commands,
        junction_query: Query<Entity, With<Junction>>,
        body_parts_query: Query<Entity, (With<CharacterSprite>, Without<ChildOf>)>,
        blood_query: Query<Entity, With<BloodSprite>>,
    ) {
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
                if let Some(index) = lever.selected_index() {
                    track.0 = option_color(index);
                }
            }
        }
    }

    pub fn switch_junction(
        time: Res<Time>,
        dilation: Res<Dilation>,
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
            let Some(target_index) = lever.selected_index() else {
                return;
            };
            let Some(target_anchor) = lever_transform.0.get(target_index) else {
                continue;
            };
            let target_position =
                Vec3::new(transform.translation.x, -target_anchor.translation.y, 1.0);

            let distance = (target_position - transform.translation).length();

            if distance > DISTANCE_THRESHOLD {
                let direction = (target_position - transform.translation).normalize();
                let movement_speed =
                    distance * PROPORTIONAL_SPEED_FACTOR * time.delta_secs() * dilation.0;
                transform.translation += direction * movement_speed;
            } else {
                transform.translation = target_position;
            }
        }
    }

    pub fn check_if_person_in_path_of_train(
        mut lever_query: Query<(&Children, &BranchTrack)>,
        mut text_query: Query<(&GlobalTransform, &mut PersonSprite)>,
        train_query: Single<&GlobalTransform, With<Train>>,
        lever: Option<Res<Lever>>,
    ) {
        if let Some(lever) = lever {
            let selected_index = lever.selected_index();
            for (children, track) in lever_query.iter_mut() {
                for child in children.iter() {
                    if let Ok((transform, mut person)) = text_query.get_mut(child) {
                        let train_is_before_person =
                            train_query.translation().x < transform.translation().x;
                        person.in_danger =
                            train_is_before_person && selected_index == Some(track.index);
                    }
                }
            }
        }
    }

    fn sync_mass_panic_audio(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        person_query: Query<&PersonSprite>,
        active_query: Query<Entity, With<CrowdPanicNoise>>,
    ) {
        let in_danger_count = person_query
            .iter()
            .filter(|person| person.in_danger)
            .count();
        let should_play = in_danger_count >= Self::MASS_PANIC_THRESHOLD;

        let mut active_entities = active_query.iter();
        let first_active = active_entities.next();

        if should_play {
            if first_active.is_none() {
                commands.spawn((
                    CrowdPanicNoise,
                    ContinuousAudioPallet::new(vec![ContinuousAudio {
                        key: EmotionSounds::Exclaim,
                        source: AudioPlayer::<AudioSource>(
                            asset_server.load(Self::MASS_PANIC_SOUND),
                        ),
                        settings: PlaybackSettings {
                            volume: Volume::Linear(0.5),
                            ..continuous_audio()
                        },
                        dilatable: true,
                    }]),
                ));
            }
            for extra in active_entities {
                commands.entity(extra).despawn();
            }
            return;
        }

        if let Some(first) = first_active {
            commands.entity(first).despawn();
        }
        for extra in active_entities {
            commands.entity(extra).despawn();
        }
    }

    fn play_mass_rescue_cheer_on_lever_pull(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        person_query: Query<&PersonSprite>,
        lever: Option<Res<Lever>>,
        mut tracker: Local<MassRescueCheerTracker>,
    ) {
        let current_danger_count = person_query
            .iter()
            .filter(|person| person.in_danger)
            .count();
        let Some(lever) = lever else {
            tracker.initialized = false;
            tracker.last_danger_count = current_danger_count;
            return;
        };

        if lever.is_added() {
            tracker.initialized = true;
            tracker.last_danger_count = current_danger_count;
            return;
        }

        let rescued_count = tracker
            .last_danger_count
            .saturating_sub(current_danger_count);

        if tracker.initialized
            && lever.is_changed()
            && rescued_count >= Self::MASS_RESCUE_CHEER_THRESHOLD
        {
            commands.spawn(OneShotAudioPallet::new(vec![OneShotAudio {
                source: asset_server.load(Self::MASS_RESCUE_CHEER_SOUND),
                volume: 0.8,
                dilatable: true,
                ..default()
            }]));
        }

        tracker.initialized = true;
        tracker.last_danger_count = current_danger_count;
    }
}

#[derive(Component)]
pub struct CrowdPanicNoise;

#[derive(Default)]
struct MassRescueCheerTracker {
    initialized: bool,
    last_danger_count: usize,
}
