# Level Select + TextInput/Search Refactor Plan

Last updated: 2026-02-24

## Goal

Refactor the level select overlay into a robust, extensible UI flow with:

- full content reachability (no clipped/off-screen rows)
- reusable `TextInputBox` primitive
- `SearchBox` composed from `TextInputBox`
- filesystem-style folder/file navigation
- dev/release unlock policy separation
- dev launch choice modal (continue campaign / play once / cancel)

## Status Snapshot

- Stage 1 (`TextInputBox` primitive): completed (`896ceb3`).
- Stage 2 (`SearchBox` composition primitive): completed (`896ceb3`).
- Stage 3 (catalog/tree model extraction): completed (folder/file rows + expansion state + deterministic row projection).
- Stage 4 (scroll root-cause fix): completed (explicit list scroll root + deterministic extent/focus-follow).
- Stage 5 (search integration): completed (SearchBox wiring + incremental filtered projection + ancestor inclusion).

## Architecture Direction

1. Put reusable primitives in `src/systems/ui/*`.
2. Keep level select as composition/policy in `src/systems/ui/menu/*`.
3. Keep scene queue behavior in scene runtime (`src/scenes/*`), not in UI primitives.
4. Keep owner/layer contracts explicit (`UiLayer`, `UiInputPolicy`, `UiInteractionState`).
5. Keep query-safety contracts explicit (`ParamSet`, `Without<T>`, no aliasing writes).

## Scope

In scope:

- level select window behavior and UX
- text input/search primitives
- folder/file tree model and rendering
- unlock gating and launch-mode modal routing
- tests and docs for the refactor

Out of scope:

- save-file persistence for unlocks
- graph editor implementation
- non-level-select menu redesign

## Stage Plan

## Stage 0 - Baseline and Contracts

Deliverables:

- Baseline tests for current level-select command flow and overlay lifecycle.
- Contract notes for expected keyboard/mouse behavior.
- This plan becomes canonical for this refactor.

Acceptance:

- Current behavior captured in tests before structural changes.
- No regressions in existing menu tests.

Checkpoint commit:

- `docs/tests(level-select): baseline coverage and refactor contract`

## Stage 1 - New Primitive: `TextInputBox`

Deliverables:

- New primitive module under `src/systems/ui/text_input_box.rs`.
- Root component with required components and insertion hook wiring.
- Value state resource/components for text content and caret position.
- Keyboard handling (character input, backspace/delete, left/right caret, submit/cancel hooks).
- Focus-aware owner/layer/policy gating.

Acceptance:

- Primitive works outside level select (standalone smoke test harness).
- No dependence on menu-specific enums or dilemma domain types.
- Query-safe system initialization tests added.

Checkpoint commit:

- `feat(ui): add TextInputBox primitive`

## Stage 2 - New Primitive: `SearchBox` (Composed)

Deliverables:

- New module `src/systems/ui/search_box.rs`.
- `SearchBox` composes `TextInputBox` and emits normalized query state/events.
- Placeholder text, clear action, and incremental update support.

Acceptance:

- `SearchBox` contains no domain-specific filtering logic.
- Incremental query updates are deterministic and test-covered.

Checkpoint commit:

- `feat(ui): add SearchBox composed from TextInputBox`

## Stage 3 - Level Catalog + Filesystem Tree Model

Deliverables:

- Extract level definitions from hardcoded row list into typed catalog/tree model.
- Folder/file node types with stable IDs and deterministic ordering.
- Flattened visible-row projection for rendering and selection.

Acceptance:

- Existing levels fully represented in tree model.
- Folder open/close behavior is deterministic under keyboard and click paths.

Checkpoint commit:

- `refactor(level-select): extract catalog and tree row model`

## Stage 4 - Scroll Refactor (Root Cause Fix)

Deliverables:

- Replace fixed Y-position rows with a `ScrollableRoot`-driven list.
- Add scrollbar visuals and row extent tracking.
- Add focus-follow scrolling so selected row is always visible.

Acceptance:

- Last row is reachable via wheel, keyboard, and scrollbar drag.
- No row can become permanently unreachable because of viewport size.

Checkpoint commit:

- `feat(level-select): migrate list to scroll primitive`

## Stage 5 - Search Integration

Deliverables:

- Place `SearchBox` at top of level select window.
- Incremental filtering of visible tree rows.
- Auto-expand ancestor folders for matching files.
- Preserve/repair selected row on query changes.

Acceptance:

- Typing updates result list per keystroke.
- Empty query restores full tree state.
- Keyboard navigation remains stable while filtering.

Checkpoint commit:

- `feat(level-select): incremental search and filtered tree projection`

## Stage 6 - Unlock Policy and Gating

Deliverables:

- `LevelUnlockPolicy` model:
  - dev mode: all dilemmas unlocked
  - release mode: only campaign-reached dilemmas unlocked
- `LevelUnlockState` tracking with session-scoped progression data.
- Locked rows visibly differentiated and non-activating.

Acceptance:

- Dev builds can launch any level.
- Release behavior only enables reached levels.
- Policy is centralized and test-covered.

Checkpoint commit:

- `feat(level-select): unlock policy and campaign-reached gating`

## Stage 7 - Dev Launch Choice Modal

Deliverables:

- Reuse modal primitive flow for dev-only launch choice:
  - continue campaign from this point
  - play this level once
  - cancel
- Add scene queue helper for campaign continuation from a selected dilemma.

Acceptance:

- Dev mode shows modal before launch.
- Cancel is side-effect free.
- Play-once preserves existing single-level semantics.

Checkpoint commit:

- `feat(level-select): dev launch mode modal and routing`

## Stage 8 - Cleanup, Review, Docs

Deliverables:

- Code review/cleanup pass across level select module boundaries.
- Remove obsolete constants and duplicated logic.
- Update docs (`ui_ecs_reference`, level-select usage doc, backlog item state).

Acceptance:

- Clear separation: primitive vs composition vs scene runtime.
- No dead code from pre-refactor list implementation.

Checkpoint commit:

- `chore(level-select): cleanup and docs pass`

## Validation Matrix

Manual:

1. Open level select and reach top/bottom with wheel.
2. Reach top/bottom with keyboard only.
3. Toggle folders by keyboard and mouse.
4. Type query, verify incremental filtering and restore on clear.
5. Verify locked/unlocked visuals and activation rules in release-mode path.
6. In dev mode, verify launch modal options and resulting flow behavior.

Automated:

1. Primitive init tests for `TextInputBox`/`SearchBox`.
2. Reducer/effects tests for new level-select commands.
3. Scroll focus-follow and row-visibility tests for level-select adapter path.
4. Unlock policy tests across dev/release predicates.

## Working Rules

1. Stage boundaries require a clean repo and one checkpoint commit.
2. Run `cargo check` at each stage boundary.
3. Keep primitive modules domain-agnostic.
4. Record deviations from this plan in this file with date + rationale.
