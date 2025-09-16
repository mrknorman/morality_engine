pub mod dialogue;
pub mod dilemma;
pub mod loading;
pub mod ending;
pub mod menu;

use std::collections::VecDeque;

use bevy::prelude::*;

use menu::MenuScenePlugin;
use loading::LoadingScenePlugin;
use dialogue::{
    DialogueScenePlugin,
    content::*
};
use dilemma::{
    DilemmaScenePlugin,
    content::*
};
use ending::{
    EndingScenePlugin,
    content::*
};


pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app         
        .add_plugins(
            (
                MenuScenePlugin,
                LoadingScenePlugin,
                DialogueScenePlugin,
                DilemmaScenePlugin,
                EndingScenePlugin
            )
        )  
        .insert_resource(SceneQueue::dilemma_start());
    }
}

#[derive(Resource)]
pub struct SceneQueue{
    queue : VecDeque<Scene>,
    next: Option<Scene>,
    current : Scene
}

impl SceneQueue {
    pub fn pop(&mut self) -> Scene {
        let scene = match self.queue.pop_front() {
            Some(scene) => scene,
            _ => panic!("Queue Is Empty!")
        };
        self.next = self.queue.front().copied();
        self.current = scene;
        scene
    }

    pub fn replace(&mut self, new_queue: Vec<Scene>) {
        self.queue = VecDeque::from(new_queue);
        self.next = self.queue.front().copied();
    }
}


impl Default for SceneQueue{
    fn default() -> Self {
        Self {
            queue : VecDeque::from([
                Scene::Loading, 
                Scene::Dialogue(DialogueScene::Lab0(Lab0Dialogue::Intro)),   
                Scene::Dilemma(DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit))
            ]),
            next : Some(Scene::Loading),
            current : Scene::Menu
        }
    }
}

impl SceneQueue {
    fn dilemma_start() -> Self {
        Self {
            queue : VecDeque::from([
                Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem))
            ]),
            next : None,
            current : Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem))
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
#[require(Transform, Visibility)]
pub enum Scene {
    Menu,
    Loading,
    Dialogue(DialogueScene),
    Dilemma(DilemmaScene),
    Ending(EndingScene)
}


