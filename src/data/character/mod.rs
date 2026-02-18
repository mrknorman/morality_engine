use bevy::prelude::*;
use enum_map::Enum;
use std::path::PathBuf;

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterKey {
    Creator,
}

#[derive(Component, Clone)]
pub struct Character {
    pub name: String,
    pub audio_source_file_path: PathBuf,
    pub color: Color,
    pub key: CharacterKey,
}

impl Character {
    pub fn new(
        name: &str,
        audio_source_file_path: &str,
        color: Color,
        key: CharacterKey,
    ) -> Character {
        Character {
            name: String::from(name),
            audio_source_file_path: PathBuf::from(audio_source_file_path),
            color,
            key,
        }
    }
}
