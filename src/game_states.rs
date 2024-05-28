use bevy::prelude::*;
use serde::{Serialize, Deserialize};


#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MainState {
    Menu,
    InGame,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameState {
    None,
    Loading,
    Dialogue,
    Dilemma,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubState {
    None,
    Intro,
    Decision,
    Results,
}


