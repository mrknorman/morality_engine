use std::f32::consts::FRAC_PI_4;

use bevy::{
    audio::Volume,
    camera::primitives::Aabb,
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use enum_map::{enum_map, Enum};
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use serde::Deserialize;

pub mod content;
use content::TrainTypes;

use crate::{
    data::rng::GlobalRng,
    entities::text::{scaled_text_units, Animated, GlyphString, TextFrames, TextSprite},
    systems::{
        audio::{
            continuous_audio, AudioPlugin, ContinuousAudio, ContinuousAudioPallet, OneShotAudio,
            OneShotAudioPallet, TransientAudio, TransientAudioPallet,
        },
        colors::ColorAnchor,
        interaction::{ActionPallet, Clickable, InputAction, InteractionPlugin},
        motion::Wobble,
        physics::Velocity,
    },
};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TrainSystemsActive {
    #[default]
    False,
    True,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainActions {
    TriggerHorn,
}

impl std::fmt::Display for TrainActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct TrainPlugin;
impl Plugin for TrainPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<InteractionPlugin>() {
            app.add_plugins(InteractionPlugin);
        };

        if !app.is_plugin_added::<AudioPlugin>() {
            app.add_plugins(AudioPlugin);
        };

        app.init_state::<TrainSystemsActive>()
            .add_systems(Update, activate_systems)
            .add_systems(
                Update,
                (Animated::animate_text).run_if(in_state(TrainSystemsActive::True)),
            );
    }
}

fn activate_systems(
    mut train_state: ResMut<NextState<TrainSystemsActive>>,
    train_query: Query<&Train>,
) {
    if !train_query.is_empty() {
        train_state.set(TrainSystemsActive::True)
    } else {
        train_state.set(TrainSystemsActive::False)
    }
}

#[derive(Deserialize, Clone)]
pub struct TrainType {
    pub carriages: Vec<String>,
    #[serde(default)]
    pub carriage_step_x: Option<f32>,
    pub smoke: Option<Vec<String>>,
    pub track_audio_path: String,
    pub stopped_audio_path: String,
    pub horn_audio_path: Option<String>,
    pub rising_smoke: Option<Vec<String>>,
}

impl TrainType {
    fn fallback() -> Self {
        Self {
            carriages: vec![String::from("____"), String::from("____")],
            carriage_step_x: None,
            smoke: Some(vec![String::from("~~")]),
            track_audio_path: String::from("./audio/effects/static.ogg"),
            stopped_audio_path: String::from("./audio/effects/static.ogg"),
            horn_audio_path: None,
            rising_smoke: Some(vec![
                String::from(" . "),
                String::from(" .."),
                String::from("..."),
                String::from(" ::"),
                String::from(":::"),
                String::from(" **"),
                String::from("***"),
            ]),
        }
    }

    fn normalized(mut self) -> Self {
        if self.carriages.is_empty() {
            warn!("train content has no carriages; using fallback carriage");
            self.carriages = vec![String::from("____")];
        }

        if self.smoke.is_none() {
            warn!("train content has no smoke frames; using fallback smoke frame");
            self.smoke = Some(vec![String::from("~~")]);
        }

        if self
            .rising_smoke
            .as_ref()
            .is_none_or(|frames| frames.len() < 7)
        {
            warn!("train content has insufficient rising smoke frames; using fallback set");
            self.rising_smoke = Some(vec![
                String::from(" . "),
                String::from(" .."),
                String::from("..."),
                String::from(" ::"),
                String::from(":::"),
                String::from(" **"),
                String::from("***"),
            ]);
        }

        if self.carriage_step_x.is_some_and(|step| step <= 0.0) {
            warn!("train content has non-positive carriage_step_x; using default spacing");
            self.carriage_step_x = None;
        }

        self
    }

    pub fn load_from_json(train_type: TrainTypes) -> TrainType {
        let train_type: TrainType = match serde_json::from_str(train_type.content()) {
            Ok(train_type) => train_type,
            Err(error) => {
                warn!("failed to parse train content: {error}; using fallback train content");
                Self::fallback()
            }
        };

        if train_type.carriages.is_empty() {
            warn!("train content has no carriages; using fallback train content");
            return Self::fallback();
        }

        train_type.normalized()
    }
}

#[derive(Component)]
#[require(TextSprite)]
pub struct TrainCarriage;

#[derive(Component)]
#[require(Animated)]
pub struct TrainSmoke;

#[derive(Clone, Component)]
#[require(Transform, Visibility, Velocity, TrainState, TextSprite)]
#[component(on_insert = Train::on_insert)]
pub struct Train(pub TrainTypes);

#[derive(Component, Clone, Copy, Default, PartialEq, Eq)]
pub enum TrainState {
    #[default]
    Moving,
    Wrecked,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainSounds {
    Tracks,
    Horn,
    Bounce,
}

impl Train {
    const BASE_TRAIN_HEIGHT: f32 = 32.0; // 3 glyph lines
    const BASE_TRAIN_WIDTH: f32 = 30.0; // side-to-side depth
    const BASE_CARRIAGE_STEP_X: f32 = 85.0;
    const BASE_CARRIAGE_Y: f32 = -46.0;
    const BASE_TRACK_ALIGNMENT_OFFSET_Y: f32 = -30.0;
    const BASE_SMOKE_X: f32 = -35.0;
    const BASE_SMOKE_Y: f32 = 32.0;
    const BASE_BURNING_SMOKE_Y: f32 = 100.0;

    pub const fn track_alignment_offset_y() -> f32 {
        scaled_text_units(Self::BASE_TRACK_ALIGNMENT_OFFSET_Y)
    }

    const fn carriage_baseline_y() -> f32 {
        scaled_text_units(Self::BASE_CARRIAGE_Y)
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        // Get GlobalRng first, before other borrows
        let mut rng_option = None;
        if let Some(rng) = world.get_resource_mut::<GlobalRng>() {
            rng_option = Some(rng.uniform.clone()); // Clone the Rng so we can use it without holding the borrow
        }

        // Now collect other data
        let (
            carriages,
            smoke_frames,
            burning_frames,
            horn_audio,
            track_audio,
            hiss_audio,
            burning_audio,
            color,
            train_state,
            bounce_audio,
            carriage_step_x,
        ) = {
            if let (Some(train), Some(color), Some(train_state)) = (
                world.entity(entity).get::<Train>(),
                world.entity(entity).get::<TextColor>(),
                world.entity(entity).get::<TrainState>(),
            ) {
                let train_type = TrainType::load_from_json(train.0);

                let carriages: Vec<String> = train_type.carriages.clone();
                let smoke_frames: Vec<String> = train_type.clone().smoke.unwrap_or(vec![]);
                let burning_frames: Vec<String> = train_type.clone().rising_smoke.unwrap_or(vec![]);
                let Some(asset_server) = world.get_resource::<AssetServer>() else {
                    warn!("AssetServer missing during train setup; skipping train spawn");
                    return;
                };
                let horn_audio: Option<TransientAudio> =
                    train_type.clone().horn_audio_path.map(|path| {
                        TransientAudio::new(asset_server.load(path), 2.0, false, 1.0, true)
                    });
                let track_audio: Handle<AudioSource> =
                    asset_server.load(train_type.track_audio_path);

                const VOLUME: f32 = 2.0;
                let bounce_audio = vec![
                    TransientAudio::new(
                        asset_server.load("./audio/effects/meat_bounce/meat_bounce_1.ogg"),
                        0.2,
                        true,
                        VOLUME,
                        true,
                    ),
                    TransientAudio::new(
                        asset_server.load("./audio/effects/meat_bounce/meat_bounce_2.ogg"),
                        0.2,
                        true,
                        VOLUME,
                        true,
                    ),
                    TransientAudio::new(
                        asset_server.load("./audio/effects/meat_bounce/meat_bounce_3.ogg"),
                        0.2,
                        true,
                        VOLUME,
                        true,
                    ),
                    TransientAudio::new(
                        asset_server.load("./audio/effects/meat_bounce/meat_bounce_4.ogg"),
                        0.2,
                        true,
                        VOLUME,
                        true,
                    ),
                ];

                let hiss_audio: Handle<AudioSource> =
                    asset_server.load(train_type.stopped_audio_path);
                let burning_audio = asset_server.load("./audio/effects/train/fire.ogg");

                (
                    carriages,
                    smoke_frames,
                    burning_frames,
                    horn_audio,
                    track_audio,
                    hiss_audio,
                    burning_audio,
                    *color,
                    *train_state,
                    bounce_audio,
                    train_type
                        .carriage_step_x
                        .unwrap_or(Self::BASE_CARRIAGE_STEP_X),
                )
            } else {
                warn!("train entity missing required components; skipping train setup");
                return;
            }
        };

        let train_height = scaled_text_units(Self::BASE_TRAIN_HEIGHT);
        let train_width = scaled_text_units(Self::BASE_TRAIN_WIDTH);

        let aabb_headroom_x = carriage_step_x + 25.0;
        let aabb_center_step_x = carriage_step_x * 0.5;
        let len_x = scaled_text_units(carriage_step_x) * (carriages.len() as f32 - 1.0)
            + scaled_text_units(aabb_headroom_x); // whole train
        let centre = Vec3::new(
            -(carriages.len() as f32 - 1.0) * scaled_text_units(aabb_center_step_x),
            0.0,
            0.0,
        );
        let half = Vec3::new(len_x * 0.5, train_height * 0.5, train_width * 0.5);

        world.commands().entity(entity).insert(Aabb {
            center: centre.into(),
            half_extents: half.into(),
        });

        // Continue only if we got the RNG
        let mut rng = rng_option.unwrap_or_else(|| {
            warn!("GlobalRng missing during train setup; using deterministic fallback rng");
            Pcg64Mcg::seed_from_u64(12345)
        });

        // Now get the commands after all data is collected
        let mut commands = world.commands();

        // Use the commands to modify entities
        let mut carriage_entities: Vec<Entity> = vec![];

        commands
            .entity(entity)
            .insert(TransientAudioPallet::new(vec![(
                TrainSounds::Bounce,
                bounce_audio.clone(),
            )]));

        if train_state == TrainState::Moving {
            commands
                .entity(entity)
                .insert(ContinuousAudioPallet::new(vec![ContinuousAudio {
                    key: TrainSounds::Tracks,
                    source: AudioPlayer::<AudioSource>(track_audio),
                    settings: PlaybackSettings {
                        volume: Volume::Linear(0.1),
                        ..continuous_audio()
                    },
                    dilatable: true,
                }]));
        } else if train_state == TrainState::Wrecked {
            commands.entity(entity).insert((
                OneShotAudioPallet::new(vec![OneShotAudio {
                    source: hiss_audio,
                    volume: 0.2,
                    dilatable: true,
                    ..default()
                }]),
                ContinuousAudioPallet::new(vec![ContinuousAudio {
                    key: TrainSounds::Tracks,
                    source: AudioPlayer::<AudioSource>(burning_audio),
                    settings: PlaybackSettings {
                        volume: Volume::Linear(0.4),
                        ..continuous_audio()
                    },
                    dilatable: true,
                }]),
            ));
        }

        let mut carriage_translation = Vec3::new(0.0, Self::carriage_baseline_y(), 0.0);
        commands.entity(entity).with_children(|parent| {
            for carriage in carriages.clone() {
                let mut transform = Transform::from_translation(carriage_translation);
                if train_state == TrainState::Wrecked {
                    // Add random rotation between 0 and 20 degrees for wrecked carriages (except first)
                    let random_rotation = rng.random_range(-FRAC_PI_4..FRAC_PI_4);
                    transform.rotate(Quat::from_rotation_z(random_rotation));
                }

                let mut entity = parent.spawn((
                    TrainCarriage,
                    TextSprite,
                    GlyphString {
                        text: carriage,
                        depth: train_width,
                    },
                    transform,
                    color,
                ));

                carriage_entities.push(entity.id());

                if train_state == TrainState::Moving {
                    entity.insert(Wobble::default());
                }

                // Update translation for next carriage
                if train_state == TrainState::Wrecked {
                    // Vary the distance for wrecked trains (except first position)
                    let min_step = (carriage_step_x - 15.0).max(20.0);
                    let max_step = carriage_step_x + 15.0;
                    carriage_translation.x -=
                        rng.random_range(scaled_text_units(min_step)..scaled_text_units(max_step));
                } else {
                    carriage_translation.x -= scaled_text_units(carriage_step_x);
                }
            }

            if train_state != TrainState::Wrecked {
                // Explicit empty smoke list in content disables smoke for this train variant.
                if !smoke_frames.is_empty() {
                    parent.spawn((
                        TrainSmoke,
                        Wobble::default(),
                        TextSprite,
                        Text2d(smoke_frames[0].clone()),
                        TextFrames::new(smoke_frames),
                        Transform::from_xyz(
                            scaled_text_units(Self::BASE_SMOKE_X),
                            scaled_text_units(Self::BASE_SMOKE_Y),
                            0.1,
                        ),
                        color,
                        ColorAnchor(color.0),
                    ));
                }
            } else if train_state == TrainState::Wrecked {
                // Clone once for creating the TextFrames component.
                let text_frames = TextFrames::new(burning_frames.clone());

                // Define the X positions and the corresponding indices in burning_frames.
                let spawn_data = [
                    (scaled_text_units(Self::BASE_SMOKE_X), 0),
                    (0.0, 3),
                    (scaled_text_units(-Self::BASE_SMOKE_X), 6),
                ];

                for (x, idx) in spawn_data {
                    // Clone only the specific frame for this spawn.
                    let frame = burning_frames[idx].clone();
                    parent.spawn((
                        Animated::new(idx, 0.3),
                        Text2d(frame),
                        // Clone the text_frames for each spawn if needed.
                        text_frames.clone(),
                        Transform::from_xyz(x, scaled_text_units(Self::BASE_BURNING_SMOKE_Y), 0.1),
                        color,
                        ColorAnchor(color.0),
                    ));
                }
            }
        });

        if train_state != TrainState::Wrecked {
            if let Some(horn_audio) = horn_audio {
                let mut engine = commands.entity(carriage_entities[0]);

                engine.insert((
                    Clickable::new(vec![TrainActions::TriggerHorn]),
                    ActionPallet::<TrainActions, TrainSounds>(enum_map!(
                        TrainActions::TriggerHorn => vec![
                            InputAction::PlaySound(
                                TrainSounds::Horn
                            )
                        ]
                    )),
                    TransientAudioPallet::new(vec![(TrainSounds::Horn, vec![horn_audio])]),
                ));
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_empty_smoke_list_is_preserved() {
        let truck = TrainType::load_from_json(TrainTypes::PsychopathTruck);
        assert!(truck.smoke.as_ref().is_some_and(|frames| frames.is_empty()));
    }

    #[test]
    fn non_positive_carriage_step_falls_back_to_default_spacing() {
        let parsed: TrainType = serde_json::from_str(
            r#"{
                "carriages": ["cab"],
                "carriage_step_x": 0.0,
                "smoke": [],
                "track_audio_path": "./audio/effects/static.ogg",
                "stopped_audio_path": "./audio/effects/static.ogg",
                "rising_smoke": ["1","2","3","4","5","6","7"]
            }"#,
        )
        .expect("inline train json should parse");

        let normalized = parsed.normalized();
        assert!(
            normalized.carriage_step_x.is_none(),
            "non-positive carriage step should reset to default spacing"
        );
    }
}
