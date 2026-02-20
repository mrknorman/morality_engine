# Extension Playbook

Use this when adding new UI behavior.

## Core Rules

1. Start from primitives in `src/systems/ui/*`.
2. Keep ownership explicit via `UiLayer { owner, kind }`.
3. Keep interaction truth in `Hoverable`, `Clickable`, `Pressable`, and `Selectable`.
4. Keep visual-only state in `InteractionVisualState`.
5. Keep query mutability disjoint (`ParamSet` or `Without<T>`).
6. For reusable primitives, do not add new Bundle-first APIs; prefer root component + `#[require]` + `on_insert`.
7. Primitives should be self-contained: inserting the root should produce working behavior, not just visuals.

## Recommended Workflow

1. Define a reusable primitive API first (component + system contract).
   - Use required components and insert hooks as the default construction pattern.
2. Add deterministic arbitration rules before visuals.
3. Add tests for mixed keyboard/mouse and layer transitions.
4. Integrate into menu composition (`src/systems/ui/menu/*`) without embedding feature logic in primitive modules.
5. Document extension points in `docs/ui_ecs_reference.md`.

## Quick Links

- [Architecture Contract](./ui_architecture_contract.md)
- [ECS Reference](./ui_ecs_reference.md)
- [Scrollable Usage](./ui_scrollable_usage.md)
