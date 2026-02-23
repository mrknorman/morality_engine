use crate::data::states::{DilemmaPhase, GameState, MainState, StateVector};

use super::{Scene, SceneQueue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneNavigationError {
    EmptyQueue,
}

pub struct SceneNavigator;

impl SceneNavigator {
    pub fn fallback_state_vector() -> StateVector {
        StateVector::new(Some(MainState::Menu), None, None)
    }

    pub fn state_vector_for(scene: Scene) -> StateVector {
        match scene {
            Scene::Menu => StateVector::new(Some(MainState::Menu), None, None),
            Scene::Loading => {
                StateVector::new(Some(MainState::InGame), Some(GameState::Loading), None)
            }
            Scene::Dialogue(_) => {
                StateVector::new(Some(MainState::InGame), Some(GameState::Dialogue), None)
            }
            Scene::Dilemma(_) => StateVector::new(
                Some(MainState::InGame),
                Some(GameState::Dilemma),
                Some(DilemmaPhase::Intro),
            ),
            Scene::Ending(_) => {
                StateVector::new(Some(MainState::InGame), Some(GameState::Ending), None)
            }
        }
    }

    pub fn advance(queue: &mut SceneQueue) -> Result<(Scene, StateVector), SceneNavigationError> {
        let scene = queue.try_pop().ok_or(SceneNavigationError::EmptyQueue)?;
        Ok((scene, Self::state_vector_for(scene)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenes::{
        dialogue::content::{DialogueScene, Lab0Dialogue},
        dilemma::content::{DilemmaScene, Lab0Dilemma},
        ending::content::EndingScene,
    };

    #[test]
    fn state_vector_for_dilemma_sets_intro_substate() {
        let scene = Scene::Dilemma(DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit));

        assert_eq!(
            SceneNavigator::state_vector_for(scene),
            StateVector::new(
                Some(MainState::InGame),
                Some(GameState::Dilemma),
                Some(DilemmaPhase::Intro),
            )
        );
    }

    #[test]
    fn state_vector_for_menu_sets_only_main_state() {
        assert_eq!(
            SceneNavigator::state_vector_for(Scene::Menu),
            StateVector::new(Some(MainState::Menu), None, None)
        );
    }

    #[test]
    fn fallback_state_vector_routes_to_menu() {
        assert_eq!(
            SceneNavigator::fallback_state_vector(),
            StateVector::new(Some(MainState::Menu), None, None)
        );
    }

    #[test]
    fn advance_moves_queue_and_returns_route() {
        let mut queue = SceneQueue::default();

        let (scene, state_vector) = SceneNavigator::advance(&mut queue).expect("queue should advance");

        assert!(matches!(scene, Scene::Loading));
        assert_eq!(
            state_vector,
            StateVector::new(Some(MainState::InGame), Some(GameState::Loading), None)
        );
        assert!(matches!(queue.current, Scene::Loading));
        assert!(matches!(
            queue.next,
            Some(Scene::Dialogue(DialogueScene::Lab0(Lab0Dialogue::Intro)))
        ));
    }

    #[test]
    fn advance_on_empty_queue_returns_error() {
        let mut queue = SceneQueue::default();
        queue.replace(vec![]);

        assert!(matches!(
            SceneNavigator::advance(&mut queue),
            Err(SceneNavigationError::EmptyQueue)
        ));
    }

    #[test]
    fn state_vector_for_ending_routes_to_ending_game_state() {
        let scene = Scene::Ending(EndingScene::TrueNeutral);

        assert_eq!(
            SceneNavigator::state_vector_for(scene),
            StateVector::new(Some(MainState::InGame), Some(GameState::Ending), None)
        );
    }
}
