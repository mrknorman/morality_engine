pub mod composition;
pub mod dialogue;
pub mod dilemma;
pub mod ending;
pub mod flow;
pub mod loading;
pub mod menu;
pub mod runtime;

use std::collections::VecDeque;

use bevy::prelude::*;

use composition::SceneCompositionPlugin;
use dialogue::{content::*, DialogueScenePlugin};
use dilemma::{content::*, DilemmaScenePlugin};
use ending::{content::*, EndingScenePlugin};
use loading::LoadingScenePlugin;
use menu::MenuScenePlugin;

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SceneCompositionPlugin,
            MenuScenePlugin,
            LoadingScenePlugin,
            DialogueScenePlugin,
            DilemmaScenePlugin,
            EndingScenePlugin,
        ))
        .insert_resource(SceneQueue::default());
    }
}

#[derive(Resource)]
pub struct SceneQueue {
    queue: VecDeque<Scene>,
    next: Option<Scene>,
    current: Scene,
    flow_mode: SceneFlowMode,
}

impl SceneQueue {
    pub fn current_scene(&self) -> Scene {
        self.current
    }

    pub fn next_scene(&self) -> Option<Scene> {
        self.next
    }

    pub fn reset_campaign(&mut self) {
        *self = Self::default();
    }

    pub fn try_pop(&mut self) -> Option<Scene> {
        let scene = self.queue.pop_front()?;
        self.next = self.queue.front().copied();
        self.current = scene;
        Some(scene)
    }

    pub fn pop(&mut self) -> Option<Scene> {
        self.try_pop()
    }

    pub fn replace(&mut self, new_queue: Vec<Scene>) {
        self.queue = VecDeque::from(new_queue);
        self.next = self.queue.front().copied();
        self.flow_mode = SceneFlowMode::Campaign;
    }

    pub fn configure_single_scene(&mut self, scene: Scene) {
        self.queue = VecDeque::from([scene, Scene::Menu]);
        self.next = self.queue.front().copied();
        self.current = Scene::Menu;
        self.flow_mode = SceneFlowMode::SingleLevel;
    }

    pub fn configure_campaign_from_dilemma(&mut self, scene: DilemmaScene) {
        self.queue = VecDeque::from([Scene::Dilemma(scene)]);
        self.next = self.queue.front().copied();
        self.current = Scene::Menu;
        self.flow_mode = SceneFlowMode::Campaign;
    }

    pub fn configure_single_level(&mut self, scene: DilemmaScene) {
        self.configure_single_scene(Scene::Dilemma(scene));
    }

    pub fn flow_mode(&self) -> SceneFlowMode {
        self.flow_mode
    }
}

impl Default for SceneQueue {
    fn default() -> Self {
        Self {
            queue: VecDeque::from([
                Scene::Loading,
                Scene::Dialogue(DialogueScene::Lab0(Lab0Dialogue::Intro)),
                Scene::Dilemma(DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit)),
            ]),
            next: Some(Scene::Loading),
            current: Scene::Menu,
            flow_mode: SceneFlowMode::Campaign,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneFlowMode {
    Campaign,
    SingleLevel,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
#[require(Transform, Visibility)]
pub enum Scene {
    Menu,
    Loading,
    Dialogue(DialogueScene),
    Dilemma(DilemmaScene),
    Ending(EndingScene),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configure_single_scene_sets_expected_queue_and_mode() {
        let mut queue = SceneQueue::default();
        let scene = Scene::Dialogue(DialogueScene::Lab0(Lab0Dialogue::Intro));

        queue.configure_single_scene(scene);

        assert_eq!(queue.flow_mode(), SceneFlowMode::SingleLevel);
        assert!(matches!(
            queue.pop(),
            Some(Scene::Dialogue(DialogueScene::Lab0(Lab0Dialogue::Intro)))
        ));
        assert!(matches!(queue.pop(), Some(Scene::Menu)));
    }

    #[test]
    fn configure_single_level_sets_expected_queue_and_mode() {
        let mut queue = SceneQueue::default();
        let scene = DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit);

        queue.configure_single_level(scene);

        assert_eq!(queue.flow_mode(), SceneFlowMode::SingleLevel);
        assert!(matches!(queue.pop(), Some(Scene::Dilemma(found)) if found == scene));
        assert!(matches!(queue.pop(), Some(Scene::Menu)));
    }

    #[test]
    fn configure_campaign_from_dilemma_sets_campaign_mode_and_queue() {
        let mut queue = SceneQueue::default();
        let scene = DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem);

        queue.configure_campaign_from_dilemma(scene);

        assert_eq!(queue.flow_mode(), SceneFlowMode::Campaign);
        assert!(matches!(queue.pop(), Some(Scene::Dilemma(found)) if found == scene));
        assert!(queue.pop().is_none());
    }

    #[test]
    fn replace_resets_flow_mode_to_campaign() {
        let mut queue = SceneQueue::default();
        queue.configure_single_level(DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit));

        queue.replace(vec![Scene::Menu]);

        assert_eq!(queue.flow_mode(), SceneFlowMode::Campaign);
        assert!(matches!(queue.pop(), Some(Scene::Menu)));
    }

    #[test]
    fn try_pop_on_empty_queue_returns_none() {
        let mut queue = SceneQueue::default();
        queue.replace(vec![]);

        assert!(queue.try_pop().is_none());
    }
}
