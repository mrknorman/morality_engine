# Scene System Refactor Plan

Last updated: 2026-02-24

## Goal

Refactor scene flow and scene module boundaries so runtime behavior is fault-tolerant,
ECS-friendly, modular, and extensible, while aligning with the UI architecture model:

- primitives/runtime in shared modules
- composition/policy in dedicated modules
- scene modules as usage consumers

## Execution Protocol (Mandatory)

1. Work in explicit stages.
2. End each stage with a clean repository and one checkpoint commit.
3. Run `cargo check` at each stage boundary.
4. Update scene documentation at each stage boundary (not deferred).
5. No panic-driven runtime control flow for recoverable errors.

## Stage Status

- Stage 0 complete: `d2772a1`
- Stage 1 complete: `5152bf5`
- Stage 2 complete: `eebf918`
- Stage 3 complete: `13d49d5`
- Stage 4 complete: `aa749d6`
- Stage 5 complete: `14bf517`
- Stage 6 complete: `751b002`
- Stage 7 complete: docs finalization and validation
- Stage 8 complete: lever ownership normalization, shared plugin consolidation, train fallback hardening

## Scene Documentation Set

- `docs/scene_architecture_contract.md`
- `docs/scene_flow_reference.md`
- `docs/scene_compliance_matrix.md`
- `docs/scene_manual_validation_checklist.md`
- `docs/scene_progression_graph_spec.md`
- `docs/scene_flow_graph_authoring.md`

## Data-Driven Branching Track

Status:
- Stage 1 complete: progression graph schema + contract + example content
- Stage 2 complete: validator + typed scene IDs
- Stage 3 complete: graph evaluator runtime with hardcoded fallback
- Stage 4 complete: parity shadow mode and tests for graph-covered routes
- Stage 5 complete: campaign graph migration (shadow-mode parity retained)
- Stage 6 complete: graph cutover + hardcoded path removal
- Stage 7 complete: graph authoring docs + validation workflow

## High-Level Issue Status

Resolved in Stages 1-8:
1. Scene flow panic paths in core progression were replaced with fallbacks and guarded routing.
2. Scene-to-state mapping duplication was removed through `SceneNavigator`.
3. Campaign/branching policy was moved into `src/scenes/flow/mod.rs`.
4. Shared plugin/dependency wiring was centralized in `src/scenes/composition.rs`.
5. Lever ownership was normalized to a single contract (`Lever` resource + `LeverRoot` entity marker).
6. Scene-local ad-hoc registration of shared sprite/text/cascade plugins was removed.
7. Train setup/content-loading runtime paths now use warning + fallback behavior instead of hard panics.
8. Singleton assumptions in scene phases were reviewed; current remaining `Single` call sites are invariant-driven and non-panicking.

Remaining targeted follow-up (tracked in compliance matrix):
1. Build campaign graph viewer/editor tooling for non-linear authoring workflows (`SCN-016`).
2. Continue targeted audits of scene-content/runtime setup for fallback quality and error observability.

## Stage Plan

### Stage 0: Baseline Checkpoint and Protocol

Scope:
- Record baseline in a single checkpoint commit on `refactor/scene-system-refactor`.
- Add this plan document.

Exit gate:
- `cargo check` passes.
- `git status` is clean.

### Stage 1: Scene Runtime Primitives and Architecture Contract

Scope:
- Introduce shared scene runtime primitives (`SceneNavigator`, route mapping, queue API).
- Add a scene architecture contract doc with module boundary rules.

Exit gate:
- Runtime primitives compile with tests for route and queue invariants.

### Stage 2: Fault-Tolerant Flow (Remove Panic-Driven Runtime)

Scope:
- Replace recoverable `panic!/expect!/todo!` flow control in scene progression with guarded
  fallbacks and explicit error handling.

Exit gate:
- Core scene progression paths recover safely on bad state/content.

### Stage 3: Unify Scene Transition Dispatch

Scope:
- Remove duplicated `Scene -> state` dispatch logic from menu/interaction call sites.
- Route all next-scene transitions through one source of truth.

Exit gate:
- Single canonical transition mapping in runtime module.

### Stage 4: Isolate Campaign/Branching Policy

Scope:
- Move dilemma path/campaign branching out of scene runtime into dedicated policy module.
- Ensure unresolved branches have explicit outcomes.

Exit gate:
- Branching logic testable independently from scene setup systems.

### Stage 5: Normalize Plugin and Dependency Wiring

Scope:
- Consolidate shared scene plugin dependencies in one composition layer.
- Keep scene modules focused on scene-local systems.

Exit gate:
- Reduced per-scene plugin duplication and clear dependency direction.

### Stage 6: ECS Consistency and Query Safety

Scope:
- Normalize ownership contracts for scene runtime state.
- Replace fragile singleton assumptions with explicit root/query guards.

Exit gate:
- Scene runtime behavior no longer depends on implicit singleton cardinality.

### Stage 7: Documentation Finalization and Validation

Scope:
- Publish scene architecture/reference/compliance/checklist docs.
- Run scene manual validation checklist and final compile gate.

Exit gate:
- Scene docs complete and linked.
- `cargo check` passes.
- Clean checkpoint commit.

### Stage 8: Ownership and Composition Completion

Scope:
- Normalize lever ownership to one ECS ownership contract.
- Remove remaining per-scene registration of shared composition plugins.
- Harden train setup/content-loading runtime behavior with safe fallbacks.

Exit gate:
- No resource+component dual ownership for lever state.
- Shared sprite/text/cascade dependencies come from composition plugin layer.
- Targeted scene/runtime tests pass with no new panic-driven flow paths.

## Commit Convention

Use one commit per stage with clear scope prefixes, for example:

- `chore(scene-refactor): checkpoint stage 0 baseline`
- `refactor(scenes): stage 1 runtime primitives`
- `refactor(scenes): stage 2 fault-tolerant flow`
- `refactor(scenes): stage 3 unified transition dispatch`
- `refactor(scenes): stage 4 branching policy extraction`
- `refactor(scenes): stage 5 plugin/dependency normalization`
- `refactor(scenes): stage 6 ecs ownership/query safety`
- `docs(scenes): stage 7 finalize docs and validation`
- `refactor(scenes): stage 8 ownership/composition completion`
