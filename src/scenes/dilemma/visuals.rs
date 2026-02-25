use std::fmt;

use bevy::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{
    entities::{
        text::TextFrames,
        train::{content::TrainTypes, TrainType},
    },
    systems::backgrounds::content::BackgroundTypes,
};

const VISUAL_PROFILES_JSON: &str = include_str!("./content/visual_profiles.json");
pub const DEFAULT_VISUAL_PROFILE: &str = "desert_default";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DilemmaVisualSelectionLoader {
    #[serde(default = "default_profile_owned")]
    pub profile: String,
    #[serde(default)]
    pub intensity: u8,
}

impl Default for DilemmaVisualSelectionLoader {
    fn default() -> Self {
        Self {
            profile: default_profile_owned(),
            intensity: 0,
        }
    }
}

impl DilemmaVisualSelectionLoader {
    pub fn into_runtime(self) -> DilemmaVisualSelection {
        DilemmaVisualSelection {
            profile: self.profile,
            intensity: self.intensity,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DilemmaVisualSelection {
    pub profile: String,
    pub intensity: u8,
}

impl Default for DilemmaVisualSelection {
    fn default() -> Self {
        Self {
            profile: String::from(DEFAULT_VISUAL_PROFILE),
            intensity: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DilemmaVisualCatalog {
    pub version: u32,
    pub profiles: Vec<DilemmaVisualProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DilemmaVisualProfile {
    pub id: String,
    #[serde(default)]
    pub background_layers: Vec<BackgroundLayerProfile>,
    #[serde(default)]
    pub ambient_smoke: AmbientSmokeProfile,
    #[serde(default)]
    pub ambient_viscera: AmbientVisceraProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackgroundLayerProfile {
    pub background_type: BackgroundTypes,
    pub density_base: f32,
    #[serde(default)]
    pub density_per_intensity: f32,
    #[serde(default)]
    pub speed_base: f32,
    #[serde(default)]
    pub speed_per_stage_speed: f32,
    #[serde(default = "default_alpha_multiplier")]
    pub alpha_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AmbientSmokeProfile {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub base_count: usize,
    #[serde(default)]
    pub count_per_intensity: usize,
    #[serde(default = "default_smoke_frame_seconds")]
    pub frame_seconds: f32,
    #[serde(default = "default_smoke_rise_speed")]
    pub rise_speed: f32,
    #[serde(default = "default_smoke_drift_speed")]
    pub drift_speed: f32,
    #[serde(default = "default_smoke_drift_jitter")]
    pub drift_jitter: f32,
    #[serde(default = "default_smoke_min_x")]
    pub min_x: f32,
    #[serde(default = "default_smoke_max_x")]
    pub max_x: f32,
    #[serde(default = "default_smoke_min_y")]
    pub min_y: f32,
    #[serde(default = "default_smoke_max_y")]
    pub max_y: f32,
}

impl Default for AmbientSmokeProfile {
    fn default() -> Self {
        Self {
            enabled: false,
            base_count: 0,
            count_per_intensity: 0,
            frame_seconds: default_smoke_frame_seconds(),
            rise_speed: default_smoke_rise_speed(),
            drift_speed: default_smoke_drift_speed(),
            drift_jitter: default_smoke_drift_jitter(),
            min_x: default_smoke_min_x(),
            max_x: default_smoke_max_x(),
            min_y: default_smoke_min_y(),
            max_y: default_smoke_max_y(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AmbientVisceraProfile {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub body_parts_base_count: usize,
    #[serde(default)]
    pub body_parts_per_intensity: usize,
    #[serde(default)]
    pub blood_base_count: usize,
    #[serde(default)]
    pub blood_per_intensity: usize,
    #[serde(default = "default_viscera_min_x")]
    pub min_x: f32,
    #[serde(default = "default_viscera_max_x")]
    pub max_x: f32,
    #[serde(default = "default_viscera_min_y")]
    pub min_y: f32,
    #[serde(default = "default_viscera_max_y")]
    pub max_y: f32,
}

impl Default for AmbientVisceraProfile {
    fn default() -> Self {
        Self {
            enabled: false,
            body_parts_base_count: 0,
            body_parts_per_intensity: 0,
            blood_base_count: 0,
            blood_per_intensity: 0,
            min_x: default_viscera_min_x(),
            max_x: default_viscera_max_x(),
            min_y: default_viscera_min_y(),
            max_y: default_viscera_max_y(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedDilemmaVisuals {
    pub background_layers: Vec<ResolvedBackgroundLayer>,
    pub ambient_smoke: Option<ResolvedAmbientSmoke>,
    pub ambient_viscera: Option<ResolvedAmbientViscera>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedBackgroundLayer {
    pub background_type: BackgroundTypes,
    pub density: f32,
    pub speed: f32,
    pub alpha_multiplier: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedAmbientSmoke {
    pub count: usize,
    pub frame_seconds: f32,
    pub rise_speed: f32,
    pub drift_speed: f32,
    pub drift_jitter: f32,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedAmbientViscera {
    pub body_parts_count: usize,
    pub blood_count: usize,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

#[derive(Debug, Clone)]
pub enum VisualCatalogError {
    Parse(String),
}

impl fmt::Display for VisualCatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for VisualCatalogError {}

static VISUAL_CATALOG: Lazy<Result<DilemmaVisualCatalog, VisualCatalogError>> =
    Lazy::new(load_visual_catalog);

#[derive(Component)]
pub struct AmbientSmokePlume;

#[derive(Component)]
pub struct AmbientBackgroundElement;

#[derive(Component)]
pub struct AmbientSmokeAnimation {
    frame_index: usize,
    timer: Timer,
}

impl AmbientSmokeAnimation {
    pub fn new(frame_seconds: f32, frame_index: usize) -> Self {
        Self {
            frame_index,
            timer: Timer::from_seconds(frame_seconds.max(0.01), TimerMode::Repeating),
        }
    }

    pub fn animate(
        time: Res<Time>,
        dilation: Res<crate::systems::time::Dilation>,
        mut query: Query<(&mut AmbientSmokeAnimation, &TextFrames, &mut Text2d)>,
    ) {
        for (mut animation, frames, mut text) in &mut query {
            if frames.frames.is_empty() {
                continue;
            }

            animation.timer.tick(time.delta().mul_f32(dilation.0));
            if animation.timer.just_finished() {
                animation.frame_index = (animation.frame_index + 1) % frames.frames.len();
                text.0 = frames.frames[animation.frame_index].clone();
            }
        }
    }
}

pub fn resolve_visuals(
    selection: &DilemmaVisualSelection,
    stage_speed: f32,
) -> ResolvedDilemmaVisuals {
    let profile = match VISUAL_CATALOG.as_ref() {
        Ok(catalog) => {
            let direct = catalog
                .profiles
                .iter()
                .find(|profile| profile.id == selection.profile);
            if direct.is_some() {
                direct
            } else {
                warn!(
                    "unknown dilemma visuals profile `{}`; falling back to `{}`",
                    selection.profile, DEFAULT_VISUAL_PROFILE
                );
                catalog
                    .profiles
                    .iter()
                    .find(|profile| profile.id == DEFAULT_VISUAL_PROFILE)
            }
        }
        Err(error) => {
            warn!("failed to load dilemma visual profiles: {error}");
            None
        }
    };

    if let Some(profile) = profile {
        return resolve_from_profile(profile, selection.intensity, stage_speed);
    }

    fallback_desert_visuals(stage_speed)
}

pub fn smoke_frames() -> Vec<String> {
    TrainType::load_from_json(TrainTypes::SteamTrain)
        .rising_smoke
        .unwrap_or_else(default_smoke_frames)
}

pub const AMBIENT_BODY_PART_GLYPHS: [&str; 6] = ["@", "/", "\\", "|", "x", "#"];
pub const AMBIENT_BLOOD_GLYPHS: [&str; 4] = [",", ".", "\"", ":"];

fn resolve_from_profile(
    profile: &DilemmaVisualProfile,
    intensity: u8,
    stage_speed: f32,
) -> ResolvedDilemmaVisuals {
    let intensity = intensity as f32;

    let background_layers = profile
        .background_layers
        .iter()
        .map(|layer| ResolvedBackgroundLayer {
            background_type: layer.background_type,
            density: (layer.density_base + layer.density_per_intensity * intensity).max(0.0),
            speed: layer.speed_base + layer.speed_per_stage_speed * stage_speed,
            alpha_multiplier: layer.alpha_multiplier.clamp(0.0, 1.0),
        })
        .collect();

    let ambient_smoke = if profile.ambient_smoke.enabled {
        let count = profile
            .ambient_smoke
            .base_count
            .saturating_add(profile.ambient_smoke.count_per_intensity * (intensity as usize));
        if count == 0 {
            None
        } else {
            Some(ResolvedAmbientSmoke {
                count,
                frame_seconds: profile.ambient_smoke.frame_seconds.max(0.01),
                rise_speed: profile.ambient_smoke.rise_speed,
                drift_speed: profile.ambient_smoke.drift_speed,
                drift_jitter: profile.ambient_smoke.drift_jitter.max(0.0),
                min_x: profile.ambient_smoke.min_x.min(profile.ambient_smoke.max_x),
                max_x: profile.ambient_smoke.max_x.max(profile.ambient_smoke.min_x),
                min_y: profile.ambient_smoke.min_y.min(profile.ambient_smoke.max_y),
                max_y: profile.ambient_smoke.max_y.max(profile.ambient_smoke.min_y),
            })
        }
    } else {
        None
    };

    let ambient_viscera = if profile.ambient_viscera.enabled {
        let body_parts_count = profile
            .ambient_viscera
            .body_parts_base_count
            .saturating_add(
                profile.ambient_viscera.body_parts_per_intensity * (intensity as usize),
            );
        let blood_count = profile
            .ambient_viscera
            .blood_base_count
            .saturating_add(profile.ambient_viscera.blood_per_intensity * (intensity as usize));

        if body_parts_count == 0 && blood_count == 0 {
            None
        } else {
            Some(ResolvedAmbientViscera {
                body_parts_count,
                blood_count,
                min_x: profile
                    .ambient_viscera
                    .min_x
                    .min(profile.ambient_viscera.max_x),
                max_x: profile
                    .ambient_viscera
                    .max_x
                    .max(profile.ambient_viscera.min_x),
                min_y: profile
                    .ambient_viscera
                    .min_y
                    .min(profile.ambient_viscera.max_y),
                max_y: profile
                    .ambient_viscera
                    .max_y
                    .max(profile.ambient_viscera.min_y),
            })
        }
    } else {
        None
    };

    ResolvedDilemmaVisuals {
        background_layers,
        ambient_smoke,
        ambient_viscera,
    }
}

fn fallback_desert_visuals(stage_speed: f32) -> ResolvedDilemmaVisuals {
    ResolvedDilemmaVisuals {
        background_layers: vec![ResolvedBackgroundLayer {
            background_type: BackgroundTypes::Desert,
            density: 0.00002,
            speed: -0.5 * (stage_speed / 70.0),
            alpha_multiplier: 1.0,
        }],
        ambient_smoke: None,
        ambient_viscera: None,
    }
}

fn load_visual_catalog() -> Result<DilemmaVisualCatalog, VisualCatalogError> {
    serde_json::from_str(VISUAL_PROFILES_JSON).map_err(|error| {
        VisualCatalogError::Parse(format!(
            "failed to parse dilemma visual profile catalog: {error}"
        ))
    })
}

fn default_profile_owned() -> String {
    String::from(DEFAULT_VISUAL_PROFILE)
}

fn default_alpha_multiplier() -> f32 {
    1.0
}

fn default_smoke_frame_seconds() -> f32 {
    0.35
}

fn default_smoke_rise_speed() -> f32 {
    7.0
}

fn default_smoke_drift_speed() -> f32 {
    -3.0
}

fn default_smoke_drift_jitter() -> f32 {
    1.5
}

fn default_smoke_min_x() -> f32 {
    -900.0
}

fn default_smoke_max_x() -> f32 {
    900.0
}

fn default_smoke_min_y() -> f32 {
    -240.0
}

fn default_smoke_max_y() -> f32 {
    240.0
}

fn default_viscera_min_x() -> f32 {
    -900.0
}

fn default_viscera_max_x() -> f32 {
    900.0
}

fn default_viscera_min_y() -> f32 {
    -270.0
}

fn default_viscera_max_y() -> f32 {
    120.0
}

fn default_smoke_frames() -> Vec<String> {
    vec![
        String::from(" . "),
        String::from(" .."),
        String::from("..."),
        String::from(" ::"),
        String::from(":::"),
        String::from(" **"),
        String::from("***"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_catalog_parses() {
        let catalog: DilemmaVisualCatalog =
            serde_json::from_str(VISUAL_PROFILES_JSON).expect("visual profiles json should parse");
        assert_eq!(catalog.version, 1);
        assert!(!catalog.profiles.is_empty());
    }

    #[test]
    fn unknown_profile_falls_back_to_default_desert_profile() {
        let selection = DilemmaVisualSelection {
            profile: String::from("unknown"),
            intensity: 3,
        };
        let resolved = resolve_visuals(&selection, 70.0);
        assert!(!resolved.background_layers.is_empty());
        assert_eq!(
            resolved.background_layers[0].background_type,
            BackgroundTypes::Desert
        );
    }

    #[test]
    fn apocalypse_density_scales_with_intensity() {
        let low = resolve_visuals(
            &DilemmaVisualSelection {
                profile: String::from("apocalypse"),
                intensity: 1,
            },
            70.0,
        );
        let high = resolve_visuals(
            &DilemmaVisualSelection {
                profile: String::from("apocalypse"),
                intensity: 4,
            },
            70.0,
        );

        let low_apocalypse = low
            .background_layers
            .iter()
            .find(|layer| layer.background_type == BackgroundTypes::Apocalypse)
            .expect("apocalypse profile should include apocalypse layer");
        let high_apocalypse = high
            .background_layers
            .iter()
            .find(|layer| layer.background_type == BackgroundTypes::Apocalypse)
            .expect("apocalypse profile should include apocalypse layer");

        assert_eq!(low.background_layers.len(), 1);
        assert_eq!(high.background_layers.len(), 1);
        assert_eq!(high_apocalypse.density, low_apocalypse.density);
        assert!(
            high.ambient_smoke
                .as_ref()
                .expect("smoke should be enabled")
                .count
                > low
                    .ambient_smoke
                    .as_ref()
                    .expect("smoke should be enabled")
                    .count
        );
        assert!(
            high.ambient_viscera
                .as_ref()
                .expect("viscera should be enabled")
                .body_parts_count
                > low
                    .ambient_viscera
                    .as_ref()
                    .expect("viscera should be enabled")
                    .body_parts_count
        );
        assert!(
            high.ambient_viscera
                .as_ref()
                .expect("viscera should be enabled")
                .blood_count
                > low
                    .ambient_viscera
                    .as_ref()
                    .expect("viscera should be enabled")
                    .blood_count
        );
    }
}
