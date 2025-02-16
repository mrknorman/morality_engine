use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MainState {
    Menu,
    #[default]
    InGame
}

#[derive(Default, SubStates, Debug, Clone, PartialEq, Eq, Hash, Serialize, 
    Deserialize)]
#[source(MainState = MainState::InGame)]
pub enum GameState {
    Loading,
    Dialogue,
    #[default]
    Dilemma,
}

#[derive(Default, SubStates, Debug, Clone, PartialEq, Eq, Hash)]
#[source(GameState = GameState::Dilemma)]
pub enum DilemmaPhase {
    #[default]
    Intro,
    IntroDecisionTransition,
    Decision,
    ConsequenceAnimation,
    Results
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateVector {
    main: Option<MainState>,
    game: Option<GameState>,
    sub: Option<DilemmaPhase>,
}

impl StateVector {
    pub fn new(
        main: Option<MainState>,
        game: Option<GameState>,
        sub: Option<DilemmaPhase>,
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
        next_sub_state: &mut ResMut<NextState<DilemmaPhase>>,
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