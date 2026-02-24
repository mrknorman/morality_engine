use crate::{
    data::stats::{DilemmaStats, GameStats},
    scenes::Scene,
};

pub mod engine;
pub mod ids;
pub mod schema;
pub mod validate;

pub fn next_scenes_for_current_dilemma(
    current_scene: Scene,
    latest: &DilemmaStats,
    stats: &GameStats,
) -> Option<Vec<Scene>> {
    match engine::evaluate_next_scenes_from_graph(current_scene, latest, stats) {
        Ok(next_scenes) => next_scenes,
        Err(error) => {
            bevy::log::warn!(
                "graph-driven flow evaluation failed; falling back to menu route: {}",
                error
            );
            Some(vec![Scene::Menu])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenes::{
        dialogue::content::{DialogueScene, Lab3aDialogue, Lab3bDialogue, Lab4Dialogue},
        dilemma::content::{DilemmaScene, Lab0Dilemma, Lab3Dilemma, Lab4Dilemma},
    };

    #[test]
    fn lab_three_junction_without_five_fatalities_routes_to_utilitarian_path() {
        let latest = DilemmaStats {
            num_fatalities: 1,
            ..Default::default()
        };
        let stats = GameStats::default();

        let next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::Lab3(Lab3Dilemma::AsleepAtTheJob)),
            &latest,
            &stats,
        )
        .expect("expected a route");

        assert!(matches!(
            next.as_slice(),
            [
                Scene::Dialogue(DialogueScene::Lab3a(Lab3aDialogue::PassUtilitarian)),
                Scene::Dialogue(DialogueScene::Lab3b(Lab3bDialogue::Intro)),
                Scene::Dilemma(scene)
            ] if *scene == DilemmaScene::PATH_UTILITARIAN[0]
        ));
    }

    #[test]
    fn utilitarian_path_with_no_selection_defaults_to_fail_route() {
        let latest = DilemmaStats::default();
        let stats = GameStats::default();

        let next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::PATH_UTILITARIAN[0]),
            &latest,
            &stats,
        )
        .expect("expected a route");

        assert!(matches!(
            next.as_slice(),
            [
                Scene::Dialogue(DialogueScene::Lab4(Lab4Dialogue::UtilitarianPath(_))),
                Scene::Dialogue(DialogueScene::Lab4(Lab4Dialogue::Outro)),
                Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths))
            ]
        ));
    }

    #[test]
    fn random_deaths_scene_has_no_followup_route() {
        let latest = DilemmaStats::default();
        let stats = GameStats::default();

        assert!(next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths)),
            &latest,
            &stats,
        )
        .is_none());
    }

    #[test]
    fn graph_routes_lab0_to_pass_path_when_no_decision_or_fatality() {
        let latest = DilemmaStats::default();
        let stats = GameStats::default();

        let next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit)),
            &latest,
            &stats,
        )
        .expect("expected a route");

        assert!(matches!(
            next.as_slice(),
            [
                Scene::Dialogue(_),
                Scene::Dialogue(_),
                Scene::Dilemma(_)
            ]
        ));
    }
}
