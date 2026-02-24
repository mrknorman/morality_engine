use once_cell::sync::Lazy;

use crate::{
    data::stats::{DilemmaStats, GameStats},
    scenes::{
        dialogue::content::{
            DialogueScene, Lab1aDialogue, Lab1bDialogue, Lab2aDialogue, Lab2bDialogue,
            Lab3aDialogue, Lab3bDialogue, Lab4Dialogue, PathOutcome,
        },
        dilemma::content::{DilemmaScene, Lab0Dilemma, Lab1Dilemma, Lab2Dilemma, Lab3Dilemma, Lab4Dilemma},
        ending::content::EndingScene,
        Scene,
    },
};

use super::{
    ids::{DialogueSceneId, DilemmaSceneId, EndingSceneId, PathOutcomeId, TypedSceneRef},
    schema::{FlowEvalContext, SceneProgressionGraph},
    validate::{validate_graph, GraphValidationError},
};

const CAMPAIGN_GRAPH_JSON: &str = include_str!("./content/campaign_graph.json");

static CAMPAIGN_GRAPH: Lazy<Result<SceneProgressionGraph, FlowEvalError>> =
    Lazy::new(load_campaign_graph);

#[derive(Debug, Clone)]
pub enum FlowEvalError {
    GraphParse(String),
    GraphValidation(Vec<GraphValidationError>),
    RouteMapping(String),
}

impl std::fmt::Display for FlowEvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GraphParse(message) => write!(f, "{message}"),
            Self::GraphValidation(errors) => {
                write!(f, "campaign graph failed validation with {} error(s)", errors.len())
            }
            Self::RouteMapping(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for FlowEvalError {}

pub fn evaluate_next_scenes_from_graph(
    current_scene: Scene,
    latest: &DilemmaStats,
    stats: &GameStats,
) -> Result<Option<Vec<Scene>>, FlowEvalError> {
    let graph = CAMPAIGN_GRAPH.as_ref().map_err(Clone::clone)?;
    evaluate_with_graph(graph, current_scene, latest, stats)
}

fn load_campaign_graph() -> Result<SceneProgressionGraph, FlowEvalError> {
    let graph: SceneProgressionGraph =
        serde_json::from_str(CAMPAIGN_GRAPH_JSON).map_err(|error| {
            FlowEvalError::GraphParse(format!("failed to parse campaign graph: {error}"))
        })?;

    let validation_errors = validate_graph(&graph);
    if !validation_errors.is_empty() {
        return Err(FlowEvalError::GraphValidation(validation_errors));
    }

    Ok(graph)
}

fn evaluate_with_graph(
    graph: &SceneProgressionGraph,
    current_scene: Scene,
    latest: &DilemmaStats,
    stats: &GameStats,
) -> Result<Option<Vec<Scene>>, FlowEvalError> {
    let Some(from_scene_id) = typed_scene_ref_for_runtime_scene(current_scene) else {
        return Ok(None);
    };

    let Some(route) = graph.routes.iter().find(|route| {
        TypedSceneRef::try_from(&route.from)
            .map(|route_from| route_from == from_scene_id)
            .unwrap_or(false)
    }) else {
        return Ok(None);
    };

    let context = FlowEvalContext {
        num_fatalities: latest.num_fatalities,
        num_decisions: latest.num_decisions,
        total_decisions: stats.total_decisions,
        selected_option: latest.result.and_then(|state| state.to_int()),
        duration_remaining_at_last_decision_secs: latest
            .duration_remaining_at_last_decision
            .map(|duration| duration.as_secs_f32()),
        overall_avg_time_remaining_secs: stats
            .overall_avg_time_remaining
            .map(|duration| duration.as_secs_f32()),
    };

    let mut resolved = Vec::new();
    for scene_ref in route.resolve_then(&context) {
        let typed_scene = TypedSceneRef::try_from(scene_ref).map_err(|error| {
            FlowEvalError::RouteMapping(format!("invalid scene id in resolved route: {error}"))
        })?;
        resolved.push(runtime_scene_from_typed_scene_ref(typed_scene)?);
    }

    Ok(Some(resolved))
}

fn typed_scene_ref_for_runtime_scene(scene: Scene) -> Option<TypedSceneRef> {
    match scene {
        Scene::Menu => Some(TypedSceneRef::Menu),
        Scene::Loading => Some(TypedSceneRef::Loading),
        Scene::Dialogue(_) => None,
        Scene::Ending(_) => None,
        Scene::Dilemma(dilemma) => {
            let id = match dilemma {
                DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit) => {
                    DilemmaSceneId::Lab0IncompetentBandit
                }
                DilemmaScene::Lab1(Lab1Dilemma::NearSightedBandit) => {
                    DilemmaSceneId::Lab1NearSightedBandit
                }
                DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem) => {
                    DilemmaSceneId::Lab2TheTrolleyProblem
                }
                DilemmaScene::Lab3(Lab3Dilemma::AsleepAtTheJob) => {
                    DilemmaSceneId::Lab3AsleepAtTheJob
                }
                DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths) => DilemmaSceneId::Lab4RandomDeaths,
                DilemmaScene::PathInaction(_, stage) => DilemmaSceneId::PathInaction {
                    stage: u8::try_from(stage).ok()?,
                },
                DilemmaScene::PathDeontological(_, stage) => DilemmaSceneId::PathDeontological {
                    stage: u8::try_from(stage).ok()?,
                },
                DilemmaScene::PathUtilitarian(_, stage) => DilemmaSceneId::PathUtilitarian {
                    stage: u8::try_from(stage).ok()?,
                },
            };
            Some(TypedSceneRef::Dilemma(id))
        }
    }
}

fn runtime_scene_from_typed_scene_ref(scene: TypedSceneRef) -> Result<Scene, FlowEvalError> {
    let to_usize = |stage: u8| usize::from(stage);

    match scene {
        TypedSceneRef::Menu => Ok(Scene::Menu),
        TypedSceneRef::Loading => Ok(Scene::Loading),
        TypedSceneRef::Dialogue(id) => {
            let dialogue = match id {
                DialogueSceneId::Lab1aFail => DialogueScene::Lab1a(Lab1aDialogue::Fail),
                DialogueSceneId::Lab1aPassIndecisive => {
                    DialogueScene::Lab1a(Lab1aDialogue::PassIndecisive)
                }
                DialogueSceneId::Lab1aFailVeryIndecisive => {
                    DialogueScene::Lab1a(Lab1aDialogue::FailVeryIndecisive)
                }
                DialogueSceneId::Lab1aPass => DialogueScene::Lab1a(Lab1aDialogue::Pass),
                DialogueSceneId::Lab1aPassSlow => DialogueScene::Lab1a(Lab1aDialogue::PassSlow),
                DialogueSceneId::Lab1bIntro => DialogueScene::Lab1b(Lab1bDialogue::DilemmaIntro),
                DialogueSceneId::Lab2aFailIndecisive => {
                    DialogueScene::Lab2a(Lab2aDialogue::FailIndecisive)
                }
                DialogueSceneId::Lab2aFail => DialogueScene::Lab2a(Lab2aDialogue::Fail),
                DialogueSceneId::Lab2aPassSlowAgain => {
                    DialogueScene::Lab2a(Lab2aDialogue::PassSlowAgain)
                }
                DialogueSceneId::Lab2aPassSlow => DialogueScene::Lab2a(Lab2aDialogue::PassSlow),
                DialogueSceneId::Lab2aPass => DialogueScene::Lab2a(Lab2aDialogue::Pass),
                DialogueSceneId::Lab2bIntro => DialogueScene::Lab2b(Lab2bDialogue::Intro),
                DialogueSceneId::Lab3aFailIndecisive => {
                    DialogueScene::Lab3a(Lab3aDialogue::FailIndecisive)
                }
                DialogueSceneId::Lab3aFailInaction => {
                    DialogueScene::Lab3a(Lab3aDialogue::FailInaction)
                }
                DialogueSceneId::Lab3aPassUtilitarian => {
                    DialogueScene::Lab3a(Lab3aDialogue::PassUtilitarian)
                }
                DialogueSceneId::Lab3bIntro => DialogueScene::Lab3b(Lab3bDialogue::Intro),
                DialogueSceneId::Lab4Outro => DialogueScene::Lab4(Lab4Dialogue::Outro),
                DialogueSceneId::PathInaction { stage, outcome } => {
                    DialogueScene::path_inaction(to_usize(stage), path_outcome_from_id(outcome))
                }
                DialogueSceneId::PathDeontological { stage, outcome } => {
                    DialogueScene::path_deontological(
                        to_usize(stage),
                        path_outcome_from_id(outcome),
                    )
                }
                DialogueSceneId::PathUtilitarian { stage, outcome } => {
                    DialogueScene::path_utilitarian(to_usize(stage), path_outcome_from_id(outcome))
                }
            };
            Ok(Scene::Dialogue(dialogue))
        }
        TypedSceneRef::Dilemma(id) => {
            let dilemma = match id {
                DilemmaSceneId::Lab0IncompetentBandit => {
                    DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit)
                }
                DilemmaSceneId::Lab1NearSightedBandit => {
                    DilemmaScene::Lab1(Lab1Dilemma::NearSightedBandit)
                }
                DilemmaSceneId::Lab2TheTrolleyProblem => {
                    DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem)
                }
                DilemmaSceneId::Lab3AsleepAtTheJob => {
                    DilemmaScene::Lab3(Lab3Dilemma::AsleepAtTheJob)
                }
                DilemmaSceneId::Lab4RandomDeaths => DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths),
                DilemmaSceneId::PathInaction { stage } => DilemmaScene::PATH_INACTION
                    .get(to_usize(stage))
                    .copied()
                    .ok_or_else(|| {
                        FlowEvalError::RouteMapping(format!(
                            "path_inaction stage {stage} is out of range"
                        ))
                    })?,
                DilemmaSceneId::PathDeontological { stage } => DilemmaScene::PATH_DEONTOLOGICAL
                    .get(to_usize(stage))
                    .copied()
                    .ok_or_else(|| {
                        FlowEvalError::RouteMapping(format!(
                            "path_deontological stage {stage} is out of range"
                        ))
                    })?,
                DilemmaSceneId::PathUtilitarian { stage } => DilemmaScene::PATH_UTILITARIAN
                    .get(to_usize(stage))
                    .copied()
                    .ok_or_else(|| {
                        FlowEvalError::RouteMapping(format!(
                            "path_utilitarian stage {stage} is out of range"
                        ))
                    })?,
            };
            Ok(Scene::Dilemma(dilemma))
        }
        TypedSceneRef::Ending(id) => {
            let ending = match id {
                EndingSceneId::IdioticPsychopath => EndingScene::IdioticPsychopath,
                EndingSceneId::ImpatientPsychopath => EndingScene::ImpatientPsychopath,
                EndingSceneId::Leverophile => EndingScene::Leverophile,
                EndingSceneId::SelectiveDeontologist => EndingScene::SelectiveDeontologist,
                EndingSceneId::TrueDeontologist => EndingScene::TrueDeontologist,
                EndingSceneId::TrueNeutral => EndingScene::TrueNeutral,
            };
            Ok(Scene::Ending(ending))
        }
    }
}

fn path_outcome_from_id(outcome: PathOutcomeId) -> PathOutcome {
    match outcome {
        PathOutcomeId::Pass => PathOutcome::Pass,
        PathOutcomeId::Fail => PathOutcome::Fail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::stats::DilemmaStats,
        scenes::{dialogue::content::Lab1aDialogue, dilemma::content::Lab2Dilemma},
    };

    const EXAMPLE_GRAPH_JSON: &str = include_str!("./content/campaign_graph.example.json");

    fn example_graph() -> SceneProgressionGraph {
        let graph: SceneProgressionGraph =
            serde_json::from_str(EXAMPLE_GRAPH_JSON).expect("example graph should parse");
        let errors = validate_graph(&graph);
        assert!(errors.is_empty(), "example graph should validate");
        graph
    }

    #[test]
    fn evaluates_example_fatality_route() {
        let graph = example_graph();
        let latest = DilemmaStats {
            num_fatalities: 1,
            ..Default::default()
        };
        let stats = GameStats::default();

        let resolved = evaluate_with_graph(
            &graph,
            Scene::Dilemma(DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit)),
            &latest,
            &stats,
        )
        .expect("graph evaluation should succeed");

        assert!(matches!(
            resolved.as_deref(),
            Some([
                Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::Fail)),
                Scene::Ending(EndingScene::IdioticPsychopath)
            ])
        ));
    }

    #[test]
    fn returns_none_when_scene_has_no_route_in_graph() {
        let graph = example_graph();
        let latest = DilemmaStats::default();
        let stats = GameStats::default();

        let resolved = evaluate_with_graph(
            &graph,
            Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem)),
            &latest,
            &stats,
        )
        .expect("graph evaluation should succeed");

        assert!(resolved.is_none());
    }
}
