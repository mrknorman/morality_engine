use bevy::prelude::*;
use serde::{Serialize, Deserialize};


#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MainState {
    #[default]
    Menu,
    InGame,
}

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameState {
    #[default]
    None,
    Loading,
    Dialogue,
    Dilemma,
}

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubState {
    #[default]
    None,
    Intro,
    IntroDecisionTransition,
    Decision,
    ConsequenceAnimation,
    Results,
}


