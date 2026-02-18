use crate::entities::text::TextSprite;
use bevy::prelude::*;

#[derive(Component)]
#[require(TextSprite, Text2d)]
pub struct Track;

impl Track {
    const TRACK_STRING: &str = "~";
    pub const LENGTH: usize = Self::TRACK_STRING.len();

    pub fn generate_track(length: usize) -> String {
        Self::TRACK_STRING.to_string().repeat(length)
    }

    pub fn new(length: usize) -> (Track, Text2d) {
        (Track, Text2d::new(Self::generate_track(length)))
    }
}
