use std::collections::HashSet;

use super::{
    ids::{SceneIdParseError, TypedSceneRef},
    schema::{RouteDefinition, SceneProgressionGraph, SceneRef},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphValidationError {
    DuplicateRouteSource { from: SceneRef },
    UnsupportedRouteSource { from: SceneRef },
    EmptyDefaultRoute { from: SceneRef },
    EmptyRuleRoute { from: SceneRef, rule_name: String },
    DuplicateRuleName { from: SceneRef, rule_name: String },
    UnknownSceneId {
        context: String,
        error: SceneIdParseError,
    },
}

pub fn validate_graph(graph: &SceneProgressionGraph) -> Vec<GraphValidationError> {
    let mut errors = Vec::new();
    let mut route_sources = HashSet::new();

    for route in &graph.routes {
        if !route_sources.insert(route.from.clone()) {
            errors.push(GraphValidationError::DuplicateRouteSource {
                from: route.from.clone(),
            });
        }
        validate_route(route, &mut errors);
    }

    errors
}

fn validate_route(route: &RouteDefinition, errors: &mut Vec<GraphValidationError>) {
    validate_scene_ref(&route.from, "route.from", errors);

    if !matches!(TypedSceneRef::try_from(&route.from), Ok(TypedSceneRef::Dilemma(_))) {
        errors.push(GraphValidationError::UnsupportedRouteSource {
            from: route.from.clone(),
        });
    }

    if route.default_then.is_empty() {
        errors.push(GraphValidationError::EmptyDefaultRoute {
            from: route.from.clone(),
        });
    }

    for (default_index, scene) in route.default_then.iter().enumerate() {
        let context = format!("route.default[{default_index}]");
        validate_scene_ref(scene, &context, errors);
    }

    let mut rule_names = HashSet::new();
    for (rule_index, rule) in route.rules.iter().enumerate() {
        if !rule_names.insert(rule.name.clone()) {
            errors.push(GraphValidationError::DuplicateRuleName {
                from: route.from.clone(),
                rule_name: rule.name.clone(),
            });
        }

        if rule.then.is_empty() {
            errors.push(GraphValidationError::EmptyRuleRoute {
                from: route.from.clone(),
                rule_name: rule.name.clone(),
            });
        }

        for (then_index, scene) in rule.then.iter().enumerate() {
            let context = format!("route.rules[{rule_index}].then[{then_index}]");
            validate_scene_ref(scene, &context, errors);
        }
    }
}

fn validate_scene_ref(scene_ref: &SceneRef, context: &str, errors: &mut Vec<GraphValidationError>) {
    if let Err(error) = TypedSceneRef::try_from(scene_ref) {
        errors.push(GraphValidationError::UnknownSceneId {
            context: context.to_string(),
            error,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenes::flow::schema::{RouteRule, SceneProgressionGraph};

    const EXAMPLE_GRAPH_JSON: &str = include_str!("./content/campaign_graph.example.json");

    #[test]
    fn example_graph_passes_validation() {
        let graph: SceneProgressionGraph =
            serde_json::from_str(EXAMPLE_GRAPH_JSON).expect("example graph should parse");

        let errors = validate_graph(&graph);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn reports_unknown_scene_ids() {
        let graph = SceneProgressionGraph {
            version: 1,
            routes: vec![RouteDefinition {
                from: SceneRef::Dilemma {
                    id: String::from("lab_0.incompetent_bandit"),
                },
                rules: vec![],
                default_then: vec![SceneRef::Dialogue {
                    id: String::from("lab_999.invalid"),
                }],
            }],
        };

        let errors = validate_graph(&graph);

        assert!(matches!(
            errors.as_slice(),
            [GraphValidationError::UnknownSceneId { .. }]
        ));
    }

    #[test]
    fn reports_duplicate_route_sources() {
        let route = RouteDefinition {
            from: SceneRef::Dilemma {
                id: String::from("lab_0.incompetent_bandit"),
            },
            rules: vec![],
            default_then: vec![SceneRef::Menu],
        };

        let graph = SceneProgressionGraph {
            version: 1,
            routes: vec![route.clone(), route],
        };

        let errors = validate_graph(&graph);

        assert!(errors.iter().any(|error| matches!(
            error,
            GraphValidationError::DuplicateRouteSource { .. }
        )));
    }

    #[test]
    fn reports_empty_default_routes() {
        let graph = SceneProgressionGraph {
            version: 1,
            routes: vec![RouteDefinition {
                from: SceneRef::Dilemma {
                    id: String::from("lab_1.near_sighted_bandit"),
                },
                rules: vec![],
                default_then: vec![],
            }],
        };

        let errors = validate_graph(&graph);

        assert!(errors.iter().any(|error| matches!(
            error,
            GraphValidationError::EmptyDefaultRoute { .. }
        )));
    }

    #[test]
    fn reports_duplicate_rule_names() {
        let graph = SceneProgressionGraph {
            version: 1,
            routes: vec![RouteDefinition {
                from: SceneRef::Dilemma {
                    id: String::from("lab_2.the_trolley_problem"),
                },
                rules: vec![
                    RouteRule {
                        name: String::from("duplicate"),
                        when: vec![],
                        then: vec![SceneRef::Menu],
                    },
                    RouteRule {
                        name: String::from("duplicate"),
                        when: vec![],
                        then: vec![SceneRef::Loading],
                    },
                ],
                default_then: vec![SceneRef::Menu],
            }],
        };

        let errors = validate_graph(&graph);

        assert!(errors.iter().any(|error| matches!(
            error,
            GraphValidationError::DuplicateRuleName { .. }
        )));
    }

    #[test]
    fn reports_unsupported_non_dilemma_route_sources() {
        let graph = SceneProgressionGraph {
            version: 1,
            routes: vec![RouteDefinition {
                from: SceneRef::Dialogue {
                    id: String::from("lab_1.a.fail"),
                },
                rules: vec![],
                default_then: vec![SceneRef::Menu],
            }],
        };

        let errors = validate_graph(&graph);

        assert!(errors.iter().any(|error| matches!(
            error,
            GraphValidationError::UnsupportedRouteSource { .. }
        )));
    }
}
