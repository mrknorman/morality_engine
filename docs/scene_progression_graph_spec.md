# Scene Progression Graph Spec

Last updated: 2026-02-24

## Goal

Represent campaign branching as content data instead of hardcoded Rust path trees,
while keeping runtime evaluation deterministic and fault-tolerant.

## File Format

Graph files are JSON deserialized into:

- `SceneProgressionGraph`
- `RouteDefinition`
- `RouteRule`
- `SceneRef`
- `FlowCondition`

Current schema implementation:

- `src/scenes/flow/schema.rs`
- typed id registry: `src/scenes/flow/ids.rs`
- validator: `src/scenes/flow/validate.rs`
- runtime evaluator: `src/scenes/flow/engine.rs`
- example content: `src/scenes/flow/content/campaign_graph.example.json`
- runtime graph content: `src/scenes/flow/content/campaign_graph.json`

## Top-Level Contract

`SceneProgressionGraph` fields:

- `version: u32`
- `routes: Vec<RouteDefinition>`

Each `RouteDefinition` contains:

- `from: SceneRef`
- `rules: Vec<RouteRule>` (ordered, optional)
- `default: Vec<SceneRef>` (required non-empty fallback route)

## Deterministic Rule Semantics

Route evaluation is strict first-match in declared order:

1. Iterate `rules` from index 0 upward.
2. For each rule, evaluate all `when` conditions with AND semantics.
3. Return `then` for the first rule that matches.
4. If no rule matches, return `default`.

No randomized dispatch is allowed inside graph evaluation.

## Condition Model (Stage 1)

Supported `FlowCondition` operators:

- `fatalities_gt`, `fatalities_eq`
- `decisions_gt`, `decisions_eq`
- `total_decisions_gt`, `total_decisions_eq`
- `selected_option_eq`
- `last_decision_remaining_is_some`, `last_decision_remaining_is_none`
- `last_decision_remaining_lt_secs`, `last_decision_remaining_gte_secs`
- `overall_avg_remaining_is_some`, `overall_avg_remaining_is_none`
- `overall_avg_remaining_lt_secs`, `overall_avg_remaining_gte_secs`

Evaluation context source model:

- latest dilemma stats
- aggregate game stats
- selected option (if present)

## Stage Boundaries

Stage 1 scope:

- graph schema/types
- first-match deterministic semantics
- parseable example graph content

Out of Stage 1 scope:

- full route validation
- typed scene-id registry
- runtime cutover from hardcoded branch tree

Those are handled in subsequent stages.

## Stage 2 Validation Contract

Graph validation currently enforces:

1. Every scene reference resolves to a typed ID (`TypedSceneRef`).
2. Every route has a non-empty `default`.
3. Route sources are dilemma-scoped (`SceneRef::Dilemma`) for campaign progression.
4. Duplicate route sources are rejected.
5. Duplicate rule names within a route are rejected.
6. Empty `then` routes in rules are rejected.

## Stage 3 Runtime Contract

Runtime evaluation behavior:

1. Attempt graph evaluation first via `evaluate_next_scenes_from_graph(...)`.
2. If route exists in graph, return graph-selected scenes.
3. If scene has no graph route, return `None` to allow caller fallback policy.
4. If graph load/validation/evaluation fails, runtime logs warning and uses hardcoded branch fallback.

## Stage 4 Parity Contract

Shadow-mode behavior:

1. Runtime computes hardcoded baseline and graph route in the same call.
2. Baseline remains authoritative during migration.
3. If graph and baseline differ for a graph-covered route, runtime logs a mismatch snapshot
   (scene kind + key stats context).
4. Parity tests must assert graph/baseline equality for every currently graph-covered route.

## Stage 5 Migration Contract

Campaign migration requirements:

1. `campaign_graph.json` covers all currently hardcoded branch source scenes.
2. Graph rule ordering preserves previous if/else precedence semantics.
3. Path-stage progressions (`path_inaction`, `path_deontological`, `path_utilitarian`) are encoded
   explicitly as route sources and next-scene outputs.
4. Shadow-mode parity tests cover representative branches across all migrated route families.
