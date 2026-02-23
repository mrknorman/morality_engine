# Scene Architecture Contract

This contract defines strict scene-system boundaries so scene flow remains modular,
ECS-friendly, and fault-tolerant under iteration.

## 1. Module Boundaries

### `src/scenes/runtime/*` (runtime primitives)
- Owns generic scene-runtime mechanics.
- Allowed responsibilities:
  - queue operations and queue invariants
  - scene route mapping (`Scene -> StateVector`)
  - generic scene navigation helpers
- Must not depend on:
  - dilemma-specific branching policy
  - UI menu policy reducers
  - per-scene content/asset parsing rules

### `src/scenes/composition/*` (shared composition wiring)
- Owns shared scene dependency wiring.
- Allowed responsibilities:
  - registering cross-scene plugins used by multiple scene modules
  - centralizing plugin ownership so scene modules do not duplicate it
- Must not:
  - include scene-local setup/update systems
  - own campaign branching policy
  - own route mapping internals

### `src/scenes/flow/*` (composition/policy)
- Owns campaign and branching policy.
- Allowed responsibilities:
  - mapping stats and outcomes to next-scene sequences
  - selecting path progression
- Must not:
  - re-implement queue internals
  - mutate unrelated scene entity internals

### `src/scenes/*` (scene usage modules)
- `menu`, `loading`, `dialogue`, `dilemma`, `ending` are scene consumers.
- Allowed responsibilities:
  - scene-local entity spawn/despawn
  - scene-local audio/visual/input components
  - invoking runtime primitives for scene navigation
- Must not:
  - duplicate global scene-route mapping tables
  - duplicate campaign policy branches that belong in `flow/*`

## 2. Fault-Tolerance Rules

1. Recoverable runtime conditions must not use `panic!/todo!/unwrap()/expect()`.
2. Invalid queue state and scene mismatches must route through explicit fallback behavior.
3. Parsing/configuration errors should return typed errors and produce a safe fallback route.
4. Fail-hard behavior is allowed only for compile-time invariants or explicit debug assertions.

## 3. Scene Route Source of Truth

1. Route mapping from `Scene` to `MainState/GameState/DilemmaPhase` must exist in one runtime location.
2. Systems that advance scenes (`interaction`, `menu command effects`, scene-local flows)
   consume the same runtime mapper.
3. Scene modules may request transitions, but they must not own duplicated mapping tables.

## 4. Queue and Ownership Rules

1. Queue reads/writes are centralized via `SceneQueue` APIs (`try_pop`, `replace`, `current_scene`, `next_scene`, `reset_campaign`).
2. Scene progression must not rely on implicit cardinality assumptions.
3. Scene modules and systems must not read queue internals directly when a queue API exists.
4. Shared runtime state (for example lever/selection state) must have explicit ownership contracts:
   - either resource-owned, or entity-owned + root marker, but not mixed ad hoc.

## 5. Dependency Direction

Allowed direction:
1. `scenes/runtime` -> `data/states` + core scene enum types.
2. `scenes/composition` -> shared plugins/resources used across scene modules.
3. `scenes/flow` -> `scenes/runtime` + scene content enums + stats.
4. `scenes/*` usage modules -> `scenes/runtime` and `scenes/flow` APIs.
5. `systems/*` (interaction/menu) -> `scenes/runtime` transition APIs.

Disallowed direction:
- `scenes/runtime` -> `scenes/dilemma/*` phase internals.
- `scenes/runtime` -> `systems/ui/menu/*` policy internals.
- scene usage modules owning duplicate global transition mapping logic.

## 6. Query-Safety Rules

1. Systems reading and mutating related scene runtime sets must use disjoint query contracts.
2. Prefer explicit root markers over `.iter().next().expect(...)` singleton assumptions.
3. Root-cardinality-sensitive systems should use guarded single-entity access (`single()` / `Single`) with explicit fallback behavior.
4. Cross-scene cleanup systems must target tagged entities/components, not broad archetype sweeps.

## 7. Documentation Rules

1. Scene runtime API changes must be recorded in `docs/scene_refactor_plan.md`.
2. Routing/flow semantics must be documented in a scene flow reference doc.
3. Compliance status is tracked in a scene compliance matrix.

## 8. Acceptance Criteria

1. Scene progression has one canonical route mapping implementation.
2. Recoverable scene-flow faults no longer hard-crash runtime.
3. Campaign branching policy is modular and testable without scene setup systems.
4. Scene modules remain usage-focused with minimal global policy logic.
