# UI Secondary Refactor Plan

Purpose:
- Enforce strict primitive architecture across all UI modules.
- Eliminate partial/visual-only compositions for interactive features.
- Ensure debug UI showcase examples are fully interactive and built from the same reusable primitives.

## Guardrails (Non-negotiable)

1. No new Bundle-first APIs for reusable UI primitives.
2. New primitives must use:
   - Root component
   - `#[require(...)]` contracts
   - `#[component(on_insert = ...)]` hierarchy construction
3. Primitives must be self-contained and independently operable.
4. Interaction behavior must read primitive truth components (`Hoverable`, `Clickable`, `Pressable`, `SelectableMenu`, `Selectable`, `OptionCycler`) and not `InteractionVisualState`.

## Stage 1: Audit and Classification

- [x] Inventory all UI modules under `src/systems/ui/*` and `src/systems/ui/menu/*`.
- [x] Classify each feature as:
  - Primitive-compliant
  - Partially compliant
  - Non-compliant (bundle-first, externally wired internals, or visual-only facades).
- [x] Produce a migration table with owner file, current construction pattern, and target primitive pattern.

Deliverable:
- A compliance matrix appended to this plan.

### Initial Compliance Matrix (Baseline)

| Module | Classification | Notes |
| --- | --- | --- |
| `src/systems/ui/layer.rs` | Primitive-compliant | Owner-scoped layer resolution is reusable and domain-agnostic. |
| `src/systems/ui/dropdown.rs` | Primitive-compliant | Generic owner-scoped dropdown state + helpers, no menu domain coupling. |
| `src/systems/ui/tabs.rs` | Primitive-compliant | Generic tab state/activation helpers. |
| `src/systems/ui/selector.rs` | Primitive-compliant | Generic shortcut + selector bound helpers. |
| `src/systems/ui/scroll/*` | Primitive-compliant | Reusable render-to-texture scroll + scrollbar primitives with tests. |
| `src/systems/ui/hover_box.rs` | Primitive-compliant | Reusable hover tooltip primitive. |
| `src/systems/ui/discrete_slider.rs` | Partially compliant | Reusable widget, but showcase/adapters need stronger self-contained root integration. |
| `src/systems/ui/menu/*` | Composition-heavy / partially compliant | Strong reducer/effects split, but still contains some primitive-like behavior that should migrate down into `systems/ui/*`. |
| `src/systems/ui/menu/debug_showcase.rs` | Primitive-compliant | Dedicated self-contained showcase root with insert-hook composition and interactive primitive demos. |

## Stage 2: Primitive Contract Normalization

- [ ] Define and standardize root primitive components for:
  - Menu surface
  - Selector/cycler surface
  - Tab bar
  - Dropdown
  - Scrollable/scrollbar
  - Hover box
- [ ] Ensure each root primitive owns its required child hierarchy via insert hooks.
- [ ] Remove hidden external wiring assumptions from primitive behavior systems.

Deliverable:
- Primitive APIs that can be used by adding a single root component plus config components.

## Stage 3: Menu Composition Migration

- [ ] Refactor `src/systems/ui/menu/*` to compose primitives only, not rebuild primitive internals.
- [ ] Reduce menu-specific branching where primitive state machines already exist.
- [ ] Keep reducer/effects split intact while moving mechanics to primitives.

Deliverable:
- Menu modules focused on policy and command mapping.

## Stage 4: Debug UI Showcase Rebuild

- [x] Move debug showcase into its own module (outside command effect body).
- [x] Replace table-only demos with primitive-backed demos:
  - Interactive selector menu window
  - Interactive tabs window
  - Interactive dropdown window
  - Interactive scroll window
- [x] Ensure each demo window is assembled from reusable primitives with no one-off interaction logic.

Deliverable:
- Debug showcase as a reference implementation of composable primitives.

## Stage 5: Interaction and Layer Conformance Pass

- [ ] Verify owner-scoped layer behavior for all primitives (`Base`, `Dropdown`, `Modal`).
- [ ] Verify input arbitration consistency (keyboard lock, hover handoff, active layer priority).
- [ ] Remove any behavior paths that still depend on visual-state fields.

Deliverable:
- Deterministic interaction behavior under mixed mouse+keyboard input.

## Stage 6: Query-Safety Hardening

- [ ] Audit all UI systems for overlapping mutable query risk.
- [ ] Apply `ParamSet` and `Without<T>` contracts where needed.
- [ ] Add short query contract comments at each multi-query system.

Deliverable:
- No B0001 panics under stress interactions in menus/debug showcase.

## Stage 7: Test Coverage

- [x] Add/extend unit tests for primitive reducers and state transitions.
- [ ] Add integration-level tests for:
  - Tabs activation and content switching
  - Dropdown open/select/close paths
  - Selector/cycler keyboard+mouse interaction
  - Scrollbar drag and wheel behavior
- [ ] Add debug showcase interaction smoke test hooks where feasible.

Deliverable:
- Regression safety for primitive behavior and composition.

Progress notes:
- `command_reducer` and menu flow tests are in place (`src/systems/ui/menu/command_reducer.rs`, `src/systems/ui/menu/flow_tests.rs`).
- Debug showcase now has targeted unit coverage for core index-cycling behavior (`src/systems/ui/menu/debug_showcase.rs`).

## Stage 8: Documentation and Adoption

- [ ] Update docs/examples to use primitive-root construction patterns.
- [ ] Add "do/dont" examples for primitive composition in `docs/ui_ecs_reference.md`.
- [ ] Mark deprecated bundle-first helper surfaces as migration targets.

Deliverable:
- Documentation aligned with implementation and enforced conventions.

## Acceptance Checklist

- [x] Debug showcase windows are all interactive (not visual-only).
- [ ] Primitive-root insertion is sufficient to stand up each major UI feature.
- [ ] No new reusable UI bundles introduced.
- [ ] Layering and input arbitration are owner-scoped and deterministic.
- [ ] No known B0001 query conflicts in UI systems.
