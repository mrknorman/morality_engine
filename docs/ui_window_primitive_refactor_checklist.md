# UI Window Primitive Refactor Checklist

Goal: make window behavior a first-class UI primitive under `systems::ui`, with sprite skinning as an adapter layer and clean migration from legacy `entities::sprites::window` call sites.

## Stage 0: Baseline + Guardrails
- [x] Keep repo compiling after every stage (`cargo check`).
- [ ] Keep behavior parity for drag/resize/close/z-order during migration.
- [ ] Keep query-safety contracts explicit (`ParamSet` / `Without<T>` comments/tests).

## Stage 1: Primitive Home + Renamed API (compat mode)
- [x] Add canonical module: `src/systems/ui/window/mod.rs`.
- [x] Move existing window logic to UI module (current logic copied into canonical module).
- [x] Add new naming aliases in canonical module:
  - `UiWindowPlugin`, `UiWindowSystem`, `UiWindow`, `UiWindowTitle`
  - `UiWindowContentHost`, `UiWindowContentMetrics`, `UiWindowOverflowPolicy`
  - `UiWindowContentRect`, `UiWindowResizeInProgress`
  - `UiWindowActions`, `UiWindowSounds`
- [x] Convert `src/entities/sprites/window.rs` to compatibility shim re-exporting canonical UI window module.
- [x] Expose `window` from `src/systems/ui/mod.rs`.

## Stage 2: Plugin Ownership + Composition Boundaries
- [x] Ensure startup/plugin wiring treats window as a UI primitive (not sprite-owned behavior).
- [x] Keep sprite-only construction concerns in `entities/sprites` (skin adapter).
- [x] Remove any remaining behavior ownership from sprite module.

## Stage 3: Caller Migration (new names)
- [x] Migrate `systems/ui/menu/debug_showcase.rs` imports to `systems::ui::window::UiWindow*` names.
- [x] Migrate `entities/text/mod.rs` integration imports to `UiWindow*` names.
- [x] Migrate scene/startup imports using `WindowTitle` to `UiWindowTitle` where practical.
- [x] Migrate interaction enum imports to `UiWindowActions` / `UiWindowSounds`.
- [ ] Keep temporary aliases in place until migration complete.

## Stage 4: Adapter Cleanup
- [x] Reduce compatibility shim to minimal aliases or remove once no legacy imports remain.
- [x] Ensure no module imports `entities::sprites::window` for behavior ownership.

## Stage 5: Tests + Docs
- [ ] Add/adjust tests that prove:
  - window primitive initializes from UI module,
  - drag region is valid,
  - close action routing still works,
  - no B0001 query conflicts.
  - Status: first three covered by `ui_window_*` tests in `src/systems/ui/window/mod.rs`; B0001 coverage still pending.
- [x] Update UI docs (`docs/ui_ecs_reference.md`, `docs/ui_compliance_matrix.md`) to point to canonical `systems/ui/window` ownership.

## Stage 6: Finalization
- [ ] Remove migration aliases that are no longer needed.
- [ ] Final cleanup pass for dead code and imports.
- [ ] Final `cargo check` and targeted UI regression run.
