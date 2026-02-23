pub mod dialogue;
pub mod dilemma;
pub mod ending;
pub mod loading;
pub mod menu;

use std::collections::VecDeque;

use bevy::prelude::*;

use dialogue::{content::*, DialogueScenePlugin};
use dilemma::{content::*, DilemmaScenePlugin};
use ending::{content::*, EndingScenePlugin};
use loading::LoadingScenePlugin;
use menu::MenuScenePlugin;

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
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
    pub fn pop(&mut self) -> Scene {
        let scene = match self.queue.pop_front() {
            Some(scene) => scene,
            _ => panic!("Queue Is Empty!"),
        };
        self.next = self.queue.front().copied();
        self.current = scene;
        scene
    }

    pub fn replace(&mut self, new_queue: Vec<Scene>) {
        self.queue = VecDeque::from(new_queue);
        self.next = self.queue.front().copied();
        self.flow_mode = SceneFlowMode::Campaign;
    }

    pub fn configure_single_level(&mut self, scene: DilemmaScene) {
        self.queue = VecDeque::from([Scene::Dilemma(scene), Scene::Menu]);
        self.next = self.queue.front().copied();
        self.current = Scene::Menu;
        self.flow_mode = SceneFlowMode::SingleLevel;
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

impl SceneQueue {
    fn dilemma_start() -> Self {
        Self {
            queue: VecDeque::from([Scene::Dilemma(DilemmaScene::Lab4(
                Lab4Dilemma::RandomDeaths,
            ))]),
            next: None,
            current: Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths)),
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
    fn configure_single_level_sets_expected_queue_and_mode() {
        let mut queue = SceneQueue::default();
        let scene = DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit);

        queue.configure_single_level(scene);

        assert_eq!(queue.flow_mode(), SceneFlowMode::SingleLevel);
        assert!(matches!(queue.pop(), Scene::Dilemma(found) if found == scene));
        assert!(matches!(queue.pop(), Scene::Menu));
    }

    #[test]
    fn replace_resets_flow_mode_to_campaign() {
        let mut queue = SceneQueue::default();
        queue.configure_single_level(DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit));

        queue.replace(vec![Scene::Menu]);

        assert_eq!(queue.flow_mode(), SceneFlowMode::Campaign);
        assert!(matches!(queue.pop(), Scene::Menu));
    }
}
