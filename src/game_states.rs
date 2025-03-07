use bevy::prelude::*;
use serde::{Serialize, Deserialize};

use crate::{dialogue::content::*, dilemma::content::*};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MainState {   
    #[default]
    Menu,
    InGame
}

#[derive(Default, SubStates, Debug, Clone, PartialEq, Eq, Hash, Serialize, 
    Deserialize)]
#[source(MainState = MainState::InGame)]
pub enum GameState {
    #[default]
    Loading,
    Dialogue,
    Dilemma,
}

#[derive(Default, SubStates, Debug, Clone, PartialEq, Eq, Hash)]
#[source(GameState = GameState::Dilemma)]
pub enum DilemmaPhase {
    #[default]
    Intro,
    IntroDecisionTransition,
    Decision,
    Consequence,
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


#[derive(Resource)]
pub struct Memory{
    pub next_dialogue : Vec<DialogueContent>,
    pub next_dilemma : Option<DilemmaContent>
}

impl Default for Memory{
    fn default() -> Self {
        Self{
            next_dialogue : vec![DialogueContent::Lab0(Lab0Dialogue::Intro)],
            next_dilemma : Some(DilemmaContent::Lab0(Lab0Dilemma::IncompetentBandit))
        }
    }

}