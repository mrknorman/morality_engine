
use std::path::PathBuf;
use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct Character {
    pub name : String,
    pub audio_source_file_path : PathBuf
}

impl Character {
    pub fn new(name : &str, audio_source_file_path : &str) -> Character {
    
        Character {
            name: String::from(name),
            audio_source_file_path: PathBuf::from(audio_source_file_path)
        }
    }
}