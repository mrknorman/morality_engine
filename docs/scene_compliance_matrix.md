# Scene Compliance Matrix

Last updated: 2026-02-23

Scope:
- `src/scenes/*`
- scene transition integration points in `src/systems/interaction/*` and `src/systems/ui/menu/*`

Legend:
- `Compliant`: matches current scene ECS contract and refactor goals.
- `Partial`: largely aligned, but still has targeted debt that should be removed in follow-up work.
- `Non-compliant`: violates the current scene architecture contract.

## Runtime / Composition / Policy

| Module | Status | Current Pattern | Follow-up |
|---|---|---|---|
| `src/scenes/runtime/mod.rs` | Compliant | Canonical `Scene -> StateVector` mapping and queue advance/fallback helpers. | Keep this as the only global route source of truth. |
| `src/scenes/composition.rs` | Compliant | Shared cross-scene plugin wiring is centralized in one composition plugin. | Add new shared dependencies here instead of scene modules. |
| `src/scenes/flow/mod.rs` | Compliant | Campaign branching policy is isolated from scene setup/runtime dispatch. | Keep content-path branching localized to this module. |
| `src/scenes/mod.rs` (`SceneQueue`) | Compliant | Queue ownership API (`try_pop`, `replace`, `current_scene`, `next_scene`, `reset_campaign`). | Continue moving callsites to queue APIs instead of direct internals. |

## Scene Usage Modules

| Module | Status | Current Pattern | Follow-up |
|---|---|---|---|
| `src/scenes/menu/mod.rs` | Compliant | Scene-local setup only; shared dependencies come from composition layer. | None. |
| `src/scenes/loading/mod.rs` | Compliant | Root/timer updates use guarded single-root access with safe early return. | None. |
| `src/scenes/dialogue/mod.rs` | Compliant | Runtime routing/fallback through `SceneNavigator`; queue access uses queue APIs. | None. |
| `src/scenes/dilemma/mod.rs` | Compliant | Runtime routing/fallback through `SceneNavigator`; scene-local setup remains modular. | None. |
| `src/scenes/ending/mod.rs` | Compliant | Typed parse + fallback route on ending-content failure. | None. |

## Scene Content Loader Safety

| Module | Status | Current Pattern | Follow-up |
|---|---|---|---|
| `src/scenes/dialogue/dialogue.rs` | Partial | Dialogue content parse/character resolution still uses `expect(...)`. | Convert to typed loader errors + route-safe fallback for content iteration. |
| `src/scenes/dilemma/dilemma.rs` | Partial | Dilemma JSON parse still uses `expect(...)`. | Convert to typed loader errors and scene fallback behavior. |
| `src/scenes/loading/loading_bar.rs` | Partial | Loading bar config parse still uses `expect(...)`. | Return typed config errors and keep loading scene recoverable. |

## Transition Consumers

| Module | Status | Current Pattern | Follow-up |
|---|---|---|---|
| `src/systems/interaction/mod.rs` (`trigger_next_scene`) | Compliant | Uses `SceneNavigator::next_state_vector_or_fallback(...)`. | None. |
| `src/systems/ui/menu/command_effects.rs` (`handle_next_scene_command`) | Compliant | Uses `SceneNavigator::next_state_vector_or_fallback(...)`. | None. |
