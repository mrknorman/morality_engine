# Scene Flow Graph Authoring Guide

Last updated: 2026-02-24

## Purpose

Author and maintain campaign branching in:

- `src/scenes/flow/content/campaign_graph.json`

without modifying Rust branching code.

## Authoring Model

Each route has:

1. `from`: source scene id (`SceneRef`)
2. `rules`: ordered first-match rule list
3. `default`: mandatory fallback route

Rules use AND semantics across `when` conditions.

## Scene ID Conventions

Kinds:

- `menu`
- `loading`
- `dialogue`
- `dilemma`
- `ending`

IDs are validated by typed registries in:

- `src/scenes/flow/ids.rs`

If an ID is unknown, graph validation fails before runtime use.

## Condition Conventions

Use `FlowCondition` operators from:

- `src/scenes/flow/schema.rs`

Key guidance:

1. Prefer explicit presence checks when comparing optional timing fields.
2. Keep rule ordering aligned with intended priority/precedence.
3. Use `default` for guaranteed fallback behavior.

## Editing Workflow

1. Edit `campaign_graph.json`.
2. Run validation tests:
   - `cargo test scenes::flow::schema::tests`
   - `cargo test scenes::flow::validate::tests`
3. Run behavior tests:
   - `cargo test scenes::flow::tests`
4. Run compile gate:
   - `cargo check`
5. For substantial branching edits, manually run in-game progression paths from the scene manual checklist.

## Safety Rules

1. Never leave a route without `default`.
2. Avoid duplicate route sources.
3. Avoid duplicate rule names within a route.
4. Keep route arrays non-empty (`default` and each rule `then`).
5. Preserve deterministic first-match semantics; do not encode ambiguous overlapping rules without explicit order intent.
