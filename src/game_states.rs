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

#[derive(Clone)]
pub struct StateVector {
    main: Option<MainState>,
    game: Option<GameState>,
    sub: Option<SubState>,
}

impl StateVector {

    pub fn new(
        main: Option<MainState>,
        game: Option<GameState>,
        sub: Option<SubState>,
    ) -> StateVector {
        StateVector {
            main,
            game,
            sub,
        }
    }

    pub fn set_state(
        self,
        next_main_state: &mut ResMut<NextState<MainState>>,
        next_game_state: &mut ResMut<NextState<GameState>>,
        next_sub_state: &mut ResMut<NextState<SubState>>,
    ) {
        if let Some(state) = &self.main {
            next_main_state.set(state.clone());
        }
    
        if let Some(state) = &self.game {
            next_game_state.set(state.clone());
        }
    
        if let Some(state) = &self.sub {
            next_sub_state.set(state.clone());
        }
    }
}