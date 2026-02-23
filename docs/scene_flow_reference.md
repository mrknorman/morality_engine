# Scene Flow Reference

Last updated: 2026-02-23

## Canonical Transition Source

`Scene -> state` routing is defined in one place:

- `src/scenes/runtime/mod.rs`
  - `SceneNavigator::state_vector_for(...)`
  - `SceneNavigator::next_state_vector_or_fallback(...)`

No other module should define or duplicate this mapping.

Campaign branching policy is isolated in:

- `src/scenes/flow/mod.rs`
  - `next_scenes_for_current_dilemma(...)`

Shared cross-scene dependency wiring is centralized in:

- `src/scenes/composition.rs`
  - `SceneCompositionPlugin`

## Routing Table

| Scene | MainState | GameState | DilemmaPhase |
|---|---|---|---|
| `Scene::Menu` | `Menu` | unchanged | unchanged |
| `Scene::Loading` | `InGame` | `Loading` | unchanged |
| `Scene::Dialogue(_)` | `InGame` | `Dialogue` | unchanged |
| `Scene::Dilemma(_)` | `InGame` | `Dilemma` | `Intro` |
| `Scene::Ending(_)` | `InGame` | `Ending` | unchanged |

## Queue Advancement Contract

1. `SceneQueue::try_pop()` is the canonical non-panicking dequeue operation.
2. `SceneNavigator::advance(...)` returns an error when the queue is empty.
3. `SceneNavigator::next_state_vector_or_fallback(...)` guarantees a route by
   falling back to menu when the queue cannot advance.
4. Scene consumers read queue state through `SceneQueue::current_scene()` and
   `SceneQueue::next_scene()`.
5. Campaign reset uses `SceneQueue::reset_campaign()` to keep queue ownership
   explicit.

## Integration Points

Current consumers of canonical scene routing:

- `src/systems/interaction/mod.rs`
  - `trigger_next_scene`
- `src/systems/ui/menu/command_effects.rs`
  - `handle_next_scene_command`

Scene modules consume shared dependencies via one composition layer:

- `src/scenes/mod.rs`
  - `SceneCompositionPlugin` is registered once before scene usage plugins

## Fault-Tolerance Behavior

When a next-scene action occurs and the queue is empty:

- runtime returns the menu fallback state vector
- state transition resolves safely to `MainState::Menu`
- no panic is used for this recoverable flow path

## Related Docs

- `docs/scene_architecture_contract.md`
- `docs/scene_compliance_matrix.md`
- `docs/scene_manual_validation_checklist.md`
