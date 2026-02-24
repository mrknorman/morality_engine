use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneProgressionGraph {
    pub version: u32,
    pub routes: Vec<RouteDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteDefinition {
    pub from: SceneRef,
    #[serde(default)]
    pub rules: Vec<RouteRule>,
    #[serde(rename = "default")]
    pub default_then: Vec<SceneRef>,
}

impl RouteDefinition {
    pub fn resolve_then<'a>(&'a self, context: &FlowEvalContext) -> &'a [SceneRef] {
        for rule in &self.rules {
            if rule.matches(context) {
                return &rule.then;
            }
        }

        &self.default_then
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteRule {
    pub name: String,
    #[serde(default)]
    pub when: Vec<FlowCondition>,
    pub then: Vec<SceneRef>,
}

impl RouteRule {
    pub fn matches(&self, context: &FlowEvalContext) -> bool {
        self.when.iter().all(|condition| condition.matches(context))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SceneRef {
    Menu,
    Loading,
    Dialogue { id: String },
    Dilemma { id: String },
    Ending { id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum FlowCondition {
    FatalitiesGt { value: usize },
    FatalitiesEq { value: usize },
    DecisionsGt { value: usize },
    DecisionsEq { value: usize },
    TotalDecisionsGt { value: usize },
    TotalDecisionsEq { value: usize },
    SelectedOptionEq { value: usize },
    LastDecisionRemainingLtSecs { value: f32 },
    LastDecisionRemainingGteSecs { value: f32 },
    OverallAvgRemainingLtSecs { value: f32 },
    OverallAvgRemainingGteSecs { value: f32 },
}

impl FlowCondition {
    fn matches(&self, context: &FlowEvalContext) -> bool {
        match self {
            Self::FatalitiesGt { value } => context.num_fatalities > *value,
            Self::FatalitiesEq { value } => context.num_fatalities == *value,
            Self::DecisionsGt { value } => context.num_decisions > *value,
            Self::DecisionsEq { value } => context.num_decisions == *value,
            Self::TotalDecisionsGt { value } => context.total_decisions > *value,
            Self::TotalDecisionsEq { value } => context.total_decisions == *value,
            Self::SelectedOptionEq { value } => context.selected_option == Some(*value),
            Self::LastDecisionRemainingLtSecs { value } => context
                .duration_remaining_at_last_decision_secs
                .is_some_and(|seconds| seconds < *value),
            Self::LastDecisionRemainingGteSecs { value } => context
                .duration_remaining_at_last_decision_secs
                .is_some_and(|seconds| seconds >= *value),
            Self::OverallAvgRemainingLtSecs { value } => context
                .overall_avg_time_remaining_secs
                .is_some_and(|seconds| seconds < *value),
            Self::OverallAvgRemainingGteSecs { value } => context
                .overall_avg_time_remaining_secs
                .is_some_and(|seconds| seconds >= *value),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FlowEvalContext {
    pub num_fatalities: usize,
    pub num_decisions: usize,
    pub total_decisions: usize,
    pub selected_option: Option<usize>,
    pub duration_remaining_at_last_decision_secs: Option<f32>,
    pub overall_avg_time_remaining_secs: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_GRAPH_JSON: &str = include_str!("./content/campaign_graph.example.json");

    #[test]
    fn parses_example_graph_contract() {
        let graph: SceneProgressionGraph =
            serde_json::from_str(EXAMPLE_GRAPH_JSON).expect("example graph should parse");

        assert_eq!(graph.version, 1);
        assert!(!graph.routes.is_empty());
        assert!(graph.routes.iter().all(|route| !route.default_then.is_empty()));
    }

    #[test]
    fn resolve_then_uses_first_matching_rule_in_declared_order() {
        let route = RouteDefinition {
            from: SceneRef::Dilemma {
                id: String::from("lab_0.incompetent_bandit"),
            },
            rules: vec![
                RouteRule {
                    name: String::from("fatalities_gt_0"),
                    when: vec![FlowCondition::FatalitiesGt { value: 0 }],
                    then: vec![SceneRef::Ending {
                        id: String::from("idiotic_psychopath"),
                    }],
                },
                RouteRule {
                    name: String::from("fallback_decisions_gt_0"),
                    when: vec![FlowCondition::DecisionsGt { value: 0 }],
                    then: vec![SceneRef::Dialogue {
                        id: String::from("lab_1.a.pass_indecisive"),
                    }],
                },
            ],
            default_then: vec![SceneRef::Menu],
        };

        let context = FlowEvalContext {
            num_fatalities: 1,
            num_decisions: 10,
            total_decisions: 10,
            selected_option: None,
            duration_remaining_at_last_decision_secs: None,
            overall_avg_time_remaining_secs: None,
        };

        let resolved = route.resolve_then(&context);

        assert_eq!(
            resolved,
            &[SceneRef::Ending {
                id: String::from("idiotic_psychopath")
            }]
        );
    }

    #[test]
    fn resolve_then_falls_back_to_default_when_no_rules_match() {
        let route = RouteDefinition {
            from: SceneRef::Dilemma {
                id: String::from("lab_2.the_trolley_problem"),
            },
            rules: vec![RouteRule {
                name: String::from("requires_selected_option_1"),
                when: vec![FlowCondition::SelectedOptionEq { value: 1 }],
                then: vec![SceneRef::Ending {
                    id: String::from("leverophile"),
                }],
            }],
            default_then: vec![SceneRef::Dialogue {
                id: String::from("lab_3.a.pass_utilitarian"),
            }],
        };

        let context = FlowEvalContext {
            selected_option: Some(0),
            ..Default::default()
        };

        let resolved = route.resolve_then(&context);

        assert_eq!(
            resolved,
            &[SceneRef::Dialogue {
                id: String::from("lab_3.a.pass_utilitarian")
            }]
        );
    }
}
