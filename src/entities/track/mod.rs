use crate::entities::text::TextSprite;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrackSkin {
    #[default]
    Rail,
    Road,
}

impl TrackSkin {
    const fn segment(self) -> &'static str {
        match self {
            TrackSkin::Rail => "~",
            TrackSkin::Road => "=",
        }
    }
}

#[derive(Component)]
#[require(TextSprite, Text2d)]
pub struct Track;

impl Track {
    pub const LENGTH: usize = 1;

    pub fn generate_track(length: usize, skin: TrackSkin) -> String {
        skin.segment().repeat(length)
    }

    pub fn new(length: usize) -> (Track, Text2d) {
        Self::new_with_skin(length, TrackSkin::Rail)
    }

    pub fn new_with_skin(length: usize, skin: TrackSkin) -> (Track, Text2d) {
        (Track, Text2d::new(Self::generate_track(length, skin)))
    }
}
